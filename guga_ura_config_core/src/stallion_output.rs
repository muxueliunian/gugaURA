//! 种马/玩家数据输出
//!
//! 从已解码的玩家个人主页响应中提取 stallion_data 和 player_profile，
//! 按规范格式写入文件系统，供 Stallion Runner 消费。

use crate::config::Config;
use chrono::{Local, NaiveDateTime, TimeZone};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// 种马输出设置
#[derive(Debug, Clone)]
pub struct StallionOutputSettings {
    pub enabled: bool,
    pub output_dir: PathBuf,
}

/// 种马输出结果
#[derive(Debug, Default)]
pub struct StallionOutputResult {
    /// stallion_data 输出路径
    pub stallion_data_path: Option<PathBuf>,
    /// player_profile 输出路径
    pub player_profile_path: Option<PathBuf>,
    /// 处理过程中的错误信息
    pub error: Option<String>,
}

/// 从 EXE 同级配置中解析种马输出设置
pub fn resolve_stallion_output_settings() -> StallionOutputSettings {
    let config = Config::load_from_exe_dir();
    StallionOutputSettings {
        enabled: config.stallion_output_enabled,
        output_dir: resolve_output_dir(config.stallion_output_dir.as_deref()),
    }
}

/// 默认输出根目录
pub fn default_stallion_output_dir() -> PathBuf {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        exe_path.push("stallion_output");
        return exe_path;
    }
    PathBuf::from("stallion_output")
}

/// 尝试从已解码的 response payload 中提取种马和玩家数据并输出到文件
///
/// 仅当 payload 包含 `data.partner_chara_info_array` 时触发（玩家个人主页响应）
pub fn extract_and_write(
    payload: &Value,
    direction: &str,
    route: &str,
    output_dir: &Path,
) -> StallionOutputResult {
    let mut result = StallionOutputResult::default();

    if direction != "response" && !route.to_ascii_lowercase().contains("response") {
        return result;
    }

    let data = match payload.get("data") {
        Some(d) => d,
        None => return result,
    };

    // 检测是否为玩家个人主页响应：必须同时有 partner_chara_info_array 和 user_info_summary
    let partner_array = match data.get("partner_chara_info_array") {
        Some(arr) if arr.is_array() => arr,
        _ => return result,
    };
    let user_summary = match data.get("user_info_summary") {
        Some(obj) if obj.is_object() => obj,
        _ => return result,
    };

    // 提取 viewer_id（种马主人 ID）
    let viewer_id = match user_summary.get("viewer_id").and_then(|v| v.as_u64()) {
        Some(id) => id,
        None => {
            result.error = Some("user_info_summary 中缺少 viewer_id".to_string());
            return result;
        }
    };

    let now = Local::now();
    let timestamp = build_output_timestamp(now);
    let captured_at = now.to_rfc3339();

    // 输出 stallion_data
    match build_stallion_data(viewer_id, &captured_at, partner_array) {
        Ok(stallion_json) => {
            let stallion_dir = output_dir.join("stallion_data");
            let filename = format!("stallion_data_{}_{}.json", viewer_id, timestamp);
            match write_json_file(&stallion_dir, &filename, &stallion_json) {
                Ok(path) => result.stallion_data_path = Some(path),
                Err(e) => {
                    result.error = Some(format!("写入 stallion_data 失败: {}", e));
                    return result;
                }
            }
        }
        Err(e) => {
            result.error = Some(format!("构建 stallion_data 失败: {}", e));
            return result;
        }
    }

    // 输出 player_profile
    match build_player_profile(viewer_id, &captured_at, data) {
        Ok(profile_json) => {
            let profile_dir = output_dir.join("player_profile");
            let filename = format!("player_profile_{}_{}.json", viewer_id, timestamp);
            match write_json_file(&profile_dir, &filename, &profile_json) {
                Ok(path) => result.player_profile_path = Some(path),
                Err(e) => {
                    let msg = format!("写入 player_profile 失败: {}", e);
                    result.error = Some(match result.error {
                        Some(prev) => format!("{}; {}", prev, msg),
                        None => msg,
                    });
                }
            }
        }
        Err(e) => {
            let msg = format!("构建 player_profile 失败: {}", e);
            result.error = Some(match result.error {
                Some(prev) => format!("{}; {}", prev, msg),
                None => msg,
            });
        }
    }

    result
}

/// 构建 stallion_data JSON
fn build_stallion_data(
    viewer_id: u64,
    captured_at: &str,
    partner_array: &Value,
) -> Result<Value, String> {
    let charas = partner_array
        .as_array()
        .ok_or("partner_chara_info_array 不是数组")?;

    let mut trained_charas = Vec::with_capacity(charas.len());
    for chara in charas {
        let mut chara_obj = chara.clone();
        // 过滤 succession_chara_array 中非直系父辈
        if let Some(succession) = chara_obj.get("succession_chara_array") {
            if let Some(arr) = succession.as_array() {
                let filtered: Vec<Value> = arr
                    .iter()
                    .filter(|item| {
                        item.get("position_id")
                            .and_then(|v| v.as_u64())
                            .map(|p| p == 10 || p == 20)
                            .unwrap_or(false)
                    })
                    .cloned()
                    .map(|mut item| {
                        convert_time_fields_in_object(&mut item);
                        item
                    })
                    .collect();
                chara_obj["succession_chara_array"] = Value::Array(filtered);
            }
        }
        // 转换种马对象内的时间字段
        convert_time_fields_in_object(&mut chara_obj);
        trained_charas.push(chara_obj);
    }

    Ok(json!({
        "type": "stallion_data",
        "viewer_id": viewer_id,
        "captured_at": captured_at,
        "trained_charas": trained_charas
    }))
}

/// 构建 player_profile JSON，将嵌套字段扁平化到 data 顶层
fn build_player_profile(viewer_id: u64, captured_at: &str, data: &Value) -> Result<Value, String> {
    let user_summary = data
        .get("user_info_summary")
        .and_then(|v| v.as_object())
        .ok_or("缺少 user_info_summary")?;

    let mut profile = Map::new();

    // 透传 user_info_summary 中的直属字段（排除嵌套对象后续单独处理的）
    for (key, value) in user_summary {
        match key.as_str() {
            // 嵌套对象需要展开，不直接透传
            "circle_info" | "circle_user" | "honor_data" => {}
            _ => {
                let mut v = value.clone();
                convert_time_fields_by_key(key, &mut v);
                profile.insert(key.clone(), v);
            }
        }
    }

    // 扁平化 circle_info
    if let Some(circle_info) = user_summary.get("circle_info").and_then(|v| v.as_object()) {
        if let Some(id) = circle_info.get("circle_id") {
            profile.insert("circle_id".to_string(), id.clone());
        }
        if let Some(name) = circle_info.get("name") {
            profile.insert("circle_name".to_string(), name.clone());
        }
    }

    // 扁平化 circle_user
    if let Some(circle_user) = user_summary.get("circle_user").and_then(|v| v.as_object()) {
        if let Some(m) = circle_user.get("membership") {
            profile.insert("membership".to_string(), m.clone());
        }
        if let Some(jt) = circle_user.get("join_time") {
            let mut v = jt.clone();
            convert_game_time_value(&mut v);
            profile.insert("join_time".to_string(), v);
        }
    }

    // 扁平化 honor_data
    if let Some(honor_data) = user_summary.get("honor_data").and_then(|v| v.as_object()) {
        if let Some(id) = honor_data.get("honor_id") {
            profile.insert("honor_id".to_string(), id.clone());
        }
    }

    // 从 data 顶层提取 follower_num / own_follow_num
    if let Some(v) = data.get("follower_num") {
        profile.insert("follower_num".to_string(), v.clone());
    }
    if let Some(v) = data.get("own_follow_num") {
        profile.insert("own_follow_num".to_string(), v.clone());
    }

    Ok(json!({
        "type": "player_profile",
        "viewer_id": viewer_id,
        "captured_at": captured_at,
        "data": profile
    }))
}

/// 将游戏时间格式 "2026-04-06 22:40:05" 转换为 ISO 8601 "+09:00"
fn convert_game_time_str(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() || s.starts_with("0000") {
        return None;
    }
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()?;
    // 游戏服务器时区为 JST (+09:00)
    let jst = chrono::FixedOffset::east_opt(9 * 3600)?;
    let dt = jst.from_local_datetime(&naive).single()?;
    Some(dt.to_rfc3339())
}

/// 对 Value 中的时间字段进行转换（就地修改）
fn convert_game_time_value(v: &mut Value) {
    if let Some(s) = v.as_str() {
        if let Some(converted) = convert_game_time_str(s) {
            *v = Value::String(converted);
        }
    }
}

/// 已知的时间字段名
const TIME_FIELD_NAMES: &[&str] = &[
    "create_time",
    "register_time",
    "last_login_time",
    "join_time",
    "follow_time",
    "follower_time",
    "make_time",
    "item_request_end_time",
    "penalty_end_time",
    "ranking_result_check_time",
    "expiration_date",
];

/// 根据字段名判断是否需要转换时间
fn convert_time_fields_by_key(key: &str, value: &mut Value) {
    if TIME_FIELD_NAMES.contains(&key) {
        convert_game_time_value(value);
    }
}

/// 递归转换对象内所有已知时间字段
fn convert_time_fields_in_object(value: &mut Value) {
    if let Some(obj) = value.as_object_mut() {
        let keys: Vec<String> = obj.keys().cloned().collect();
        for key in keys {
            if let Some(v) = obj.get_mut(&key) {
                if TIME_FIELD_NAMES.contains(&key.as_str()) {
                    convert_game_time_value(v);
                } else if v.is_object() {
                    convert_time_fields_in_object(v);
                } else if v.is_array() {
                    if let Some(arr) = v.as_array_mut() {
                        for item in arr.iter_mut() {
                            if item.is_object() {
                                convert_time_fields_in_object(item);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 写入 JSON 文件到指定目录
fn write_json_file(dir: &Path, filename: &str, value: &Value) -> Result<PathBuf, String> {
    fs::create_dir_all(dir).map_err(|e| format!("创建目录 {} 失败: {}", dir.display(), e))?;
    let file_path = dir.join(filename);
    let json_str = serde_json::to_string(value).map_err(|e| format!("JSON 序列化失败: {}", e))?;
    fs::write(&file_path, json_str)
        .map_err(|e| format!("写入 {} 失败: {}", file_path.display(), e))?;
    Ok(file_path)
}

fn build_output_timestamp(now: chrono::DateTime<Local>) -> String {
    let sequence = OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{}_{}", now.format("%Y%m%d%H%M%S%3f"), sequence)
}

/// 解析输出目录（支持自定义绝对路径，默认为 <exe>/stallion_output/）
fn resolve_output_dir(configured: Option<&str>) -> PathBuf {
    if let Some(dir) = configured {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.is_absolute() {
                return path;
            }
        }
    }
    default_stallion_output_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn convert_game_time_str_should_convert_valid_time() {
        let result = convert_game_time_str("2026-04-06 22:40:05");
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(s.contains("2026-04-06"));
        assert!(s.contains("+09:00"));
    }

    #[test]
    fn convert_game_time_str_should_skip_zero_time() {
        assert!(convert_game_time_str("0000-00-00 00:00:00").is_none());
    }

    #[test]
    fn convert_game_time_str_should_skip_empty() {
        assert!(convert_game_time_str("").is_none());
    }

    #[test]
    fn build_stallion_data_should_filter_succession() {
        let partner_array = json!([{
            "trained_chara_id": 975,
            "card_id": 112901,
            "speed": 2153,
            "create_time": "2026-03-23 07:34:47",
            "succession_chara_array": [
                {"position_id": 10, "card_id": 100301, "factor_info_array": []},
                {"position_id": 11, "card_id": 100302, "factor_info_array": []},
                {"position_id": 12, "card_id": 100303, "factor_info_array": []},
                {"position_id": 20, "card_id": 100501, "factor_info_array": []},
                {"position_id": 21, "card_id": 100502, "factor_info_array": []},
                {"position_id": 22, "card_id": 100503, "factor_info_array": []}
            ]
        }]);

        let result = build_stallion_data(12345, "2026-04-07T14:30:25+09:00", &partner_array)
            .expect("构建失败");

        assert_eq!(result["type"], "stallion_data");
        assert_eq!(result["viewer_id"], 12345);
        let charas = result["trained_charas"].as_array().unwrap();
        assert_eq!(charas.len(), 1);
        let succession = charas[0]["succession_chara_array"].as_array().unwrap();
        assert_eq!(succession.len(), 2);
        assert_eq!(succession[0]["position_id"], 10);
        assert_eq!(succession[1]["position_id"], 20);
    }

    #[test]
    fn build_stallion_data_should_convert_time_fields() {
        let partner_array = json!([{
            "trained_chara_id": 100,
            "create_time": "2026-03-23 07:34:47",
            "register_time": "2026-03-23 07:34:47",
            "succession_chara_array": []
        }]);

        let result =
            build_stallion_data(1, "2026-04-07T14:30:25+09:00", &partner_array).expect("构建失败");

        let chara = &result["trained_charas"][0];
        assert!(chara["create_time"].as_str().unwrap().contains("+09:00"));
        assert!(chara["register_time"].as_str().unwrap().contains("+09:00"));
    }

    #[test]
    fn build_player_profile_should_flatten_nested_fields() {
        let data = json!({
            "user_info_summary": {
                "viewer_id": 681803745355_u64,
                "name": "测试玩家",
                "fan": 107260203,
                "last_login_time": "2026-04-06 22:40:05",
                "circle_info": {
                    "circle_id": 668677582,
                    "name": "TestCircle"
                },
                "circle_user": {
                    "membership": 1,
                    "join_time": "2026-04-01 18:23:26"
                },
                "honor_data": {
                    "honor_id": 760001,
                    "step": 732
                }
            },
            "follower_num": 12,
            "own_follow_num": 1
        });

        let result = build_player_profile(681803745355, "2026-04-07T14:32:00+09:00", &data)
            .expect("构建失败");

        assert_eq!(result["type"], "player_profile");
        assert_eq!(result["viewer_id"], 681803745355_u64);
        let profile = &result["data"];
        assert_eq!(profile["name"], "测试玩家");
        assert_eq!(profile["fan"], 107260203);
        assert_eq!(profile["circle_id"], 668677582);
        assert_eq!(profile["circle_name"], "TestCircle");
        assert_eq!(profile["membership"], 1);
        assert_eq!(profile["honor_id"], 760001);
        assert_eq!(profile["follower_num"], 12);
        assert_eq!(profile["own_follow_num"], 1);
        // join_time 应已转换为 ISO 8601
        assert!(profile["join_time"].as_str().unwrap().contains("+09:00"));
        // last_login_time 应已转换
        assert!(profile["last_login_time"]
            .as_str()
            .unwrap()
            .contains("+09:00"));
        // circle_info / circle_user / honor_data 不应作为嵌套对象出现
        assert!(profile.get("circle_info").is_none());
        assert!(profile.get("circle_user").is_none());
        assert!(profile.get("honor_data").is_none());
    }

    #[test]
    fn extract_and_write_should_skip_non_profile_payload() {
        let payload = json!({"data": {"circle_info": {}}});
        let result = extract_and_write(&payload, "response", "/notify/response", Path::new("/tmp"));
        assert!(result.stallion_data_path.is_none());
        assert!(result.player_profile_path.is_none());
        assert!(result.error.is_none());
    }

    #[test]
    fn extract_and_write_should_not_overwrite_same_second_outputs() {
        let output_dir = tempfile::tempdir().expect("创建临时目录失败");
        let payload = json!({
            "data": {
                "partner_chara_info_array": [{
                    "trained_chara_id": 975,
                    "succession_chara_array": []
                }],
                "user_info_summary": {
                    "viewer_id": 681803745355_u64,
                    "name": "测试玩家"
                }
            }
        });

        let first = extract_and_write(&payload, "response", "/notify/response", output_dir.path());
        let second = extract_and_write(&payload, "response", "/notify/response", output_dir.path());

        assert!(first.error.is_none());
        assert!(second.error.is_none());
        assert_ne!(first.stallion_data_path, second.stallion_data_path);
        assert_ne!(first.player_profile_path, second.player_profile_path);
        assert_eq!(
            fs::read_dir(output_dir.path().join("stallion_data"))
                .expect("读取 stallion_data 目录失败")
                .count(),
            2
        );
        assert_eq!(
            fs::read_dir(output_dir.path().join("player_profile"))
                .expect("读取 player_profile 目录失败")
                .count(),
            2
        );
    }

    #[test]
    fn resolve_output_dir_should_use_custom_path() {
        let path = resolve_output_dir(Some("C:\\custom\\output"));
        assert_eq!(path, PathBuf::from("C:\\custom\\output"));
    }

    #[test]
    fn resolve_output_dir_should_fallback_to_default() {
        let path = resolve_output_dir(None);
        assert!(path.to_string_lossy().contains("stallion_output"));
    }
}

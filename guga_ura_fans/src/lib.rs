use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "guga_ura_config.json";
const DEFAULT_FANS_DIR_NAME: &str = "fans";

fn default_fans_enabled() -> bool {
    true
}

#[derive(Debug, Default, Deserialize)]
struct ReceiverConfig {
    #[serde(default = "default_fans_enabled")]
    fans_enabled: bool,

    #[serde(default)]
    fans_output_dir: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FansSettings {
    pub enabled: bool,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FanRecord {
    pub name: String,
    pub fan: u64,
    pub circle_name: String,
    pub ts: String,
    pub viewer_id: u64,
    pub comment: String,
    pub rank_score: u64,
    pub circle_id: u64,
}

pub fn default_fans_output_dir() -> PathBuf {
    exe_dir().join(DEFAULT_FANS_DIR_NAME)
}

pub fn resolve_fans_output_dir_from_exe_config() -> PathBuf {
    resolve_fans_settings_from_exe_config().output_dir
}

pub fn resolve_fans_settings_from_exe_config() -> FansSettings {
    let exe_dir = exe_dir();
    let config_path = exe_dir.join(CONFIG_FILE_NAME);

    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Some(config) = parse_receiver_config(&content) {
            if let Some(raw) = config.fans_output_dir {
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    let path = PathBuf::from(trimmed);
                    let output_dir = if path.is_absolute() {
                        path
                    } else {
                        exe_dir.join(path)
                    };
                    return FansSettings {
                        enabled: config.fans_enabled,
                        output_dir,
                    };
                }
            }
            return FansSettings {
                enabled: config.fans_enabled,
                output_dir: default_fans_output_dir(),
            };
        }
    }

    FansSettings {
        enabled: default_fans_enabled(),
        output_dir: default_fans_output_dir(),
    }
}

pub fn upsert_fans_from_decoded_payload(
    decoded_payload: &Value,
    direction: &str,
    route: &str,
    received_at_unix_ms: u128,
    fans_output_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    if !direction.eq_ignore_ascii_case("response") {
        return Ok(None);
    }
    if !route.to_ascii_lowercase().contains("response") {
        return Ok(None);
    }

    let (ts, extracted) = extract_records(decoded_payload, received_at_unix_ms);
    if extracted.is_empty() {
        return Ok(None);
    }

    fs::create_dir_all(fans_output_dir).map_err(|e| {
        format!(
            "create fans dir {} failed: {}",
            fans_output_dir.display(),
            e
        )
    })?;

    let file_path = fans_output_dir.join(format!("{}.json", ts));
    let mut merged = load_existing_records(&file_path)?;

    for (viewer_key, record) in extracted {
        let value = serde_json::to_value(record)
            .map_err(|e| format!("serialize fan record failed: {}", e))?;
        merged.insert(viewer_key, value);
    }

    write_json_atomic(&file_path, &Value::Object(merged))?;
    Ok(Some(file_path))
}

fn extract_records(
    decoded_payload: &Value,
    received_at_unix_ms: u128,
) -> (String, Vec<(String, FanRecord)>) {
    let ts = yyyymmdd_from_unix_millis(received_at_unix_ms);
    let mut out = Vec::new();

    let data = decoded_payload.get("data").and_then(Value::as_object);
    let Some(data) = data else {
        return (ts, out);
    };

    let circle_info = data.get("circle_info").and_then(Value::as_object);
    let Some(circle_info) = circle_info else {
        return (ts, out);
    };

    let circle_id = value_to_u64(circle_info.get("circle_id")).unwrap_or(0);
    let circle_name = value_to_string(circle_info.get("name"));

    let users = data
        .get("summary_user_info_array")
        .and_then(Value::as_array);
    let Some(users) = users else {
        return (ts, out);
    };

    for user in users {
        let Some(user_obj) = user.as_object() else {
            continue;
        };
        let Some(viewer_id) = value_to_u64(user_obj.get("viewer_id")) else {
            continue;
        };

        let record = FanRecord {
            name: value_to_string(user_obj.get("name")),
            fan: value_to_u64(user_obj.get("fan")).unwrap_or(0),
            circle_name: circle_name.clone(),
            ts: ts.clone(),
            viewer_id,
            comment: value_to_string(user_obj.get("comment")),
            rank_score: value_to_u64(user_obj.get("rank_score")).unwrap_or(0),
            circle_id,
        };
        out.push((viewer_id.to_string(), record));
    }

    (ts, out)
}

fn load_existing_records(file_path: &Path) -> Result<Map<String, Value>, String> {
    if !file_path.exists() {
        return Ok(Map::new());
    }

    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("read {} failed: {}", file_path.display(), e))?;
    let value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("parse {} failed: {}", file_path.display(), e))?;

    match value {
        Value::Object(map) => Ok(map),
        _ => Err(format!(
            "{} is not a JSON object; expected {{\"viewer_id\": {{...}}}}",
            file_path.display()
        )),
    }
}

fn write_json_atomic(file_path: &Path, value: &Value) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("serialize {} failed: {}", file_path.display(), e))?;

    let temp_path = file_path.with_extension(format!("json.tmp.{}", std::process::id()));
    fs::write(&temp_path, json)
        .map_err(|e| format!("write temp {} failed: {}", temp_path.display(), e))?;

    if file_path.exists() {
        fs::remove_file(file_path)
            .map_err(|e| format!("remove old {} failed: {}", file_path.display(), e))?;
    }

    match fs::rename(&temp_path, file_path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            Err(format!(
                "rename {} -> {} failed: {}",
                temp_path.display(),
                file_path.display(),
                e
            ))
        }
    }
}

fn value_to_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn value_to_u64(value: Option<&Value>) -> Option<u64> {
    match value {
        Some(Value::Number(n)) => n.as_u64().or_else(|| {
            n.as_i64()
                .and_then(|v| if v >= 0 { Some(v as u64) } else { None })
        }),
        Some(Value::String(s)) => s.trim().parse::<u64>().ok(),
        _ => None,
    }
}

fn yyyymmdd_from_unix_millis(ms: u128) -> String {
    let Ok(ms_i64) = i64::try_from(ms) else {
        return Local::now().format("%Y%m%d").to_string();
    };
    match Local.timestamp_millis_opt(ms_i64).single() {
        Some(dt) => dt.format("%Y%m%d").to_string(),
        None => Local::now().format("%Y%m%d").to_string(),
    }
}

fn parse_receiver_config(content: &str) -> Option<ReceiverConfig> {
    if let Ok(cfg) = serde_json::from_str::<ReceiverConfig>(content) {
        return Some(cfg);
    }

    let trimmed = content.trim_start_matches('\u{feff}');
    serde_json::from_str::<ReceiverConfig>(trimmed).ok()
}

fn exe_dir() -> PathBuf {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        return exe_path;
    }
    PathBuf::from(".")
}

/// 解码 payload（msgpack 或 json）
pub fn decode_payload(body: &[u8]) -> Result<(&'static str, serde_json::Value), String> {
    match rmp_serde::from_slice::<serde_json::Value>(body) {
        Ok(v) => Ok(("msgpack", v)),
        Err(msgpack_err) => match serde_json::from_slice::<serde_json::Value>(body) {
            Ok(v) => Ok(("json", v)),
            Err(json_err) => Err(format!(
                "Decode failed (msgpack: {}; json: {})",
                msgpack_err, json_err
            )),
        },
    }
}

/// 从路由推断方向（request/response/unknown）
pub fn infer_direction(route: &str) -> String {
    let route_lower = route.to_ascii_lowercase();
    if route_lower.contains("request") {
        "request".to_string()
    } else if route_lower.contains("response") {
        "response".to_string()
    } else {
        "unknown".to_string()
    }
}

/// 判断是否应该持久化 debug payload
pub fn should_persist_debug_payload(direction: &str, route: &str) -> bool {
    if direction.eq_ignore_ascii_case("response") {
        return true;
    }
    route.to_ascii_lowercase().contains("response")
}

/// 获取当前时间戳（毫秒）
pub fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn build_payload(
        circle_id: u64,
        circle_name: &str,
        viewer_id: u64,
        fan: u64,
        rank_score: u64,
    ) -> Value {
        json!({
            "data": {
                "circle_info": {
                    "circle_id": circle_id,
                    "name": circle_name
                },
                "summary_user_info_array": [
                    {
                        "viewer_id": viewer_id,
                        "name": format!("user-{}", viewer_id),
                        "fan": fan,
                        "comment": "hello",
                        "rank_score": rank_score
                    }
                ]
            }
        })
    }

    #[test]
    fn upsert_is_last_write_wins_for_same_viewer() {
        let dir = tempfile::tempdir().expect("tempdir");
        let payload1 = build_payload(1, "circle-a", 100, 10, 20);
        let payload2 = build_payload(2, "circle-b", 100, 99, 88);

        let written1 = upsert_fans_from_decoded_payload(
            &payload1,
            "response",
            "/notify/response",
            1_772_641_517_934,
            dir.path(),
        )
        .expect("write1")
        .expect("path1");

        let written2 = upsert_fans_from_decoded_payload(
            &payload2,
            "response",
            "/notify/response",
            1_772_641_517_935,
            dir.path(),
        )
        .expect("write2")
        .expect("path2");

        assert_eq!(written1, written2);

        let content = fs::read_to_string(&written2).expect("read");
        let value: Value = serde_json::from_str(&content).expect("json");
        let row = value.get("100").expect("viewer exists");
        assert_eq!(row.get("fan").and_then(Value::as_u64), Some(99));
        assert_eq!(row.get("rank_score").and_then(Value::as_u64), Some(88));
        assert_eq!(row.get("circle_id").and_then(Value::as_u64), Some(2));
        assert_eq!(
            row.get("circle_name").and_then(Value::as_str),
            Some("circle-b")
        );
    }

    #[test]
    fn supports_multiple_circles_in_one_day_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ts_ms = 1_772_641_517_934;
        let payload1 = build_payload(11, "circle-x", 101, 1, 1);
        let payload2 = build_payload(22, "circle-y", 202, 2, 2);

        let written = upsert_fans_from_decoded_payload(
            &payload1,
            "response",
            "/notify/response",
            ts_ms,
            dir.path(),
        )
        .expect("write1")
        .expect("path");

        upsert_fans_from_decoded_payload(
            &payload2,
            "response",
            "/notify/response",
            ts_ms,
            dir.path(),
        )
        .expect("write2")
        .expect("path2");

        let content = fs::read_to_string(&written).expect("read");
        let value: Value = serde_json::from_str(&content).expect("json");

        assert!(value.get("101").is_some());
        assert!(value.get("202").is_some());
        assert_eq!(
            value
                .get("101")
                .and_then(|v| v.get("circle_id"))
                .and_then(Value::as_u64),
            Some(11)
        );
        assert_eq!(
            value
                .get("202")
                .and_then(|v| v.get("circle_id"))
                .and_then(Value::as_u64),
            Some(22)
        );
    }

    #[test]
    fn real_debug_sample_matches_flattened_shape() {
        let sample_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("debug")
            .join("response_000005_1772641517934.json");
        if !sample_path.exists() {
            return;
        }

        let content = fs::read_to_string(sample_path).expect("read sample");
        let wrapped: Value = serde_json::from_str(&content).expect("parse sample json");
        let payload = wrapped.get("payload").expect("payload exists");
        let received_at = wrapped
            .get("received_at_unix_ms")
            .and_then(Value::as_u64)
            .expect("received_at_unix_ms exists");

        let out_dir = tempfile::tempdir().expect("tempdir");
        let out_path = upsert_fans_from_decoded_payload(
            payload,
            "response",
            "/notify/response",
            received_at as u128,
            out_dir.path(),
        )
        .expect("aggregate ok")
        .expect("file written");

        let out_content = fs::read_to_string(out_path).expect("read output");
        let out_json: Value = serde_json::from_str(&out_content).expect("parse output");
        let out_obj = out_json.as_object().expect("output object");
        assert_eq!(out_obj.len(), 30);

        let first = out_obj.values().next().expect("first row");
        let first_obj = first.as_object().expect("first row object");
        for key in [
            "name",
            "fan",
            "circle_name",
            "ts",
            "viewer_id",
            "comment",
            "rank_score",
            "circle_id",
        ] {
            assert!(first_obj.contains_key(key), "missing key {}", key);
        }
    }

    #[test]
    fn parse_receiver_config_with_bom() {
        let json = "\u{feff}{\"fans_enabled\":false,\"fans_output_dir\":\"./my_fans\"}";
        let cfg = parse_receiver_config(json).expect("parse with bom");
        assert!(!cfg.fans_enabled);
        assert_eq!(cfg.fans_output_dir.as_deref(), Some("./my_fans"));
    }
}

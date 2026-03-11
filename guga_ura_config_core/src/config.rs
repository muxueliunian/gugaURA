//! 配置结构

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Notifier服务地址
    #[serde(default = "Config::default_notifier_host")]
    pub notifier_host: String,

    /// Receiver监听地址（接收端使用）
    #[serde(default = "Config::default_receiver_listen_addr")]
    pub receiver_listen_addr: String,

    /// HTTP超时时间(毫秒)
    #[serde(default = "Config::default_timeout_ms")]
    pub timeout_ms: u64,

    /// 目标帧数 (-1 表示使用游戏默认, 30/60/120/240 等)
    #[serde(default = "Config::default_target_fps")]
    pub target_fps: i32,

    /// VSync 设置 (-1 表示使用游戏默认, 0=关闭, 1=开启)
    #[serde(default = "Config::default_vsync_count")]
    pub vsync_count: i32,

    /// Debug 模式：将拦截的 msgpack 数据转为 JSON 保存到本地
    #[serde(default)]
    pub debug_mode: bool,

    /// Debug 输出目录（绝对路径，优先使用）
    #[serde(default)]
    pub debug_output_dir: Option<String>,

    /// Fans 聚合输出目录（绝对路径优先，接收端使用）
    #[serde(default)]
    pub fans_output_dir: Option<String>,

    /// Fans 数据保存开关（接收端聚合）
    #[serde(default = "Config::default_fans_enabled")]
    pub fans_enabled: bool,

    /// Relay 开关（接收端二次转发）
    #[serde(default = "Config::default_relay_enabled")]
    pub relay_enabled: bool,

    /// Relay 目标地址（接收端二次转发）
    #[serde(default)]
    pub relay_target_host: Option<String>,
}

impl Config {
    fn default_notifier_host() -> String {
        "http://127.0.0.1:4693".to_string()
    }

    fn default_receiver_listen_addr() -> String {
        "127.0.0.1:4693".to_string()
    }

    fn default_timeout_ms() -> u64 {
        100
    }

    fn default_target_fps() -> i32 {
        -1 // -1 表示使用游戏默认
    }

    fn default_vsync_count() -> i32 {
        -1 // -1 表示使用游戏默认
    }

    fn default_fans_enabled() -> bool {
        true
    }

    fn default_relay_enabled() -> bool {
        false
    }

    /// 获取配置文件路径（相对于游戏目录）
    pub fn config_path(game_dir: &Path) -> PathBuf {
        game_dir.join("guga_ura_config.json")
    }

    /// 获取配置文件路径（相对于配置工具 EXE 目录）
    pub fn exe_config_path() -> PathBuf {
        if let Ok(mut exe_path) = std::env::current_exe() {
            exe_path.pop();
            return exe_path.join("guga_ura_config.json");
        }
        PathBuf::from("guga_ura_config.json")
    }

    /// 从游戏目录加载配置
    pub fn load_from(game_dir: &Path) -> Config {
        let path = Self::config_path(game_dir);

        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = parse_config_json(&content) {
                    return config;
                }
            }
        }

        Config::default()
    }

    /// 判断游戏目录配置文件中是否显式包含某个顶层字段
    pub fn game_config_has_key(game_dir: &Path, key: &str) -> bool {
        let path = Self::config_path(game_dir);
        json_file_has_key(&path, key)
    }

    /// 从配置工具 EXE 目录加载配置
    pub fn load_from_exe_dir() -> Config {
        let path = Self::exe_config_path();

        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = parse_config_json(&content) {
                    return config;
                }
            }
        }

        Config::default()
    }

    /// 保存配置到游戏目录
    pub fn save_to(&self, game_dir: &Path) -> Result<(), String> {
        let path = Self::config_path(game_dir);
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("序列化错误: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("写入错误: {}", e))?;
        Ok(())
    }

    /// 保存配置到配置工具 EXE 目录
    pub fn save_to_exe_dir(&self) -> Result<(), String> {
        let path = Self::exe_config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("序列化错误: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("写入错误: {}", e))?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            notifier_host: Self::default_notifier_host(),
            receiver_listen_addr: Self::default_receiver_listen_addr(),
            timeout_ms: Self::default_timeout_ms(),
            target_fps: Self::default_target_fps(),
            vsync_count: Self::default_vsync_count(),
            debug_mode: false,
            debug_output_dir: None,
            fans_output_dir: None,
            fans_enabled: Self::default_fans_enabled(),
            relay_enabled: Self::default_relay_enabled(),
            relay_target_host: None,
        }
    }
}

fn parse_config_json(content: &str) -> Result<Config, serde_json::Error> {
    serde_json::from_str(content).or_else(|_| {
        let trimmed = content.trim_start_matches('\u{feff}');
        serde_json::from_str(trimmed)
    })
}

fn json_file_has_key(path: &Path, key: &str) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let trimmed = content.trim_start_matches('\u{feff}');
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return false;
    };
    value
        .as_object()
        .map(|obj| obj.contains_key(key))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{parse_config_json, Config};

    #[test]
    fn default_values_should_match_current_behavior() {
        let config = Config::default();

        assert_eq!(config.notifier_host, "http://127.0.0.1:4693");
        assert_eq!(config.receiver_listen_addr, "127.0.0.1:4693");
        assert_eq!(config.timeout_ms, 100);
        assert_eq!(config.target_fps, -1);
        assert_eq!(config.vsync_count, -1);
        assert!(!config.debug_mode);
        assert_eq!(config.debug_output_dir, None);
        assert_eq!(config.fans_output_dir, None);
        assert!(config.fans_enabled);
        assert!(!config.relay_enabled);
        assert_eq!(config.relay_target_host, None);
    }

    #[test]
    fn parse_config_json_should_support_bom() {
        let content = "\u{feff}{\"timeout_ms\":250,\"fans_enabled\":false}";

        let config = parse_config_json(content).expect("BOM 配置解析失败");

        assert_eq!(config.notifier_host, "http://127.0.0.1:4693");
        assert_eq!(config.receiver_listen_addr, "127.0.0.1:4693");
        assert_eq!(config.timeout_ms, 250);
        assert!(!config.fans_enabled);
        assert!(!config.relay_enabled);
        assert_eq!(config.relay_target_host, None);
    }

    #[test]
    fn legacy_config_should_remain_compatible() {
        let content = r#"{"notifier_host":"http://127.0.0.1:4800","timeout_ms":180}"#;

        let config = parse_config_json(content).expect("旧配置解析失败");

        assert_eq!(config.notifier_host, "http://127.0.0.1:4800");
        assert_eq!(config.receiver_listen_addr, "127.0.0.1:4693");
        assert_eq!(config.timeout_ms, 180);
        assert!(!config.relay_enabled);
        assert_eq!(config.relay_target_host, None);
    }

    #[test]
    fn new_fields_should_round_trip() {
        let config = Config {
            receiver_listen_addr: "127.0.0.1:4700".to_string(),
            relay_enabled: true,
            relay_target_host: Some("http://127.0.0.1:4800".to_string()),
            fans_enabled: false,
            ..Config::default()
        };

        let json = serde_json::to_string(&config).expect("序列化配置失败");
        let reparsed = parse_config_json(&json).expect("round-trip 解析失败");

        assert_eq!(reparsed.receiver_listen_addr, "127.0.0.1:4700");
        assert!(reparsed.relay_enabled);
        assert_eq!(
            reparsed.relay_target_host.as_deref(),
            Some("http://127.0.0.1:4800")
        );
        assert!(!reparsed.fans_enabled);
    }
}

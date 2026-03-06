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
}

impl Config {
    fn default_notifier_host() -> String {
        "http://127.0.0.1:4693".to_string()
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
            timeout_ms: Self::default_timeout_ms(),
            target_fps: Self::default_target_fps(),
            vsync_count: Self::default_vsync_count(),
            debug_mode: false,
            debug_output_dir: None,
            fans_output_dir: None,
            fans_enabled: Self::default_fans_enabled(),
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

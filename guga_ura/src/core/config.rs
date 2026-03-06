//! 配置管理
//!
//! 支持热重载：通过 ArcSwap 实现无锁读取 + 原子交换

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

    /// 获取配置文件路径
    pub fn config_path() -> PathBuf {
        // 配置文件放在DLL同目录下
        let mut path = std::env::current_exe().unwrap_or_default();
        path.pop();
        path.push("guga_ura_config.json");
        path
    }

    /// 加载配置
    pub fn load() -> Config {
        let path = Self::config_path();
        match Self::try_load() {
            Ok(config) => return config,
            Err(e) if path.exists() => {
                warn!(
                    "Failed to load config {}, keeping current/default config without overwriting file: {}",
                    path.display(),
                    e
                );
            }
            Err(e) => {
                warn!(
                    "Config file not found or unreadable, will create default at {}: {}",
                    path.display(),
                    e
                );
            }
        }

        let config = Config::default();
        if !path.exists() {
            match config.save() {
                Ok(()) => {
                    info!(
                        "Default config written to {} (debug_mode = {})",
                        path.display(),
                        config.debug_mode
                    );
                }
                Err(e) => {
                    warn!("Failed to save default config {}: {}", path.display(), e);
                }
            }
        }
        info!(
            "Using default config: notifier_host = {}, timeout_ms = {}, target_fps = {}, vsync_count = {}, debug_mode = {}, debug_output_dir = {:?}, fans_enabled = {}, fans_output_dir = {:?}",
            config.notifier_host,
            config.timeout_ms,
            config.target_fps,
            config.vsync_count,
            config.debug_mode,
            config.debug_output_dir,
            config.fans_enabled,
            config.fans_output_dir
        );
        config
    }

    /// 尝试从磁盘加载配置，失败时返回错误，不覆盖现有文件。
    pub fn try_load() -> Result<Config, String> {
        let path = Self::config_path();
        info!("Loading config from: {}", path.display());
        let config = load_from_path(&path)?;
        info!(
            "Config loaded from {}: notifier_host = {}, timeout_ms = {}, target_fps = {}, vsync_count = {}, debug_mode = {}, debug_output_dir = {:?}, fans_enabled = {}, fans_output_dir = {:?}",
            path.display(),
            config.notifier_host,
            config.timeout_ms,
            config.target_fps,
            config.vsync_count,
            config.debug_mode,
            config.debug_output_dir,
            config.fans_enabled,
            config.fans_output_dir
        );
        Ok(config)
    }

    /// 保存配置
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Write error: {}", e))?;
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

fn load_from_path(path: &Path) -> Result<Config, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("read {} failed: {}", path.display(), e))?;
    parse_config_json(&content).map_err(|e| format!("parse {} failed: {}", path.display(), e))
}

fn parse_config_json(content: &str) -> Result<Config, serde_json::Error> {
    serde_json::from_str(content).or_else(|_| {
        let trimmed = content.trim_start_matches('\u{feff}');
        serde_json::from_str(trimmed)
    })
}

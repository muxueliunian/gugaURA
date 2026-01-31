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
}

impl Config {
    fn default_notifier_host() -> String {
        "http://127.0.0.1:4693".to_string()
    }
    
    fn default_timeout_ms() -> u64 {
        100
    }
    
    fn default_target_fps() -> i32 {
        -1  // -1 表示使用游戏默认
    }
    
    fn default_vsync_count() -> i32 {
        -1  // -1 表示使用游戏默认
    }
    
    /// 获取配置文件路径（相对于游戏目录）
    pub fn config_path(game_dir: &Path) -> PathBuf {
        game_dir.join("guga_ura_config.json")
    }
    
    /// 从游戏目录加载配置
    pub fn load_from(game_dir: &Path) -> Config {
        let path = Self::config_path(game_dir);
        
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        
        Config::default()
    }
    
    /// 保存配置到游戏目录
    pub fn save_to(&self, game_dir: &Path) -> Result<(), String> {
        let path = Self::config_path(game_dir);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化错误: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("写入错误: {}", e))?;
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
        }
    }
}

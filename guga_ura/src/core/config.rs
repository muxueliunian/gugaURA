//! 配置管理

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Notifier服务地址
    #[serde(default = "Config::default_notifier_host")]
    pub notifier_host: String,
    
    /// HTTP超时时间(毫秒)
    #[serde(default = "Config::default_timeout_ms")]
    pub timeout_ms: u64,
}

impl Config {
    fn default_notifier_host() -> String {
        "http://127.0.0.1:4693".to_string()
    }
    
    fn default_timeout_ms() -> u64 {
        100
    }
    
    /// 获取配置文件路径
    fn config_path() -> PathBuf {
        // 配置文件放在DLL同目录下
        let mut path = std::env::current_exe().unwrap_or_default();
        path.pop();
        path.push("guga_ura_config.json");
        path
    }
    
    /// 加载配置
    pub fn load() -> Config {
        let path = Self::config_path();
        
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            warn!("Failed to parse config: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read config: {}", e);
                }
            }
        }
        
        // 使用默认配置并保存
        let config = Config::default();
        let _ = config.save();
        config
    }
    
    /// 保存配置
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialize error: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            notifier_host: Self::default_notifier_host(),
            timeout_ms: Self::default_timeout_ms(),
        }
    }
}

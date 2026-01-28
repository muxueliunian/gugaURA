//! 核心模块

mod config;
mod interceptor;
mod http;
// 注意：反检测功能已移至独立的 Cellar (apphelp.dll)

pub use config::Config;
pub use interceptor::Interceptor;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use once_cell::sync::OnceCell;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, LoadLibraryW};

use crate::proxy;
use crate::il2cpp;

static INSTANCE: OnceCell<Arc<GugaURA>> = OnceCell::new();
static HOOKING_FINISHED: AtomicBool = AtomicBool::new(false);

/// GugaURA核心结构
pub struct GugaURA {
    pub config: Config,
    pub interceptor: Interceptor,
}

impl GugaURA {
    pub fn init() -> Result<(), String> {
        let config = Config::load();
        info!("Config loaded: notifier_host = {}", config.notifier_host);
        
        let instance = Arc::new(GugaURA {
            config,
            interceptor: Interceptor::new(),
        });
        
        INSTANCE.set(instance).map_err(|_| "Already initialized")?;
        
        // 初始化代理和Hook
        Self::setup_hooks()?;
        
        Ok(())
    }
    
    pub fn instance() -> Arc<GugaURA> {
        INSTANCE.get().expect("GugaURA not initialized").clone()
    }
    
    fn setup_hooks() -> Result<(), String> {
        let instance = Self::instance();
        
        // 检查是否已经加载了 GameAssembly.dll (游戏可能已经启动)
        let game_assembly = unsafe { GetModuleHandleW(w!("GameAssembly.dll")) };
        
        if let Ok(handle) = game_assembly {
            if !handle.is_invalid() {
                info!("Late loading detected, GameAssembly already loaded");
                il2cpp::set_handle(handle.0 as usize);
                // 延迟初始化HTTP hooks
                Self::try_init_http_hooks();
                return Ok(());
            }
        }
        
        // 正常流程：代理 UnityPlayer.dll 并 Hook LoadLibraryW
        info!("Setting up UnityPlayer proxy");
        proxy::unityplayer::init()?;
        
        info!("Hooking LoadLibraryW");
        instance.interceptor.hook_load_library()?;
        
        Ok(())
    }
    
    /// 当 GameAssembly.dll 加载后调用
    pub fn on_game_assembly_loaded(handle: usize) {
        info!("GameAssembly.dll loaded at 0x{:X}", handle);
        il2cpp::set_handle(handle);
    }
    
    /// 当CriWare库加载时调用（表示游戏初始化完成）
    pub fn on_game_ready() {
        // 🔑 防止重复初始化
        if HOOKING_FINISHED.swap(true, Ordering::Relaxed) {
            // 已经初始化过了
            return;
        }
        
        info!("Game ready, initializing HTTP hooks (first time only)");
        Self::try_init_http_hooks();
    }
    
    /// 尝试初始化HTTP hooks
    fn try_init_http_hooks() {
        // 初始化IL2CPP符号
        il2cpp::init();
        
        // 添加延迟，让IL2CPP完全初始化
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Hook HTTP请求/响应
        if let Err(e) = il2cpp::http_hook::init() {
            error!("Failed to hook HTTP: {}", e);
            // 标记为未完成，以便下次重试
            HOOKING_FINISHED.store(false, Ordering::Relaxed);
        } else {
            info!("HTTP hooks installed successfully!");
        }
    }
    
    pub fn cleanup() {
        if let Some(instance) = INSTANCE.get() {
            instance.interceptor.unhook_all();
        }
    }
}

/// 发送数据到notifier服务
pub fn notify_request(data: &[u8]) {
    let config = &GugaURA::instance().config;
    let url = format!("{}/notify/request", config.notifier_host);
    info!("Sending request data ({} bytes) to {}", data.len(), url);
    http::post_bytes(&url, data, config.timeout_ms);
}

pub fn notify_response(data: &[u8]) {
    let config = &GugaURA::instance().config;
    let url = format!("{}/notify/response", config.notifier_host);
    info!("Sending response data ({} bytes) to {}", data.len(), url);
    http::post_bytes(&url, data, config.timeout_ms);
}

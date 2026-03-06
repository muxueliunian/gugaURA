//! 核心模块

mod config;
pub mod debug;
mod http;
mod interceptor;
mod watcher;
// 注意：反检测功能已移至独立的 Cellar (apphelp.dll)

pub use config::Config;
pub use interceptor::Interceptor;

use arc_swap::ArcSwap;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use windows::core::w;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

use crate::il2cpp;
use crate::proxy;

static INSTANCE: OnceCell<Arc<GugaURA>> = OnceCell::new();
static HOOKING_FINISHED: AtomicBool = AtomicBool::new(false);
static DEBUG_MODE_STATE: AtomicU8 = AtomicU8::new(2);
static DEBUG_MODE_DISABLED_HINT_LOGGED: AtomicBool = AtomicBool::new(false);

fn log_debug_mode_state(debug_mode: bool) {
    let next = if debug_mode { 1 } else { 0 };
    let prev = DEBUG_MODE_STATE.swap(next, Ordering::Relaxed);
    if prev != next {
        info!(
            "Debug mode state changed: {}",
            if debug_mode { "ENABLED" } else { "DISABLED" }
        );
    }
}

/// GugaURA核心结构
pub struct GugaURA {
    pub config: ArcSwap<Config>,
    pub interceptor: Interceptor,
}

impl GugaURA {
    pub fn init() -> Result<(), String> {
        let config = Config::load();
        let config_path = Config::config_path();
        info!(
            "Config loaded: path = {}, notifier_host = {}, timeout_ms = {}, target_fps = {}, vsync_count = {}, debug_mode = {}, debug_output_dir = {:?}",
            config_path.display(),
            config.notifier_host,
            config.timeout_ms,
            config.target_fps,
            config.vsync_count,
            config.debug_mode,
            config.debug_output_dir
        );
        log_debug_mode_state(config.debug_mode);

        let instance = Arc::new(GugaURA {
            config: ArcSwap::new(Arc::new(config)),
            interceptor: Interceptor::new(),
        });

        INSTANCE.set(instance).map_err(|_| "Already initialized")?;

        // 初始化代理和Hook
        Self::setup_hooks()?;

        // 启动配置文件监控
        watcher::start_config_watcher();

        Ok(())
    }

    pub fn instance() -> Arc<GugaURA> {
        INSTANCE.get().expect("GugaURA not initialized").clone()
    }

    /// 重载配置
    /// 从文件重新加载配置，并应用 FPS/VSync 等运行时设置
    pub fn reload_config() {
        let instance = Self::instance();
        let new_config = match Config::try_load() {
            Ok(config) => config,
            Err(e) => {
                warn!("Config reload skipped: {}", e);
                return;
            }
        };

        let config_path = Config::config_path();
        info!(
            "Config reloaded: path = {}, notifier_host = {}, timeout_ms = {}, target_fps = {}, vsync_count = {}, debug_mode = {}, debug_output_dir = {:?}, fans_enabled = {}, fans_output_dir = {:?}",
            config_path.display(),
            new_config.notifier_host,
            new_config.timeout_ms,
            new_config.target_fps,
            new_config.vsync_count,
            new_config.debug_mode,
            new_config.debug_output_dir,
            new_config.fans_enabled,
            new_config.fans_output_dir
        );
        log_debug_mode_state(new_config.debug_mode);

        // 更新 FPS/VSync 原子变量（即使 hook 未初始化也安全）
        il2cpp::fps_hook::TARGET_FPS.store(new_config.target_fps, Ordering::Relaxed);
        il2cpp::fps_hook::VSYNC_COUNT.store(new_config.vsync_count, Ordering::Relaxed);

        // 原子交换配置
        instance.config.store(Arc::new(new_config));

        info!("Config reloaded successfully");
    }

    fn setup_hooks() -> Result<(), String> {
        let instance = Self::instance();

        // 检查是否已经加载了 GameAssembly.dll (游戏可能已经启动)
        let game_assembly = unsafe { GetModuleHandleW(w!("GameAssembly.dll")) };

        if let Ok(handle) = game_assembly {
            if !handle.is_invalid() {
                info!("Late loading detected, GameAssembly already loaded");
                il2cpp::set_handle(handle.0 as usize);

                // Steam 版晚加载：初始化 cri_mana_vpx 代理
                info!("Init cri_mana_vpx proxy (late loading)");
                if let Err(e) = proxy::cri_mana_vpx::init() {
                    warn!("cri_mana_vpx proxy init failed: {}", e);
                }

                // 延迟初始化HTTP hooks
                Self::try_init_http_hooks();
                return Ok(());
            }
        }

        // 正常流程：判断是 Steam 版还是 DMM 版
        let is_steam = Self::is_steam_release();
        info!("Game version: {}", if is_steam { "Steam" } else { "DMM" });

        if is_steam {
            // Steam 版：我们的 DLL 替换了 cri_mana_vpx.dll，需要初始化代理
            info!("Setting up cri_mana_vpx proxy (Steam)");
            proxy::cri_mana_vpx::init()?;
        } else {
            // DMM 版：代理 UnityPlayer.dll
            info!("Setting up UnityPlayer proxy (DMM)");
            proxy::unityplayer::init()?;
        }

        info!("Hooking LoadLibraryW");
        instance.interceptor.hook_load_library()?;

        Ok(())
    }

    /// 判断是否是 Steam 版
    /// 只有日本 Steam 版使用 umamusumeprettyderby_jpn.exe
    fn is_steam_release() -> bool {
        let exec_path = std::env::current_exe().unwrap_or_default();
        let file_name = exec_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // 只有日本 Steam 版使用这个可执行文件名
        file_name.eq_ignore_ascii_case("umamusumeprettyderby_jpn")
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
        let instance = Self::instance();
        let config = instance.config.load();

        // 初始化IL2CPP符号
        il2cpp::init();

        // 添加延迟，让IL2CPP完全初始化
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 初始化帧数限制Hook
        il2cpp::fps_hook::init(config.target_fps, config.vsync_count);

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
    let config = GugaURA::instance().config.load();
    log_debug_mode_state(config.debug_mode);

    // Debug 模式：保存 msgpack 数据为 JSON
    if config.debug_mode {
        debug::save_msgpack_as_json(data, "request", config.debug_output_dir.as_deref());
    } else if !DEBUG_MODE_DISABLED_HINT_LOGGED.swap(true, Ordering::Relaxed) {
        warn!(
            "Debug mode is DISABLED while request interception is active. \
            If you enabled it in Config UI, click '保存配置' (or use auto-save toggle) and check logs for loaded config path."
        );
    }

    let url = format!("{}/notify/request", config.notifier_host);
    info!("Sending request data ({} bytes) to {}", data.len(), url);
    http::post_bytes(&url, data, config.timeout_ms);
}

pub fn notify_response(data: &[u8]) {
    let config = GugaURA::instance().config.load();
    log_debug_mode_state(config.debug_mode);

    // Debug 模式：保存 msgpack 数据为 JSON
    if config.debug_mode {
        debug::save_msgpack_as_json(data, "response", config.debug_output_dir.as_deref());
    } else if !DEBUG_MODE_DISABLED_HINT_LOGGED.swap(true, Ordering::Relaxed) {
        warn!(
            "Debug mode is DISABLED while response interception is active. \
            If you enabled it in Config UI, click '保存配置' (or use auto-save toggle) and check logs for loaded config path."
        );
    }

    let url = format!("{}/notify/response", config.notifier_host);
    info!("Sending response data ({} bytes) to {}", data.len(), url);
    http::post_bytes(&url, data, config.timeout_ms);
}

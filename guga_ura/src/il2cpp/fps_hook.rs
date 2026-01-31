//! 帧数限制Hook模块
//! 
//! 通过Hook Unity的 Application.set_targetFrameRate 和 QualitySettings.set_vSyncCount
//! 来实现自定义帧数限制

use std::os::raw::c_void;
use std::sync::atomic::{AtomicI32, Ordering};
use minhook::MinHook;

use crate::il2cpp::symbols::il2cpp_resolve_icall;

/// 目标帧数 (-1 表示使用游戏默认)
pub static TARGET_FPS: AtomicI32 = AtomicI32::new(-1);

/// VSync设置 (-1 表示使用游戏默认)
pub static VSYNC_COUNT: AtomicI32 = AtomicI32::new(-1);

// 原始函数指针
static mut SET_TARGET_FRAME_RATE_ORIG: usize = 0;
static mut SET_VSYNC_COUNT_ORIG: usize = 0;

// 函数类型定义
type SetTargetFrameRateFn = extern "C" fn(i32);
type SetVSyncCountFn = extern "C" fn(i32);

/// Hooked set_targetFrameRate
/// 动态检查 TARGET_FPS 值，如果不是 -1 则覆盖游戏设置
extern "C" fn hooked_set_target_frame_rate(mut value: i32) {
    let target = TARGET_FPS.load(Ordering::Relaxed);
    if target != -1 {
        info!("Overriding targetFrameRate: {} -> {}", value, target);
        value = target;
    }
    
    // 调用原始函数
    unsafe {
        let orig_fn: SetTargetFrameRateFn = std::mem::transmute(SET_TARGET_FRAME_RATE_ORIG);
        orig_fn(value);
    }
}

/// Hooked set_vSyncCount
/// 动态检查 VSYNC_COUNT 值，如果不是 -1 则覆盖游戏设置
extern "C" fn hooked_set_vsync_count(mut value: i32) {
    let target = VSYNC_COUNT.load(Ordering::Relaxed);
    if target != -1 {
        info!("Overriding vSyncCount: {} -> {}", value, target);
        value = target;
    }
    
    // 调用原始函数
    unsafe {
        let orig_fn: SetVSyncCountFn = std::mem::transmute(SET_VSYNC_COUNT_ORIG);
        orig_fn(value);
    }
}

/// 初始化帧数Hook
/// 
/// 注意：无论配置值是什么，都会安装 Hook。
/// 这样可以确保即使初始配置是默认值 (-1)，Hook 也会被安装。
/// 在 Hook 回调中会动态检查配置值，决定是否覆盖游戏设置。
pub fn init(target_fps: i32, vsync_count: i32) {
    info!("Initializing FPS hooks...");
    info!("  Target FPS: {}", if target_fps == -1 { "default".to_string() } else { target_fps.to_string() });
    info!("  VSync: {}", if vsync_count == -1 { "default".to_string() } else { vsync_count.to_string() });
    
    // 设置配置值到静态变量
    TARGET_FPS.store(target_fps, Ordering::Relaxed);
    VSYNC_COUNT.store(vsync_count, Ordering::Relaxed);
    
    unsafe {
        // 总是尝试 Hook set_targetFrameRate
        if let Some(addr) = il2cpp_resolve_icall("UnityEngine.Application::set_targetFrameRate(System.Int32)") {
            match MinHook::create_hook(
                addr as *mut c_void,
                hooked_set_target_frame_rate as *mut c_void
            ) {
                Ok(orig) => {
                    SET_TARGET_FRAME_RATE_ORIG = orig as usize;
                    
                    if let Err(e) = MinHook::enable_hook(addr as *mut c_void) {
                        error!("Failed to enable set_targetFrameRate hook: {:?}", e);
                    } else {
                        info!("set_targetFrameRate hooked successfully at 0x{:X}", addr);
                    }
                }
                Err(e) => {
                    error!("Failed to create set_targetFrameRate hook: {:?}", e);
                }
            }
        } else {
            warn!("Could not resolve set_targetFrameRate icall");
        }
        
        // 总是尝试 Hook set_vSyncCount
        if let Some(addr) = il2cpp_resolve_icall("UnityEngine.QualitySettings::set_vSyncCount(System.Int32)") {
            match MinHook::create_hook(
                addr as *mut c_void,
                hooked_set_vsync_count as *mut c_void
            ) {
                Ok(orig) => {
                    SET_VSYNC_COUNT_ORIG = orig as usize;
                    
                    if let Err(e) = MinHook::enable_hook(addr as *mut c_void) {
                        error!("Failed to enable set_vSyncCount hook: {:?}", e);
                    } else {
                        info!("set_vSyncCount hooked successfully at 0x{:X}", addr);
                    }
                }
                Err(e) => {
                    error!("Failed to create set_vSyncCount hook: {:?}", e);
                }
            }
        } else {
            warn!("Could not resolve set_vSyncCount icall");
        }
    }
    
    info!("FPS hooks initialization complete");
}

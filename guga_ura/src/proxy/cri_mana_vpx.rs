//! cri_mana_vpx.dll 代理（Steam 版使用）
//!
//! Steam 版游戏使用 cri_mana_vpx.dll 作为视频解码库，
//! 我们替换这个 DLL 来实现注入，原始 DLL 保存到数据目录。

#![allow(non_snake_case, non_upper_case_globals)]

use std::path::PathBuf;
use widestring::U16CString;
use windows::core::PCWSTR;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

use crate::proxy_proc;
use crate::trace;

// 声明代理函数 - cri_mana_vpx.dll 导出的函数
proxy_proc!(criVvp9_GetAlphaInterface, criVvp9_GetAlphaInterface_orig);
proxy_proc!(criVvp9_GetInterface, criVvp9_GetInterface_orig);
proxy_proc!(criVvp9_SetUserAllocator, criVvp9_SetUserAllocator_orig);

/// 获取游戏目录
fn get_game_dir() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(&PathBuf::new())
        .to_path_buf()
}

/// 获取数据目录
fn get_data_dir() -> PathBuf {
    let mut path = get_game_dir();
    path.push("guga_ura_data");
    path
}

/// 初始化 cri_mana_vpx 代理
/// 从数据目录加载原始 DLL 并获取函数地址
pub fn init() -> Result<(), String> {
    let data_dir = get_data_dir();
    // 使用 _orig 后缀与 installer.rs 保持一致
    let dll_path = data_dir.join("cri_mana_vpx_orig.dll");
    trace::append_runtime_log(&format!(
        "[cri_proxy] init data_dir={} dll_path={}",
        data_dir.display(),
        dll_path.display()
    ));

    if !dll_path.exists() {
        // 原始 DLL 不存在，跳过（可能是 DMM 版或未正确安装）
        warn!("cri_mana_vpx_orig.dll not found in data dir, skipping cri proxy init");
        trace::append_runtime_log(
            "[cri_proxy] cri_mana_vpx_orig.dll not found, skipping proxy init",
        );
        return Ok(());
    }

    unsafe {
        let dll_path_str = dll_path.to_str().ok_or("Invalid path")?;
        let dll_path_wide = U16CString::from_str(dll_path_str)
            .map_err(|e| format!("Path encoding error: {}", e))?;

        let handle = LoadLibraryW(PCWSTR(dll_path_wide.as_ptr()))
            .map_err(|e| format!("Failed to load cri_mana_vpx.dll: {}", e))?;
        trace::append_runtime_log(&format!(
            "[cri_proxy] LoadLibraryW success handle=0x{:X}",
            handle.0 as usize
        ));

        let mut missing_exports = Vec::new();

        // 获取函数地址
        if let Some(addr) = GetProcAddress(handle, windows::core::s!("criVvp9_GetAlphaInterface")) {
            criVvp9_GetAlphaInterface_orig = addr as usize;
        } else {
            missing_exports.push("criVvp9_GetAlphaInterface");
        }

        if let Some(addr) = GetProcAddress(handle, windows::core::s!("criVvp9_GetInterface")) {
            criVvp9_GetInterface_orig = addr as usize;
        } else {
            missing_exports.push("criVvp9_GetInterface");
        }

        if let Some(addr) = GetProcAddress(handle, windows::core::s!("criVvp9_SetUserAllocator")) {
            criVvp9_SetUserAllocator_orig = addr as usize;
        } else {
            missing_exports.push("criVvp9_SetUserAllocator");
        }

        if !missing_exports.is_empty() {
            let error = format!(
                "cri_mana_vpx_orig.dll 缺少导出: {}",
                missing_exports.join(", ")
            );
            trace::append_runtime_log(&format!("[cri_proxy] {}", error));
            return Err(error);
        }

        info!("cri_mana_vpx proxy initialized");
        info!(
            "  criVvp9_GetAlphaInterface: 0x{:X}",
            criVvp9_GetAlphaInterface_orig
        );
        info!("  criVvp9_GetInterface: 0x{:X}", criVvp9_GetInterface_orig);
        info!(
            "  criVvp9_SetUserAllocator: 0x{:X}",
            criVvp9_SetUserAllocator_orig
        );
        trace::append_runtime_log(&format!(
            "[cri_proxy] exports resolved alpha=0x{:X} interface=0x{:X} allocator=0x{:X}",
            criVvp9_GetAlphaInterface_orig,
            criVvp9_GetInterface_orig,
            criVvp9_SetUserAllocator_orig
        ));
    }

    Ok(())
}

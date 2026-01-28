//! UnityPlayer.dll 代理
//! 
//! 游戏会加载 UnityPlayer.dll，我们替换它然后转发到原始DLL

#![allow(non_snake_case, non_upper_case_globals)]

use std::path::PathBuf;
use widestring::U16CString;
use windows::core::PCWSTR;
use windows::Win32::System::LibraryLoader::{LoadLibraryW, GetProcAddress};
use windows::Win32::Foundation::HMODULE;

use crate::proxy_proc;

// 声明代理函数 - UnityPlayer.dll只需要代理UnityMain
proxy_proc!(UnityMain, UnityMain_orig);

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

/// 准备原始DLL
/// 将原始的 UnityPlayer.dll 复制到数据目录
fn prepare_orig_dll() -> Result<PathBuf, String> {
    let src_dll = get_game_dir().join("UnityPlayer.dll");
    let data_dir = get_data_dir();
    let dest_dll = data_dir.join("UnityPlayer_orig.dll");
    
    // 确保数据目录存在
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data dir: {}", e))?;
    
    // 检查是否需要更新
    if dest_dll.exists() {
        let src_time = std::fs::metadata(&src_dll)
            .and_then(|m| m.modified())
            .ok();
        let dest_time = std::fs::metadata(&dest_dll)
            .and_then(|m| m.modified())
            .ok();
        
        if let (Some(s), Some(d)) = (src_time, dest_time) {
            if d >= s {
                // 已经是最新的
                return Ok(dest_dll);
            }
        }
    }
    
    // 复制文件
    std::fs::copy(&src_dll, &dest_dll)
        .map_err(|e| format!("Failed to copy UnityPlayer.dll: {}", e))?;
    
    Ok(dest_dll)
}

/// 初始化 UnityPlayer 代理
pub fn init() -> Result<(), String> {
    unsafe {
        let dll_path = prepare_orig_dll()?;
        
        let dll_path_str = dll_path.to_str()
            .ok_or("Invalid path")?;
        let dll_path_wide = U16CString::from_str(dll_path_str)
            .map_err(|e| format!("Path encoding error: {}", e))?;
        
        let handle = LoadLibraryW(PCWSTR(dll_path_wide.as_ptr()))
            .map_err(|e| format!("Failed to load UnityPlayer_orig.dll: {}", e))?;
        
        // 获取 UnityMain 地址
        let unity_main = GetProcAddress(handle, windows::core::s!("UnityMain"))
            .ok_or("Failed to get UnityMain address")?;
        
        UnityMain_orig = unity_main as usize;
        
        info!("UnityPlayer proxy initialized, UnityMain at 0x{:X}", UnityMain_orig);
        
        Ok(())
    }
}

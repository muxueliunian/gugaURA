//! Hook拦截器模块 - 基于MinHook

use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::Mutex;
use minhook::MinHook;
use once_cell::sync::Lazy;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::LoadLibraryW as WinLoadLibraryW;

use crate::core::GugaURA;

/// Hook记录
struct HookEntry {
    orig_addr: usize,
    trampoline_addr: usize,
}

/// 拦截器
pub struct Interceptor {
    hooks: Mutex<HashMap<usize, HookEntry>>,
}

impl Interceptor {
    pub fn new() -> Self {
        Interceptor {
            hooks: Mutex::new(HashMap::new()),
        }
    }
    
    /// 创建Hook
    pub fn hook(&self, orig_addr: usize, hook_addr: usize) -> Result<usize, String> {
        unsafe {
            let trampoline = MinHook::create_hook(
                orig_addr as *mut c_void,
                hook_addr as *mut c_void
            ).map_err(|e| format!("MinHook create error: {:?}", e))? as usize;
            
            MinHook::enable_hook(orig_addr as *mut c_void)
                .map_err(|e| format!("MinHook enable error: {:?}", e))?;
            
            self.hooks.lock().unwrap().insert(orig_addr, HookEntry {
                orig_addr,
                trampoline_addr: trampoline,
            });
            
            Ok(trampoline)
        }
    }
    
    /// 获取trampoline地址
    pub fn get_trampoline(&self, orig_addr: usize) -> Option<usize> {
        self.hooks.lock().unwrap().get(&orig_addr).map(|e| e.trampoline_addr)
    }
    
    /// 移除所有Hook
    pub fn unhook_all(&self) {
        let hooks = self.hooks.lock().unwrap();
        for (addr, _) in hooks.iter() {
            unsafe {
                let _ = MinHook::disable_hook(*addr as *mut c_void);
                let _ = MinHook::remove_hook(*addr as *mut c_void);
            }
        }
    }
    
    /// Hook LoadLibraryW
    pub fn hook_load_library(&self) -> Result<(), String> {
        // 获取kernel32.dll中的LoadLibraryW地址
        let kernel32 = unsafe { 
            windows::Win32::System::LibraryLoader::GetModuleHandleW(w!("kernel32.dll"))
                .map_err(|e| format!("GetModuleHandleW failed: {}", e))?
        };
        
        let load_library_addr = unsafe {
            windows::Win32::System::LibraryLoader::GetProcAddress(kernel32, windows::core::s!("LoadLibraryW"))
                .ok_or("GetProcAddress failed")?
        };
        
        self.hook(load_library_addr as usize, hooked_load_library_w as usize)?;
        
        Ok(())
    }
}

// 记录原始LoadLibraryW的trampoline
static mut LOAD_LIBRARY_TRAMPOLINE: usize = 0;

type LoadLibraryWFn = extern "C" fn(PCWSTR) -> HMODULE;

/// Hook后的LoadLibraryW
extern "C" fn hooked_load_library_w(filename: PCWSTR) -> HMODULE {
    // 调用原始函数
    let orig_fn: LoadLibraryWFn = unsafe {
        let trampoline = GugaURA::instance().interceptor.get_trampoline(
            windows::Win32::System::LibraryLoader::GetProcAddress(
                windows::Win32::System::LibraryLoader::GetModuleHandleW(w!("kernel32.dll")).unwrap(),
                windows::core::s!("LoadLibraryW")
            ).unwrap() as usize
        ).unwrap_or(0);
        std::mem::transmute(trampoline)
    };
    
    let handle = orig_fn(filename);
    
    // 检查是否是我们关心的DLL
    let filename_str = unsafe { 
        filename.to_string().unwrap_or_default() 
    };
    
    // 检查是否是 criware 库（表示游戏初始化完成）
    if filename_str.to_lowercase().contains("cri_ware_unity")
        || filename_str.to_lowercase().contains("cri_mana_vpx") 
    {
        info!("CriWare library loaded, game ready");
        
        // 尝试获取GameAssembly
        let ga_handle = unsafe {
            WinLoadLibraryW(w!("GameAssembly.dll"))
        };
        
        if let Ok(h) = ga_handle {
            if !h.is_invalid() {
                GugaURA::on_game_assembly_loaded(h.0 as usize);
            }
        }
        
        GugaURA::on_game_ready();
    }
    
    handle
}

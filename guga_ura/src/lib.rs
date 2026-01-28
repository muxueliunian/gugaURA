//! GugaURA - Uma Musume Data Capture Tool
//! 
//! 精简版数据抓包工具，只做一件事：
//! 拦截游戏HTTP请求/响应，转发到本地分析服务

#![allow(non_snake_case)]

#[macro_use]
extern crate log;

mod core;
mod proxy;
mod il2cpp;

use std::os::raw::{c_ulong, c_void};
use windows::Win32::Foundation::{HMODULE, TRUE};

use crate::core::GugaURA;

const DLL_PROCESS_ATTACH: c_ulong = 1;
const DLL_PROCESS_DETACH: c_ulong = 0;

pub static mut DLL_HMODULE: HMODULE = HMODULE(std::ptr::null_mut());

/// DLL入口点
#[no_mangle]
pub extern "C" fn DllMain(hmodule: HMODULE, call_reason: c_ulong, _reserved: *mut c_void) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        unsafe { DLL_HMODULE = hmodule; }
        
        // 初始化日志 - Release模式也启用以便调试
        windebug_logger::init();
        
        info!("GugaURA v{} loading...", env!("CARGO_PKG_VERSION"));
        
        // 初始化核心
        if let Err(e) = GugaURA::init() {
            error!("Init failed: {}", e);
            return TRUE.into();
        }
        
        info!("GugaURA attached successfully");
    }
    else if call_reason == DLL_PROCESS_DETACH {
        info!("GugaURA detaching...");
        GugaURA::cleanup();
    }
    
    TRUE.into()
}

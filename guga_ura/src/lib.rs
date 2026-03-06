//! GugaURA - Uma Musume Data Capture Tool
//!
//! 精简版数据抓包工具，只做一件事：
//! 拦截游戏HTTP请求/响应，转发到本地分析服务

#![allow(non_snake_case)]

#[macro_use]
extern crate log;

mod core;
mod il2cpp;
mod proxy;

use std::os::raw::{c_ulong, c_void};
use windows::core::PCSTR;
use windows::Win32::Foundation::{HMODULE, TRUE};
use windows::Win32::System::Diagnostics::Debug::OutputDebugStringA;

use crate::core::GugaURA;

const DLL_PROCESS_ATTACH: c_ulong = 1;
const DLL_PROCESS_DETACH: c_ulong = 0;

pub static mut DLL_HMODULE: HMODULE = HMODULE(std::ptr::null_mut());

fn raw_debug_output(msg: &str) {
    let mut bytes: Vec<u8> = msg.as_bytes().iter().copied().filter(|b| *b != 0).collect();
    bytes.push(b'\n');
    bytes.push(0);
    unsafe {
        OutputDebugStringA(PCSTR(bytes.as_ptr()));
    }
}

/// DLL入口点
#[no_mangle]
pub extern "C" fn DllMain(hmodule: HMODULE, call_reason: c_ulong, _reserved: *mut c_void) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        unsafe {
            DLL_HMODULE = hmodule;
        }
        raw_debug_output("[GugaURA] DllMain PROCESS_ATTACH");

        // 初始化日志 - Release模式也启用以便调试
        if let Err(e) = windebug_logger::init() {
            raw_debug_output(&format!("[GugaURA] windebug_logger init failed: {}", e));
        } else {
            raw_debug_output("[GugaURA] windebug_logger initialized");
        }

        info!("GugaURA v{} loading...", env!("CARGO_PKG_VERSION"));
        raw_debug_output(&format!(
            "[GugaURA] loading version {}",
            env!("CARGO_PKG_VERSION")
        ));

        // 初始化核心
        if let Err(e) = GugaURA::init() {
            error!("Init failed: {}", e);
            raw_debug_output(&format!("[GugaURA] init failed: {}", e));
            return TRUE.into();
        }

        info!("GugaURA attached successfully");
        raw_debug_output("[GugaURA] attached successfully");
    } else if call_reason == DLL_PROCESS_DETACH {
        info!("GugaURA detaching...");
        raw_debug_output("[GugaURA] DllMain PROCESS_DETACH");
        GugaURA::cleanup();
    }

    TRUE.into()
}

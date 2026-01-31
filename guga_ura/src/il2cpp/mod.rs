//! IL2CPP交互模块

pub mod types;
mod symbols_impl;
pub mod symbols;
pub mod http_hook;
pub mod fps_hook;

use std::ptr::null_mut;
use std::os::raw::c_void;

static mut IL2CPP_HANDLE: *mut c_void = null_mut();

/// 设置IL2CPP模块句柄
pub fn set_handle(handle: usize) {
    unsafe {
        IL2CPP_HANDLE = handle as *mut c_void;
    }
    info!("IL2CPP handle set to 0x{:X}", handle);
}

/// 获取IL2CPP模块句柄
pub fn get_handle() -> *mut c_void {
    unsafe { IL2CPP_HANDLE }
}

/// 初始化IL2CPP符号
pub fn init() {
    symbols::init();
}

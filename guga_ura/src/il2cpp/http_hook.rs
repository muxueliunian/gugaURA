//! HTTP Hook模块
//! 
//! 核心功能：拦截游戏的HTTP请求和响应
//! Hook Gallop.HttpHelper 的 CompressRequest 和 DecompressResponse 方法

use std::os::raw::c_void;
use minhook::MinHook;

use super::types::Il2CppArray;
use super::symbols;
use crate::core;

// 原始函数指针
static mut COMPRESS_REQUEST_ORIG: usize = 0;
static mut DECOMPRESS_RESPONSE_ORIG: usize = 0;

// 函数类型定义
type CompressRequestFn = extern "C" fn(*mut Il2CppArray) -> *mut Il2CppArray;
type DecompressResponseFn = extern "C" fn(*mut Il2CppArray) -> *mut Il2CppArray;

/// Hook: CompressRequest
/// 拦截游戏发送的请求数据（压缩前的明文msgpack）
extern "C" fn hooked_compress_request(data: *mut Il2CppArray) -> *mut Il2CppArray {
    // 先获取数据
    if !data.is_null() {
        unsafe {
            let array = &*data;
            let slice = array.as_slice::<u8>();
            
            // 转发到notifier服务
            core::notify_request(slice);
        }
    }
    
    // 调用原始函数
    unsafe {
        let orig_fn: CompressRequestFn = std::mem::transmute(COMPRESS_REQUEST_ORIG);
        orig_fn(data)
    }
}

/// Hook: DecompressResponse
/// 拦截游戏收到的响应数据（解压后的明文msgpack）
extern "C" fn hooked_decompress_response(data: *mut Il2CppArray) -> *mut Il2CppArray {
    // 先调用原始函数获取解压后的数据
    let decompressed = unsafe {
        let orig_fn: DecompressResponseFn = std::mem::transmute(DECOMPRESS_RESPONSE_ORIG);
        orig_fn(data)
    };
    
    // 转发解压后的数据
    if !decompressed.is_null() {
        unsafe {
            let array = &*decompressed;
            let slice = array.as_slice::<u8>();
            
            // 转发到notifier服务
            core::notify_response(slice);
        }
    }
    
    decompressed
}

/// 初始化HTTP Hook
pub fn init() -> Result<(), String> {
    // 获取 umamusume.dll 程序集 (HttpHelper类在Gallop命名空间中)
    let image = symbols::get_assembly_image("umamusume.dll")
        .ok_or("Failed to find umamusume assembly")?;
    
    info!("Found umamusume assembly");
    
    // 获取 HttpHelper 类
    let http_helper = symbols::get_class(image, "Gallop", "HttpHelper")
        .ok_or("Failed to find HttpHelper class")?;
    
    info!("Found HttpHelper class");
    
    // 获取 CompressRequest 方法
    let compress_request_addr = symbols::get_method_addr(http_helper, "CompressRequest", 1)
        .ok_or("Failed to find CompressRequest method")?;
    
    info!("Found CompressRequest at 0x{:X}", compress_request_addr);
    
    // 获取 DecompressResponse 方法
    let decompress_response_addr = symbols::get_method_addr(http_helper, "DecompressResponse", 1)
        .ok_or("Failed to find DecompressResponse method")?;
    
    info!("Found DecompressResponse at 0x{:X}", decompress_response_addr);
    
    // 安装 Hook
    unsafe {
        // Hook CompressRequest
        COMPRESS_REQUEST_ORIG = MinHook::create_hook(
            compress_request_addr as *mut c_void,
            hooked_compress_request as *mut c_void
        ).map_err(|e| format!("Failed to create CompressRequest hook: {:?}", e))? as usize;
        
        MinHook::enable_hook(compress_request_addr as *mut c_void)
            .map_err(|e| format!("Failed to enable CompressRequest hook: {:?}", e))?;
        
        info!("CompressRequest hooked");
        
        // Hook DecompressResponse
        DECOMPRESS_RESPONSE_ORIG = MinHook::create_hook(
            decompress_response_addr as *mut c_void,
            hooked_decompress_response as *mut c_void
        ).map_err(|e| format!("Failed to create DecompressResponse hook: {:?}", e))? as usize;
        
        MinHook::enable_hook(decompress_response_addr as *mut c_void)
            .map_err(|e| format!("Failed to enable DecompressResponse hook: {:?}", e))?;
        
        info!("DecompressResponse hooked");
    }
    
    info!("HTTP hooks installed successfully!");
    
    Ok(())
}

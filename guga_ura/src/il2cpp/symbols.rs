//! IL2CPP符号解析模块

use std::ffi::CStr;
use std::os::raw::c_void;

use super::types::*;
use super::symbols_impl;

// IL2CPP API函数类型
type Il2cppDomainGet = extern "C" fn() -> *mut Il2CppDomain;
type Il2cppDomainGetAssemblies = extern "C" fn(domain: *mut Il2CppDomain, size: *mut usize) -> *mut *mut c_void;
type Il2cppAssemblyGetImage = extern "C" fn(assembly: *mut c_void) -> *const Il2CppImage;
type Il2cppImageGetName = extern "C" fn(image: *const Il2CppImage) -> *const i8;
type Il2cppClassFromName = extern "C" fn(image: *const Il2CppImage, namespace: *const i8, name: *const i8) -> *mut Il2CppClass;
type Il2cppClassGetMethodFromName = extern "C" fn(klass: *mut Il2CppClass, name: *const i8, args_count: i32) -> *const MethodInfo;

// IL2CPP API函数指针
static mut IL2CPP_DOMAIN_GET: Option<Il2cppDomainGet> = None;
static mut IL2CPP_DOMAIN_GET_ASSEMBLIES: Option<Il2cppDomainGetAssemblies> = None;
static mut IL2CPP_ASSEMBLY_GET_IMAGE: Option<Il2cppAssemblyGetImage> = None;
static mut IL2CPP_IMAGE_GET_NAME: Option<Il2cppImageGetName> = None;
static mut IL2CPP_CLASS_FROM_NAME: Option<Il2cppClassFromName> = None;
static mut IL2CPP_CLASS_GET_METHOD_FROM_NAME: Option<Il2cppClassGetMethodFromName> = None;

static mut DOMAIN: *mut Il2CppDomain = std::ptr::null_mut();

/// 从GameAssembly.dll获取导出函数（使用符号映射）
unsafe fn dlsym(name: &str) -> usize {
    let handle = super::get_handle();
    if handle.is_null() {
        error!("IL2CPP handle is null!");
        return 0;
    }
    
    // 使用符号映射获取地址
    symbols_impl::dlsym(handle, name)
}

/// 初始化IL2CPP API
pub fn init() {
    info!("Initializing IL2CPP symbols...");
    
    unsafe {
        let handle = super::get_handle();
        info!("IL2CPP handle: 0x{:X}", handle as usize);
        
        let domain_get_addr = dlsym("il2cpp_domain_get");
        info!("il2cpp_domain_get at 0x{:X}", domain_get_addr);
        
        if domain_get_addr == 0 {
            error!("Failed to get il2cpp_domain_get address! Symbol resolution failed.");
            return;
        }
        
        IL2CPP_DOMAIN_GET = Some(std::mem::transmute(domain_get_addr));
        
        let assemblies_addr = dlsym("il2cpp_domain_get_assemblies");
        info!("il2cpp_domain_get_assemblies at 0x{:X}", assemblies_addr);
        IL2CPP_DOMAIN_GET_ASSEMBLIES = Some(std::mem::transmute(assemblies_addr));
        
        let get_image_addr = dlsym("il2cpp_assembly_get_image");
        info!("il2cpp_assembly_get_image at 0x{:X}", get_image_addr);
        IL2CPP_ASSEMBLY_GET_IMAGE = Some(std::mem::transmute(get_image_addr));
        
        IL2CPP_IMAGE_GET_NAME = Some(std::mem::transmute(dlsym("il2cpp_image_get_name")));
        IL2CPP_CLASS_FROM_NAME = Some(std::mem::transmute(dlsym("il2cpp_class_from_name")));
        IL2CPP_CLASS_GET_METHOD_FROM_NAME = Some(std::mem::transmute(dlsym("il2cpp_class_get_method_from_name")));
        
        if let Some(domain_get) = IL2CPP_DOMAIN_GET {
            DOMAIN = domain_get();
            info!("IL2CPP domain: 0x{:X}", DOMAIN as usize);
            
            if DOMAIN.is_null() {
                error!("IL2CPP domain is NULL! Runtime may not be initialized yet.");
            }
        }
    }
}

/// 获取程序集镜像
pub fn get_assembly_image(assembly_name: &str) -> Option<*const Il2CppImage> {
    unsafe {
        let domain_get_assemblies = IL2CPP_DOMAIN_GET_ASSEMBLIES?;
        let assembly_get_image = IL2CPP_ASSEMBLY_GET_IMAGE?;
        let image_get_name = IL2CPP_IMAGE_GET_NAME?;
        
        if DOMAIN.is_null() {
            error!("Cannot get assemblies: DOMAIN is null!");
            return None;
        }
        
        let mut size: usize = 0;
        let assemblies = domain_get_assemblies(DOMAIN, &mut size);
        
        info!("Found {} assemblies in domain", size);
        
        if size == 0 {
            error!("No assemblies loaded! IL2CPP may not be ready.");
            return None;
        }
        
        for i in 0..size {
            let assembly = *assemblies.add(i);
            let image = assembly_get_image(assembly);
            let name_ptr = image_get_name(image);
            
            if !name_ptr.is_null() {
                let name = CStr::from_ptr(name_ptr).to_string_lossy();
                // 只记录少量以避免刷屏
                if i < 5 || name.contains("Gallop") {
                    info!("  Assembly[{}]: {}", i, name);
                }
                if name == assembly_name {
                    info!("Found target assembly: {}", assembly_name);
                    return Some(image);
                }
            }
        }
        
        error!("Assembly '{}' not found in {} loaded assemblies", assembly_name, size);
        None
    }
}

/// 获取类
pub fn get_class(image: *const Il2CppImage, namespace: &str, class_name: &str) -> Option<*mut Il2CppClass> {
    unsafe {
        let class_from_name = IL2CPP_CLASS_FROM_NAME?;
        
        let namespace_cstr = std::ffi::CString::new(namespace).ok()?;
        let name_cstr = std::ffi::CString::new(class_name).ok()?;
        
        let klass = class_from_name(image, namespace_cstr.as_ptr(), name_cstr.as_ptr());
        
        if klass.is_null() {
            None
        } else {
            Some(klass)
        }
    }
}

/// 获取方法地址
pub fn get_method_addr(klass: *mut Il2CppClass, name: &str, args_count: i32) -> Option<usize> {
    unsafe {
        let get_method = IL2CPP_CLASS_GET_METHOD_FROM_NAME?;
        
        let name_cstr = std::ffi::CString::new(name).ok()?;
        let method = get_method(klass, name_cstr.as_ptr(), args_count);
        
        if method.is_null() {
            None
        } else {
            Some((*method).method_ptr as usize)
        }
    }
}

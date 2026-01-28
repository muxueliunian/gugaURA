//! IL2CPP类型定义

use std::os::raw::c_void;

/// IL2CPP对象基类
#[repr(C)]
pub struct Il2CppObject {
    pub klass: *mut c_void,
    pub monitor: *mut c_void,
}

/// IL2CPP数组
#[repr(C)]
pub struct Il2CppArray {
    pub obj: Il2CppObject,
    pub bounds: *mut c_void,
    pub max_length: usize,
    // 后面紧跟着数组元素
}

impl Il2CppArray {
    /// 获取数组长度
    pub fn len(&self) -> usize {
        self.max_length
    }
    
    /// 获取数组数据指针
    pub fn data_ptr<T>(&self) -> *const T {
        unsafe {
            let base = self as *const Il2CppArray as *const u8;
            // 数组数据紧跟在Il2CppArray结构之后
            base.add(std::mem::size_of::<Il2CppArray>()) as *const T
        }
    }
    
    /// 转换为切片
    pub unsafe fn as_slice<T>(&self) -> &[T] {
        std::slice::from_raw_parts(self.data_ptr(), self.len())
    }
}

/// IL2CPP类
#[repr(C)]
pub struct Il2CppClass {
    _data: [u8; 0], // 不透明类型
}

/// IL2CPP镜像
#[repr(C)]
pub struct Il2CppImage {
    _data: [u8; 0],
}

/// IL2CPP域
#[repr(C)]
pub struct Il2CppDomain {
    _data: [u8; 0],
}

/// 方法信息
#[repr(C)]
pub struct MethodInfo {
    pub method_ptr: *mut c_void,
    // 其他字段省略
}

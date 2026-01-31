//! 构建脚本
//! 
//! 检查 DLL 文件是否存在，生成内嵌代码

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 告诉 rustc 我们会定义 has_embedded_dlls cfg
    println!("cargo::rustc-check-cfg=cfg(has_embedded_dlls)");
    
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let release_dir = Path::new(&manifest_dir).join("..").join("target").join("release");
    
    let unity_dll = release_dir.join("UnityPlayer.dll");
    let apphelp_dll = release_dir.join("apphelp.dll");
    
    // 输出调试信息
    println!("cargo:rerun-if-changed={}", unity_dll.display());
    println!("cargo:rerun-if-changed={}", apphelp_dll.display());
    
    // 检查 DLL 是否存在
    let has_unity = unity_dll.exists();
    let has_apphelp = apphelp_dll.exists();
    
    // 根据 DLL 是否存在设置编译特性
    if has_unity && has_apphelp {
        println!("cargo:rustc-cfg=has_embedded_dlls");
        println!("cargo:warning=Found DLLs for embedding: UnityPlayer.dll ({} KB), apphelp.dll ({} KB)", 
            fs::metadata(&unity_dll).map(|m| m.len() / 1024).unwrap_or(0),
            fs::metadata(&apphelp_dll).map(|m| m.len() / 1024).unwrap_or(0)
        );
    } else {
        println!("cargo:warning=DLLs not found for embedding. Will use external files at runtime.");
        if !has_unity {
            println!("cargo:warning=  Missing: {}", unity_dll.display());
        }
        if !has_apphelp {
            println!("cargo:warning=  Missing: {}", apphelp_dll.display());
        }
    }
}

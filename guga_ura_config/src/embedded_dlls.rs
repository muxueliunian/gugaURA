//! 内嵌 DLL 模块
//!
//! 使用 include_bytes! 在编译时将 DLL 嵌入到 EXE 中
//! 通过 build.rs 的 cfg 标志控制是否嵌入

/// 内嵌的 UnityPlayer.dll 数据
#[cfg(has_embedded_dlls)]
pub static UNITY_PLAYER_DLL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../target/release/UnityPlayer.dll"
));

#[cfg(not(has_embedded_dlls))]
pub static UNITY_PLAYER_DLL: &[u8] = &[];

/// 内嵌的 apphelp.dll 数据
#[cfg(has_embedded_dlls)]
pub static APPHELP_DLL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../target/release/apphelp.dll"
));

#[cfg(not(has_embedded_dlls))]
pub static APPHELP_DLL: &[u8] = &[];

/// 检查是否有内嵌的 DLL
pub fn has_embedded_dlls() -> bool {
    !UNITY_PLAYER_DLL.is_empty() && !APPHELP_DLL.is_empty()
}

/// 获取内嵌 DLL 的信息
pub fn get_embedded_info() -> String {
    if has_embedded_dlls() {
        format!(
            "内嵌: UnityPlayer.dll ({} KB), apphelp.dll ({} KB)",
            UNITY_PLAYER_DLL.len() / 1024,
            APPHELP_DLL.len() / 1024
        )
    } else {
        "未内嵌 DLL（将从外部文件加载）".to_string()
    }
}

//! DLL代理模块

pub mod unityplayer;
pub mod cri_mana_vpx;

/// 代理函数声明宏
/// 生成一个跳转到原始函数的导出函数
#[macro_export]
macro_rules! proxy_proc {
    ($name:ident, $orig_var_name:ident) => {
        static mut $orig_var_name: usize = 0;
        std::arch::global_asm!(
            concat!(".globl ", stringify!($name)),
            concat!(stringify!($name), ":"),
            "    jmp qword ptr [rip + {}]",
            sym $orig_var_name
        );
    }
}

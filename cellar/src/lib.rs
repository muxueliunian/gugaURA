//! Cellar - 游戏反检测绕过模块
//!
//! 本代码来源于 Hachimi/Cellar 项目
//! 原始项目: https://github.com/Hachimi-Hachimi/Cellar
//! 许可证: GPL-3.0

#[macro_use] extern crate log;

pub mod core;

/** Windows **/
#[cfg(target_os = "windows")]
mod windows;
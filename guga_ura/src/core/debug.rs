//! Debug 模块
//!
//! 将拦截到的 msgpack 数据转换为 JSON 并保存到本地文件，
//! 用于分析游戏通信数据结构。

use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// 全局递增序号，确保文件名不重复
static REQUEST_SEQ: AtomicU64 = AtomicU64::new(0);
static RESPONSE_SEQ: AtomicU64 = AtomicU64::new(0);

/// 获取 debug 输出目录
/// - 优先使用 config 传入的绝对路径
/// - 未配置时回退到当前进程 EXE 同级 debug/
fn debug_dir(custom_output_dir: Option<&str>) -> PathBuf {
    if let Some(custom) = custom_output_dir {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return Path::new(trimmed).to_path_buf();
        }
    }

    let mut path = std::env::current_exe().unwrap_or_default();
    path.pop();
    path.push("debug");
    path
}

/// 确保 debug 目录存在
fn ensure_debug_dir(custom_output_dir: Option<&str>) -> Result<PathBuf, String> {
    let dir = debug_dir(custom_output_dir);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create debug dir {}: {}", dir.display(), e))?;
    Ok(dir)
}

/// 将 msgpack 数据转换为 JSON 并保存
///
/// direction: "request" 或 "response"
pub fn save_msgpack_as_json(data: &[u8], direction: &str, output_dir: Option<&str>) {
    // 在后台线程中处理，避免阻塞游戏
    let data = data.to_vec();
    let direction = direction.to_string();
    let output_dir = output_dir.map(|s| s.to_string());

    std::thread::spawn(move || {
        match panic::catch_unwind(AssertUnwindSafe(|| {
            save_impl(&data, &direction, output_dir.as_deref())
        })) {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                warn!("Debug save failed: {}", e);
            }
            Err(_) => {
                error!("Debug save thread panicked");
            }
        }
    });
}

fn save_impl(data: &[u8], direction: &str, output_dir: Option<&str>) -> Result<(), String> {
    let dir = ensure_debug_dir(output_dir)?;

    // 生成文件名：方向_序号_时间戳.json
    let seq = match direction {
        "request" => REQUEST_SEQ.fetch_add(1, Ordering::Relaxed),
        _ => RESPONSE_SEQ.fetch_add(1, Ordering::Relaxed),
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let file_stem = format!("{}_{:04}_{}", direction, seq, timestamp);

    // 尝试将 msgpack 解码为 JSON Value；失败时仅记录日志
    match rmp_serde::from_slice::<serde_json::Value>(data) {
        Ok(json_value) => {
            let json_filename = format!("{}.json", file_stem);
            let json_path = dir.join(&json_filename);

            match serde_json::to_string_pretty(&json_value) {
                Ok(json_str) => {
                    if let Err(e) = fs::write(&json_path, &json_str) {
                        warn!("Debug JSON write failed ({}): {}", json_path.display(), e);
                    } else {
                        info!(
                            "Debug: saved {} ({} bytes -> {} bytes JSON) to {}",
                            json_filename,
                            data.len(),
                            json_str.len(),
                            dir.display()
                        );
                    }
                }
                Err(e) => {
                    warn!("Debug JSON serialize failed: {}", e);
                }
            }
        }
        Err(e) => {
            warn!("Debug msgpack decode failed: {}", e);
        }
    }

    Ok(())
}

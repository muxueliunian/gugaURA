//! 运行时诊断日志
//!
//! 为了便于在用户机器上回收启动链路问题，这里额外写一份落盘日志到
//! 游戏目录下的 `guga_ura_data/guga_ura_runtime.log`。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static TRACE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// 获取游戏目录
pub fn game_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// 获取 guga_ura 数据目录
pub fn data_dir() -> PathBuf {
    game_dir().join("guga_ura_data")
}

/// 获取运行时诊断日志路径
pub fn runtime_log_path() -> PathBuf {
    preferred_runtime_log_dir().join("guga_ura_runtime.log")
}

/// 追加一行运行时诊断日志
pub fn append_runtime_log(message: &str) {
    let lock = TRACE_LOCK.get_or_init(|| Mutex::new(()));
    let Ok(_guard) = lock.lock() else {
        return;
    };

    let line = format!(
        "[{}] {}",
        unix_timestamp_millis(),
        sanitize_message(message)
    );

    for dir in runtime_log_dirs() {
        if fs::create_dir_all(&dir).is_err() {
            continue;
        }

        let log_path = dir.join("guga_ura_runtime.log");
        let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) else {
            continue;
        };

        let _ = writeln!(file, "{}", line);
        return;
    }
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn sanitize_message(message: &str) -> String {
    message.replace('\r', " ").replace('\n', " ")
}

fn preferred_runtime_log_dir() -> PathBuf {
    runtime_log_dirs().into_iter().next().unwrap_or_else(data_dir)
}

fn runtime_log_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        dirs.push(
            PathBuf::from(local_app_data)
                .join("GugaURA")
                .join("logs"),
        );
    }

    if let Some(temp_dir) = std::env::var_os("TEMP") {
        dirs.push(PathBuf::from(temp_dir).join("GugaURA"));
    }

    dirs.push(data_dir());
    dirs
}

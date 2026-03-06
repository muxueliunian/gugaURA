//! 配置文件监控
//!
//! 使用 notify crate 监控 guga_ura_config.json 文件变更，
//! 文件修改后自动触发配置热重载。

use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::time::Duration;

use super::config::Config;
use super::GugaURA;

/// 启动配置文件监控线程
///
/// 在后台线程中监控配置文件，当文件被修改时自动重载配置。
/// 包含 200ms 的 debounce 防止短时间内多次重载。
pub fn start_config_watcher() {
    std::thread::spawn(|| {
        let config_path = Config::config_path();

        // 监控配置文件所在的目录（notify 需要监控目录）
        let watch_dir = match config_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => {
                error!("Cannot determine config file directory");
                return;
            }
        };

        let config_filename = config_path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();

        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher =
            match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            }) {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to create file watcher: {}", e);
                    return;
                }
            };

        if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
            error!("Failed to watch config directory: {}", e);
            return;
        }

        info!(
            "Config file watcher started, watching: {}",
            config_path.display()
        );

        loop {
            match rx.recv() {
                Ok(event) => {
                    // 只处理修改事件
                    if !matches!(event.kind, EventKind::Modify(_)) {
                        continue;
                    }

                    // 只处理目标配置文件
                    let is_config_file = event
                        .paths
                        .iter()
                        .any(|p| p.file_name().map(|n| n == config_filename).unwrap_or(false));

                    if !is_config_file {
                        continue;
                    }

                    // Debounce: 等待 200ms 让写入完成
                    std::thread::sleep(Duration::from_millis(200));

                    // 排空后续的重复事件
                    while rx.try_recv().is_ok() {}

                    info!("Config file changed, reloading...");
                    GugaURA::reload_config();
                }
                Err(e) => {
                    error!("Config watcher channel error: {}", e);
                    break;
                }
            }
        }
    });
}

//! 内置本地接收器
//!
//! 在配置工具进程内监听本地 HTTP 端口，接收插件转发的 msgpack/json，
//! 并保存为 JSON 到配置工具 EXE 同级 debug/ 目录。

use serde_json::json;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use tiny_http::{Method, Request, Response, Server, StatusCode};

const MAX_LOG_LINES: usize = 600;

static SEQ: AtomicU64 = AtomicU64::new(0);
static LOG_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

pub fn start_embedded_receiver() -> String {
    let addr =
        std::env::var("GUGAURA_RECEIVER_ADDR").unwrap_or_else(|_| "127.0.0.1:4693".to_string());
    log_info(format!("准备启动内置接收器: {}", addr));

    let output_dir = default_output_dir();
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let msg = format!(
            "启动失败: 无法创建输出目录 {} ({})",
            output_dir.display(),
            e
        );
        log_error(&msg);
        return msg;
    }

    let server = match Server::http(&addr) {
        Ok(server) => server,
        Err(e) => {
            let msg = format!("启动失败: 监听 {} 失败 ({})", addr, e);
            log_error(&msg);
            return msg;
        }
    };

    let output_dir_for_thread = output_dir.clone();
    let addr_for_thread = addr.clone();
    std::thread::spawn(move || {
        log_info(format!("内置接收器已启动: {}", addr_for_thread));
        for request in server.incoming_requests() {
            handle_request(request, &output_dir_for_thread);
        }
    });

    let msg = format!("监听 {}，输出 {}", addr, output_dir.display());
    log_info(&msg);
    msg
}

pub fn snapshot_logs(limit: usize) -> Vec<String> {
    let take = if limit == 0 { MAX_LOG_LINES } else { limit };
    match log_buffer().lock() {
        Ok(guard) => {
            let mut lines: Vec<String> = guard.iter().rev().take(take).cloned().collect();
            lines.reverse();
            lines
        }
        Err(_) => vec!["[00:00:00][ERROR] 日志缓冲区锁异常".to_string()],
    }
}

pub fn clear_logs() {
    if let Ok(mut guard) = log_buffer().lock() {
        guard.clear();
    }
}

fn handle_request(mut request: Request, output_dir: &Path) {
    if request.method() != &Method::Post {
        log_warn(format!(
            "拒绝非 POST 请求: method={} route={}",
            request.method(),
            request.url()
        ));
        let _ = request.respond(
            Response::from_string("Only POST is supported").with_status_code(StatusCode(405)),
        );
        return;
    }

    let route = request.url().to_string();
    let direction = guga_ura_fans::infer_direction(&route);

    let mut body = Vec::new();
    if let Err(e) = request.as_reader().read_to_end(&mut body) {
        log_warn(format!("读取请求体失败: route={} error={}", route, e));
        let _ = request.respond(
            Response::from_string(format!("Read body failed: {}", e))
                .with_status_code(StatusCode(400)),
        );
        return;
    }

    if body.is_empty() {
        log_warn(format!("收到空请求体: route={}", route));
        let _ = request
            .respond(Response::from_string("Empty request body").with_status_code(StatusCode(400)));
        return;
    }

    if !guga_ura_fans::should_persist_debug_payload(&direction, &route) {
        let _ = request.respond(Response::from_string("ignored: non-response payload"));
        return;
    }

    let headers_json = headers_to_json(&request);

    match guga_ura_fans::decode_payload(&body) {
        Ok((decoded_as, payload)) => {
            let now_ms = guga_ura_fans::now_millis();
            let seq = SEQ.fetch_add(1, Ordering::Relaxed);
            let filename = format!("{}_{:06}_{}.json", direction, seq, now_ms);
            let file_path = output_dir.join(filename);

            let fans_settings = guga_ura_fans::resolve_fans_settings_from_exe_config();
            if fans_settings.enabled {
                match guga_ura_fans::upsert_fans_from_decoded_payload(
                    &payload,
                    &direction,
                    &route,
                    now_ms,
                    &fans_settings.output_dir,
                ) {
                    Ok(Some(path)) => log_info(format!(
                        "Fans 聚合更新: route={} output={}",
                        route,
                        path.display()
                    )),
                    Ok(None) => {}
                    Err(e) => log_warn(format!("Fans 聚合失败: route={} error={}", route, e)),
                }
            }

            let wrapper = json!({
                "direction": direction,
                "route": route.clone(),
                "received_at_unix_ms": now_ms,
                "payload_size": body.len(),
                "decoded_as": decoded_as,
                "headers": headers_json,
                "payload": payload
            });

            match serde_json::to_string_pretty(&wrapper) {
                Ok(content) => {
                    if let Err(e) = fs::write(&file_path, content) {
                        log_error(format!(
                            "写入 debug 文件失败: route={} output={} error={}",
                            route,
                            file_path.display(),
                            e
                        ));
                        let _ = request.respond(
                            Response::from_string(format!("Write file failed: {}", e))
                                .with_status_code(StatusCode(500)),
                        );
                        return;
                    }

                    log_info(format!(
                        "保存 payload 成功: route={} decoded={} output={}",
                        route,
                        decoded_as,
                        file_path.display()
                    ));
                    let _ = request.respond(Response::from_string(format!(
                        "saved: {}",
                        file_path.display()
                    )));
                }
                Err(e) => {
                    log_error(format!("序列化 JSON 失败: route={} error={}", route, e));
                    let _ = request.respond(
                        Response::from_string(format!("Serialize json failed: {}", e))
                            .with_status_code(StatusCode(500)),
                    );
                }
            }
        }
        Err(e) => {
            log_warn(format!("payload 解码失败: route={} error={}", route, e));
            let _ = request.respond(Response::from_string(e).with_status_code(StatusCode(400)));
        }
    }
}

fn headers_to_json(request: &Request) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for h in request.headers() {
        map.insert(
            h.field.to_string(),
            serde_json::Value::String(h.value.to_string()),
        );
    }
    serde_json::Value::Object(map)
}

fn default_output_dir() -> PathBuf {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        return exe_path.join("debug");
    }

    Path::new(".").join("debug")
}

fn log_buffer() -> &'static Mutex<VecDeque<String>> {
    LOG_BUFFER.get_or_init(|| Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES)))
}

fn log_info(message: impl AsRef<str>) {
    push_log("INFO", message.as_ref());
}

fn log_warn(message: impl AsRef<str>) {
    push_log("WARN", message.as_ref());
}

fn log_error(message: impl AsRef<str>) {
    push_log("ERROR", message.as_ref());
}

fn push_log(level: &str, message: &str) {
    let line = format!("[{}][{}] {}", hms_now(), level, message);
    eprintln!("{}", line);

    if let Ok(mut guard) = log_buffer().lock() {
        guard.push_back(line);
        while guard.len() > MAX_LOG_LINES {
            guard.pop_front();
        }
    }
}

fn hms_now() -> String {
    use chrono::Local;
    Local::now().format("%H:%M:%S").to_string()
}

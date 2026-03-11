//! 内置本地接收器
//!
//! 在配置工具进程内监听本地 HTTP 端口，接收插件转发的 msgpack/json，
//! 并保存为 JSON 到配置工具 EXE 同级 debug/ 目录。

use crate::config::Config;
use crate::receiver_pipeline::{self, ReceiverHeader, ReceiverProcessOutcome, RelayOutcome};
use serde_json::json;
use std::collections::VecDeque;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use tiny_http::{Method, Request, Response, Server, StatusCode};

const MAX_LOG_LINES: usize = 600;
pub const DEFAULT_RECEIVER_LISTEN_ADDR: &str = "127.0.0.1:4693";

static SEQ: AtomicU64 = AtomicU64::new(0);
static LOG_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverListenAddrSource {
    Cli,
    Env,
    ExeConfig,
    Default,
}

impl ReceiverListenAddrSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Env => "env",
            Self::ExeConfig => "exeConfig",
            Self::Default => "default",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReceiverListenAddrResolution {
    pub listen_addr: String,
    pub configured_listen_addr: String,
    pub source: ReceiverListenAddrSource,
}

#[derive(Debug, Clone)]
pub struct ReceiverRuntimeInfo {
    pub ready: bool,
    pub status: String,
    pub listen_addr: String,
    pub configured_listen_addr: String,
    pub source: ReceiverListenAddrSource,
}

pub fn start_embedded_receiver() -> String {
    start_embedded_receiver_with_runtime().status
}

pub fn start_embedded_receiver_with_runtime() -> ReceiverRuntimeInfo {
    let resolution = resolve_receiver_listen_addr(None);
    log_info(format!(
        "准备启动内置接收器: {} (source={} configured={})",
        resolution.listen_addr,
        resolution.source.as_str(),
        resolution.configured_listen_addr
    ));

    let output_dir = default_output_dir();
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let msg = format!(
            "启动失败: 无法创建输出目录 {} ({})",
            output_dir.display(),
            e
        );
        log_error(&msg);
        return build_runtime_info(&resolution, false, msg);
    }

    let server = match Server::http(&resolution.listen_addr) {
        Ok(server) => server,
        Err(e) => {
            let msg = format!("启动失败: 监听 {} 失败 ({})", resolution.listen_addr, e);
            log_error(&msg);
            return build_runtime_info(&resolution, false, msg);
        }
    };

    let output_dir_for_thread = output_dir.clone();
    let addr_for_thread = resolution.listen_addr.clone();
    let listen_addr_for_thread = resolution.listen_addr.clone();
    std::thread::spawn(move || {
        log_info(format!("内置接收器已启动: {}", addr_for_thread));
        for request in server.incoming_requests() {
            handle_request(request, &output_dir_for_thread, &listen_addr_for_thread);
        }
    });

    let msg = format!(
        "监听 {}，输出 {}",
        resolution.listen_addr,
        output_dir.display()
    );
    log_info(&msg);
    build_runtime_info(&resolution, true, msg)
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

pub fn resolve_receiver_listen_addr(cli_override: Option<&str>) -> ReceiverListenAddrResolution {
    let configured_listen_addr = configured_receiver_listen_addr();
    let env_override = std::env::var("GUGAURA_RECEIVER_ADDR").ok();
    resolve_receiver_listen_addr_with_inputs(
        cli_override,
        env_override.as_deref(),
        configured_listen_addr.as_str(),
    )
}

pub fn parse_receiver_listen_addr(listen_addr: &str) -> Option<(String, u16)> {
    let normalized = normalize_listen_addr(Some(listen_addr))?;
    let (host, port) = normalized.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    Some((host.to_string(), port))
}

fn handle_request(mut request: Request, output_dir: &Path, self_listen_addr: &str) {
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
    let mut body = Vec::new();
    if let Err(e) = request.as_reader().read_to_end(&mut body) {
        log_warn(format!("读取请求体失败: route={} error={}", route, e));
        let _ = request.respond(
            Response::from_string(format!("Read body failed: {}", e))
                .with_status_code(StatusCode(400)),
        );
        return;
    }

    let headers_json = headers_to_json(&request);
    let relay_headers = headers_to_relay_headers(&request);

    let response =
        match receiver_pipeline::prepare_receiver_payload(output_dir, &route, None, &body, || {
            SEQ.fetch_add(1, Ordering::Relaxed)
        }) {
            Ok(ReceiverProcessOutcome::Ignored) => {
                (StatusCode(200), "ignored: non-response payload".to_string())
            }
            Ok(ReceiverProcessOutcome::Saved(prepared)) => {
                if let Some(path) = prepared.fans_output_path.as_ref() {
                    log_info(format!(
                        "社团Fans 更新: route={} output={}",
                        prepared.route,
                        path.display()
                    ));
                }
                if let Some(error) = prepared.fans_error.as_ref() {
                    log_warn(format!(
                        "社团Fans 失败: route={} error={}",
                        prepared.route, error
                    ));
                }

                let wrapper = json!({
                    "direction": prepared.direction,
                    "route": prepared.route,
                    "received_at_unix_ms": prepared.now_ms,
                    "payload_size": body.len(),
                    "decoded_as": prepared.decoded_as,
                    "headers": headers_json,
                    "payload": prepared.payload
                });

                match receiver_pipeline::write_receiver_payload_json(&prepared.file_path, &wrapper)
                {
                    Ok(()) => {
                        log_info(format!(
                            "保存 payload 成功: route={} decoded={} output={}",
                            route,
                            wrapper["decoded_as"].as_str().unwrap_or("unknown"),
                            prepared.file_path.display()
                        ));
                        (
                            StatusCode(200),
                            format!("saved: {}", prepared.file_path.display()),
                        )
                    }
                    Err(error) => {
                        log_error(format!(
                            "写入 debug 文件失败: route={} output={} error={}",
                            route,
                            prepared.file_path.display(),
                            error
                        ));
                        (StatusCode(500), error)
                    }
                }
            }
            Err(error) => {
                if error.contains("Empty request body") {
                    log_warn(format!("收到空请求体: route={}", route));
                } else {
                    log_warn(format!("payload 解码失败: route={} error={}", route, error));
                }
                (StatusCode(400), error)
            }
        };

    log_relay_outcome(
        receiver_pipeline::relay_receiver_payload(self_listen_addr, &route, &body, &relay_headers),
        &route,
    );

    let _ = request.respond(Response::from_string(response.1).with_status_code(response.0));
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

fn headers_to_relay_headers(request: &Request) -> Vec<ReceiverHeader> {
    request
        .headers()
        .iter()
        .map(|header| ReceiverHeader::new(header.field.to_string(), header.value.to_string()))
        .collect()
}

fn default_output_dir() -> PathBuf {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        return exe_path.join("debug");
    }

    Path::new(".").join("debug")
}

fn configured_receiver_listen_addr() -> String {
    let raw = Config::load_from_exe_dir().receiver_listen_addr;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        DEFAULT_RECEIVER_LISTEN_ADDR.to_string()
    } else {
        trimmed.to_string()
    }
}

fn resolve_receiver_listen_addr_with_inputs(
    cli_override: Option<&str>,
    env_override: Option<&str>,
    configured_listen_addr: &str,
) -> ReceiverListenAddrResolution {
    if let Some(listen_addr) = normalize_listen_addr(cli_override) {
        return ReceiverListenAddrResolution {
            listen_addr,
            configured_listen_addr: configured_listen_addr.to_string(),
            source: ReceiverListenAddrSource::Cli,
        };
    }

    if let Some(listen_addr) = normalize_listen_addr(env_override) {
        return ReceiverListenAddrResolution {
            listen_addr,
            configured_listen_addr: configured_listen_addr.to_string(),
            source: ReceiverListenAddrSource::Env,
        };
    }

    if let Some(listen_addr) = normalize_listen_addr(Some(configured_listen_addr)) {
        return ReceiverListenAddrResolution {
            listen_addr,
            configured_listen_addr: configured_listen_addr.to_string(),
            source: ReceiverListenAddrSource::ExeConfig,
        };
    }

    ReceiverListenAddrResolution {
        listen_addr: DEFAULT_RECEIVER_LISTEN_ADDR.to_string(),
        configured_listen_addr: configured_listen_addr.to_string(),
        source: ReceiverListenAddrSource::Default,
    }
}

fn normalize_listen_addr(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    let (host, port) = value.rsplit_once(':')?;
    let host = host.trim();
    let port = port.trim().parse::<u16>().ok()?;

    if host.is_empty() || port == 0 || !is_valid_listen_host(host) {
        return None;
    }

    Some(format!("{}:{}", host, port))
}

fn is_valid_listen_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost") || host.parse::<IpAddr>().is_ok()
}

fn build_runtime_info(
    resolution: &ReceiverListenAddrResolution,
    ready: bool,
    status: String,
) -> ReceiverRuntimeInfo {
    ReceiverRuntimeInfo {
        ready,
        status,
        listen_addr: resolution.listen_addr.clone(),
        configured_listen_addr: resolution.configured_listen_addr.clone(),
        source: resolution.source,
    }
}

fn log_relay_outcome(outcome: RelayOutcome, route: &str) {
    match outcome {
        RelayOutcome::Disabled | RelayOutcome::AlreadyRelayed => {}
        RelayOutcome::SelfLoopBlocked => {
            log_warn(format!("Relay skipped due to self-loop: route={}", route));
        }
        RelayOutcome::Forwarded(target) => {
            log_info(format!(
                "Relay forwarded: route={} target={}",
                route, target
            ));
        }
        RelayOutcome::Failed(error) => {
            log_warn(format!("Relay failed: route={} error={}", route, error));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{
        clear_logs, parse_receiver_listen_addr, push_log, resolve_receiver_listen_addr_with_inputs,
        snapshot_logs, ReceiverListenAddrSource, DEFAULT_RECEIVER_LISTEN_ADDR, MAX_LOG_LINES,
    };

    #[test]
    fn log_buffer_should_keep_latest_lines() {
        clear_logs();

        for i in 0..(MAX_LOG_LINES + 5) {
            push_log("INFO", &format!("line {}", i));
        }

        let lines = snapshot_logs(0);

        assert_eq!(lines.len(), MAX_LOG_LINES);
        assert!(lines.first().is_some_and(|line| line.contains("line 5")));
        assert!(lines.last().is_some_and(|line| line.contains("line 604")));

        clear_logs();
    }

    #[test]
    fn receiver_listen_addr_resolution_should_follow_precedence() {
        let resolution = resolve_receiver_listen_addr_with_inputs(
            Some("127.0.0.1:4700"),
            Some("127.0.0.1:4800"),
            "127.0.0.1:4900",
        );
        assert_eq!(resolution.listen_addr, "127.0.0.1:4700");
        assert_eq!(resolution.source, ReceiverListenAddrSource::Cli);

        let env_resolution = resolve_receiver_listen_addr_with_inputs(
            None,
            Some("127.0.0.1:4800"),
            "127.0.0.1:4900",
        );
        assert_eq!(env_resolution.listen_addr, "127.0.0.1:4800");
        assert_eq!(env_resolution.source, ReceiverListenAddrSource::Env);

        let config_resolution =
            resolve_receiver_listen_addr_with_inputs(None, None, "127.0.0.1:4900");
        assert_eq!(config_resolution.listen_addr, "127.0.0.1:4900");
        assert_eq!(
            config_resolution.source,
            ReceiverListenAddrSource::ExeConfig
        );
    }

    #[test]
    fn receiver_listen_addr_resolution_should_fall_back_to_default_on_invalid_inputs() {
        let resolution =
            resolve_receiver_listen_addr_with_inputs(None, Some("bad-host:4800"), "bad:4900");

        assert_eq!(resolution.listen_addr, DEFAULT_RECEIVER_LISTEN_ADDR);
        assert_eq!(resolution.source, ReceiverListenAddrSource::Default);
    }

    #[test]
    fn parse_receiver_listen_addr_should_support_localhost_and_ip() {
        assert_eq!(
            parse_receiver_listen_addr("127.0.0.1:4693"),
            Some(("127.0.0.1".to_string(), 4693))
        );
        assert_eq!(
            parse_receiver_listen_addr("localhost:4700"),
            Some(("localhost".to_string(), 4700))
        );
        assert_eq!(parse_receiver_listen_addr("bad-host:4700"), None);
    }
}

//! Receiver 公共处理函数
//!
//! 该模块只抽取内置 Receiver 与独立 Receiver 共享的 payload 处理逻辑，
//! 不引入新的 server 抽象，不改变各自 transport 壳。

use crate::config::Config;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use url::Url;

pub const RELAY_HEADER_NAME: &str = "x-gugaura-relayed";
pub const RELAY_HEADER_VALUE: &str = "1";

#[derive(Debug)]
pub enum ReceiverProcessOutcome {
    Ignored,
    Saved(PreparedReceiverPayload),
}

#[derive(Debug)]
pub struct PreparedReceiverPayload {
    pub route: String,
    pub direction: String,
    pub now_ms: u64,
    pub decoded_as: String,
    pub payload: Value,
    pub file_path: PathBuf,
    pub fans_output_path: Option<PathBuf>,
    pub fans_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiverHeader {
    pub name: String,
    pub value: String,
}

impl ReceiverHeader {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    fn matches(&self, expected: &str) -> bool {
        self.name.eq_ignore_ascii_case(expected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayOutcome {
    Disabled,
    AlreadyRelayed,
    SelfLoopBlocked,
    Forwarded(String),
    Failed(String),
}

#[derive(Debug, Clone)]
struct ReceiverRelaySettings {
    enabled: bool,
    target_host: Option<String>,
    timeout_ms: u64,
    self_listen_addr: String,
}

pub fn prepare_receiver_payload<F>(
    output_dir: &Path,
    route: &str,
    fixed_direction: Option<&str>,
    body: &[u8],
    next_seq: F,
) -> Result<ReceiverProcessOutcome, String>
where
    F: FnOnce() -> u64,
{
    if body.is_empty() {
        return Err("Empty request body".to_string());
    }

    let direction = resolve_direction(route, fixed_direction);
    if !guga_ura_fans::should_persist_debug_payload(&direction, route) {
        return Ok(ReceiverProcessOutcome::Ignored);
    }

    fs::create_dir_all(output_dir).map_err(|e| format!("create_dir_all failed: {}", e))?;

    let (decoded_as, payload) = guga_ura_fans::decode_payload(body)?;
    let now_ms = u64::try_from(guga_ura_fans::now_millis())
        .map_err(|_| "now_millis overflowed u64".to_string())?;
    let seq = next_seq();
    let filename = format!("{}_{:06}_{}.json", sanitize_tag(&direction), seq, now_ms);
    let file_path = output_dir.join(filename);

    let fans_settings = guga_ura_fans::resolve_fans_settings_from_exe_config();
    let (fans_output_path, fans_error) = if fans_settings.enabled {
        match guga_ura_fans::upsert_fans_from_decoded_payload(
            &payload,
            &direction,
            route,
            now_ms.into(),
            &fans_settings.output_dir,
        ) {
            Ok(path) => (path, None),
            Err(error) => (None, Some(error)),
        }
    } else {
        (None, None)
    };

    Ok(ReceiverProcessOutcome::Saved(PreparedReceiverPayload {
        route: route.to_string(),
        direction,
        now_ms,
        decoded_as: decoded_as.to_string(),
        payload,
        file_path,
        fans_output_path,
        fans_error,
    }))
}

pub fn write_receiver_payload_json(file_path: &Path, wrapper: &Value) -> Result<(), String> {
    let json_str = serde_json::to_string_pretty(wrapper)
        .map_err(|e| format!("to_string_pretty failed: {}", e))?;
    fs::write(file_path, json_str)
        .map_err(|e| format!("write {} failed: {}", file_path.display(), e))
}

pub fn resolve_direction(route: &str, fixed_direction: Option<&str>) -> String {
    fixed_direction
        .map(str::to_string)
        .unwrap_or_else(|| guga_ura_fans::infer_direction(route))
}

pub fn relay_target_would_loop(listen_addr: &str, relay_target_host: &str) -> bool {
    let Some(self_origin) = normalized_origin_from_listen_addr(listen_addr) else {
        return false;
    };
    let Some(target_origin) = normalized_origin_from_target(relay_target_host) else {
        return false;
    };
    self_origin == target_origin
}

pub fn relay_receiver_payload(
    self_listen_addr: &str,
    route: &str,
    body: &[u8],
    headers: &[ReceiverHeader],
) -> RelayOutcome {
    relay_receiver_payload_with_settings(
        &load_receiver_relay_settings(self_listen_addr),
        route,
        body,
        headers,
    )
}

fn sanitize_tag(tag: &str) -> String {
    let mut out = String::with_capacity(tag.len());
    for ch in tag.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

fn load_receiver_relay_settings(self_listen_addr: &str) -> ReceiverRelaySettings {
    let config = Config::load_from_exe_dir();
    ReceiverRelaySettings {
        enabled: config.relay_enabled,
        target_host: config
            .relay_target_host
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        timeout_ms: config.timeout_ms.max(1),
        self_listen_addr: self_listen_addr.to_string(),
    }
}

fn relay_receiver_payload_with_settings(
    settings: &ReceiverRelaySettings,
    route: &str,
    body: &[u8],
    headers: &[ReceiverHeader],
) -> RelayOutcome {
    if !settings.enabled || settings.target_host.is_none() {
        return RelayOutcome::Disabled;
    }

    if headers.iter().any(|header| {
        header.matches(RELAY_HEADER_NAME) && header.value.trim() == RELAY_HEADER_VALUE
    }) {
        return RelayOutcome::AlreadyRelayed;
    }

    let target_host = settings.target_host.as_deref().unwrap_or_default();
    if relay_target_would_loop(&settings.self_listen_addr, target_host) {
        return RelayOutcome::SelfLoopBlocked;
    }

    let Some(base_url) = normalize_target_base_url(target_host) else {
        return RelayOutcome::Failed(format!("invalid relay target: {}", target_host));
    };

    let relay_url = match build_relay_url(&base_url, route) {
        Ok(url) => url,
        Err(error) => return RelayOutcome::Failed(error),
    };

    let timeout = Duration::from_millis(settings.timeout_ms);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();

    let mut request = agent.post(&relay_url);
    for header in headers
        .iter()
        .filter(|header| should_forward_header(&header.name))
    {
        request = request.set(&header.name, &header.value);
    }
    request = request.set(RELAY_HEADER_NAME, RELAY_HEADER_VALUE);

    match request.send_bytes(body) {
        Ok(_) => RelayOutcome::Forwarded(relay_url),
        Err(error) => RelayOutcome::Failed(format!("POST {} failed: {}", relay_url, error)),
    }
}

fn should_forward_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("content-type") || name.eq_ignore_ascii_case("x-plugin-name")
}

fn normalize_target_base_url(value: &str) -> Option<Url> {
    let mut url = Url::parse(value.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return None;
    }

    url.set_query(None);
    url.set_fragment(None);

    let path = if url.path().is_empty() {
        "/".to_string()
    } else {
        format!("{}/", url.path().trim_end_matches('/'))
    };
    url.set_path(&path);
    Some(url)
}

fn build_relay_url(base_url: &Url, route: &str) -> Result<String, String> {
    let relative = route.trim_start_matches('/');
    base_url
        .join(relative)
        .map(|url| url.to_string())
        .map_err(|error| format!("build relay url failed for {}: {}", route, error))
}

fn normalized_origin_from_target(target_host: &str) -> Option<String> {
    let base_url = normalize_target_base_url(target_host)?;
    normalized_origin_from_url(&base_url)
}

fn normalized_origin_from_listen_addr(listen_addr: &str) -> Option<String> {
    let (host, port) = crate::receiver::parse_receiver_listen_addr(listen_addr)?;
    let url = Url::parse(&format!("http://{}:{}/", host, port)).ok()?;
    normalized_origin_from_url(&url)
}

fn normalized_origin_from_url(url: &Url) -> Option<String> {
    let scheme = url.scheme().to_ascii_lowercase();
    let host = canonical_host(url.host_str()?);
    let port = url.port_or_known_default()?;
    Some(format!("{}://{}:{}", scheme, host, port))
}

fn canonical_host(host: &str) -> String {
    if host.eq_ignore_ascii_case("localhost") {
        "127.0.0.1".to_string()
    } else {
        host.to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        prepare_receiver_payload, relay_receiver_payload_with_settings, relay_target_would_loop,
        resolve_direction, ReceiverHeader, ReceiverProcessOutcome, ReceiverRelaySettings,
        RELAY_HEADER_NAME, RELAY_HEADER_VALUE,
    };
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::thread;
    use tiny_http::{Response, Server};

    #[test]
    fn resolve_direction_should_use_fixed_value_first() {
        assert_eq!(
            resolve_direction("/notify/response", Some("request")),
            "request"
        );
        assert_eq!(resolve_direction("/notify/response", None), "response");
    }

    #[test]
    fn prepare_receiver_payload_should_ignore_non_persist_route() {
        let output_dir = PathBuf::from("unused");
        let body = [1_u8, 2, 3];

        let outcome = prepare_receiver_payload(&output_dir, "/notify/request", None, &body, || 0)
            .expect("非 response 路由应被忽略");

        assert!(matches!(outcome, ReceiverProcessOutcome::Ignored));
    }

    #[test]
    fn prepare_receiver_payload_should_reject_empty_body() {
        let output_dir = PathBuf::from("unused");

        let error = prepare_receiver_payload(&output_dir, "/notify/response", None, &[], || 0)
            .expect_err("空请求体应报错");

        assert!(error.contains("Empty request body"));
    }

    #[test]
    fn relay_target_would_loop_should_treat_localhost_as_self() {
        assert!(relay_target_would_loop(
            "127.0.0.1:4693",
            "http://localhost:4693/api"
        ));
        assert!(!relay_target_would_loop(
            "127.0.0.1:4693",
            "http://127.0.0.1:4700/api"
        ));
    }

    #[test]
    fn relay_receiver_payload_should_skip_already_relayed_requests() {
        let settings = ReceiverRelaySettings {
            enabled: true,
            target_host: Some("http://127.0.0.1:4800".to_string()),
            timeout_ms: 100,
            self_listen_addr: "127.0.0.1:4693".to_string(),
        };
        let headers = vec![ReceiverHeader::new(RELAY_HEADER_NAME, RELAY_HEADER_VALUE)];

        let outcome =
            relay_receiver_payload_with_settings(&settings, "/notify/response", b"ping", &headers);

        assert_eq!(outcome, super::RelayOutcome::AlreadyRelayed);
    }

    #[test]
    fn relay_receiver_payload_should_block_self_loop() {
        let settings = ReceiverRelaySettings {
            enabled: true,
            target_host: Some("http://localhost:4693/relay".to_string()),
            timeout_ms: 100,
            self_listen_addr: "127.0.0.1:4693".to_string(),
        };

        let outcome =
            relay_receiver_payload_with_settings(&settings, "/notify/response", b"ping", &[]);

        assert_eq!(outcome, super::RelayOutcome::SelfLoopBlocked);
    }

    #[test]
    fn relay_receiver_payload_should_forward_body_and_headers() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("预留测试端口失败");
        let port = listener.local_addr().expect("读取测试端口失败").port();
        drop(listener);

        let server = Server::http(format!("127.0.0.1:{}", port)).expect("启动测试 HTTP 服务失败");
        let handle = thread::spawn(move || {
            let mut request = server.recv().expect("接收 relay 请求失败");
            let body = {
                let mut reader = request.as_reader();
                let mut body = String::new();
                std::io::Read::read_to_string(&mut reader, &mut body)
                    .expect("读取 relay 请求体失败");
                body
            };
            let url = request.url().to_string();
            let headers = request
                .headers()
                .iter()
                .map(|header| format!("{}: {}", header.field, header.value))
                .collect::<Vec<_>>()
                .join("\n");
            request
                .respond(Response::empty(200))
                .expect("返回 relay 响应失败");
            (url, headers, body)
        });

        let settings = ReceiverRelaySettings {
            enabled: true,
            target_host: Some(format!("http://127.0.0.1:{}/relay-base", port)),
            timeout_ms: 1000,
            self_listen_addr: "127.0.0.1:4693".to_string(),
        };
        let headers = vec![
            ReceiverHeader::new("content-type", "application/octet-stream"),
            ReceiverHeader::new("x-plugin-name", "guga"),
        ];

        let outcome =
            relay_receiver_payload_with_settings(&settings, "/notify/response", b"ping", &headers);

        let (url, headers, body) = handle.join().expect("relay 线程退出异常");
        let headers_lower = headers.to_ascii_lowercase();

        match outcome {
            super::RelayOutcome::Forwarded(url) => {
                assert!(url.contains("/relay-base/notify/response"));
            }
            other => panic!("应成功转发，实际为 {:?}", other),
        }

        assert_eq!(url, "/relay-base/notify/response");
        assert!(headers_lower.contains("content-type: application/octet-stream"));
        assert!(headers_lower.contains("x-plugin-name: guga"));
        assert!(headers_lower.contains("x-gugaura-relayed: 1"));
        assert_eq!(body, "ping");
    }
}

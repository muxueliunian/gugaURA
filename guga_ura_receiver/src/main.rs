use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use clap::Parser;
use guga_ura_config_core::receiver;
use guga_ura_config_core::receiver_pipeline::{
    self, ReceiverHeader, ReceiverProcessOutcome, RelayOutcome,
};
use log::{error, info, warn};
use serde_json::json;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path as FsPath, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug, Parser)]
#[command(name = "guga_ura_receiver")]
#[command(about = "Receive msgpack payloads from local plugins and save as JSON")]
struct Cli {
    #[arg(long)]
    host: Option<String>,

    #[arg(long)]
    port: Option<u16>,

    #[arg(long)]
    output_dir: Option<PathBuf>,
}

#[derive(Clone)]
struct AppState {
    output_dir: Arc<PathBuf>,
    seq: Arc<AtomicU64>,
    self_listen_addr: Arc<String>,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let base_resolution = receiver::resolve_receiver_listen_addr(None);
    let cli_listen_addr = build_cli_listen_addr(&cli, &base_resolution.listen_addr);
    let listen_resolution = cli_listen_addr
        .as_deref()
        .map(|listen_addr| receiver::resolve_receiver_listen_addr(Some(listen_addr)))
        .unwrap_or(base_resolution);

    let output_dir = cli.output_dir.unwrap_or_else(default_output_dir);
    if let Err(e) = fs::create_dir_all(&output_dir) {
        error!(
            "Failed to create output dir {}: {}",
            output_dir.display(),
            e
        );
        std::process::exit(1);
    }

    let (listen_host, listen_port) =
        receiver::parse_receiver_listen_addr(&listen_resolution.listen_addr).unwrap_or_else(|| {
            error!(
                "Failed to parse resolved listen addr {}",
                listen_resolution.listen_addr
            );
            std::process::exit(1);
        });
    let ip = parse_ip(&listen_host);
    let addr = SocketAddr::new(ip, listen_port);

    let state = AppState {
        output_dir: Arc::new(output_dir.clone()),
        seq: Arc::new(AtomicU64::new(0)),
        self_listen_addr: Arc::new(listen_resolution.listen_addr.clone()),
    };

    let app = Router::new()
        .route("/notify/request", post(handle_notify_request))
        .route("/notify/response", post(handle_notify_response))
        .route("/", post(handle_root))
        .route("/{*path}", post(handle_any))
        .with_state(state);

    info!("Receiver listening on http://{}", addr);
    info!(
        "Receiver listen addr source: {} (configured={})",
        listen_resolution.source.as_str(),
        listen_resolution.configured_listen_addr
    );
    info!("Debug output dir: {}", output_dir.display());
    let fans_settings = guga_ura_fans::resolve_fans_settings_from_exe_config();
    info!("Fans aggregate enabled: {}", fans_settings.enabled);
    info!("Fans output dir: {}", fans_settings.output_dir.display());

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }
}

async fn handle_notify_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    handle_payload(state, headers, body, "/notify/request", Some("request"))
}

async fn handle_notify_response(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    handle_payload(state, headers, body, "/notify/response", Some("response"))
}

async fn handle_root(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    handle_payload(state, headers, body, "/", None)
}

async fn handle_any(
    State(state): State<AppState>,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let route = format!("/{}", path);
    handle_payload(state, headers, body, &route, None)
}

fn handle_payload(
    state: AppState,
    headers: HeaderMap,
    body: Bytes,
    route: &str,
    fixed_direction: Option<&str>,
) -> (StatusCode, String) {
    if body.is_empty() {
        return (StatusCode::BAD_REQUEST, "Empty request body".to_string());
    }

    let relay_headers = headers_to_relay_headers(&headers);
    let direction = fixed_direction
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| guga_ura_fans::infer_direction(route));

    let response = match save_payload_as_json(&state, route, &direction, &headers, &body) {
        Ok(Some(file_path)) => (StatusCode::OK, format!("saved: {}", file_path.display())),
        Ok(None) => (StatusCode::OK, "ignored: non-response payload".to_string()),
        Err(e) => {
            warn!("Decode/save failed on route {}: {}", route, e);
            (StatusCode::BAD_REQUEST, e)
        }
    };

    log_relay_outcome(
        receiver_pipeline::relay_receiver_payload(
            &state.self_listen_addr,
            route,
            &body,
            &relay_headers,
        ),
        route,
    );

    response
}

fn save_payload_as_json(
    state: &AppState,
    route: &str,
    direction: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<Option<PathBuf>, String> {
    let plugin = header_value(headers, "x-plugin-name").unwrap_or("unknown");
    let content_type = header_value(headers, "content-type").unwrap_or("unknown");

    match receiver_pipeline::prepare_receiver_payload(
        state.output_dir.as_ref(),
        route,
        Some(direction),
        body,
        || state.seq.fetch_add(1, Ordering::Relaxed),
    )? {
        ReceiverProcessOutcome::Ignored => Ok(None),
        ReceiverProcessOutcome::Saved(prepared) => {
            if let Some(path) = prepared.fans_output_path.as_ref() {
                info!("Fans aggregate updated: {}", path.display());
            }
            if let Some(error) = prepared.fans_error.as_ref() {
                warn!("Fans aggregate failed on route {}: {}", route, error);
            }

            let wrapped = json!({
                "direction": prepared.direction,
                "route": prepared.route,
                "received_at_unix_ms": prepared.now_ms,
                "payload_size": body.len(),
                "decoded_as": prepared.decoded_as,
                "source": {
                    "plugin": plugin,
                    "content_type": content_type
                },
                "payload": prepared.payload
            });

            receiver_pipeline::write_receiver_payload_json(&prepared.file_path, &wrapped)?;

            info!(
                "Saved {} bytes from {} as {} ({})",
                body.len(),
                route,
                prepared.file_path.display(),
                wrapped["decoded_as"].as_str().unwrap_or("unknown")
            );

            Ok(Some(prepared.file_path))
        }
    }
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn headers_to_relay_headers(headers: &HeaderMap) -> Vec<ReceiverHeader> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| ReceiverHeader::new(name.as_str(), value))
        })
        .collect()
}

fn default_output_dir() -> PathBuf {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        return exe_path.join("debug");
    }
    FsPath::new(".").join("debug")
}

fn parse_ip(host: &str) -> IpAddr {
    if host.eq_ignore_ascii_case("localhost") {
        return IpAddr::V4(Ipv4Addr::LOCALHOST);
    }
    host.parse().unwrap_or_else(|_| {
        warn!("Invalid host '{}', fallback to 127.0.0.1", host);
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    })
}

fn build_cli_listen_addr(cli: &Cli, base_listen_addr: &str) -> Option<String> {
    if cli.host.is_none() && cli.port.is_none() {
        return None;
    }

    let (base_host, base_port) = receiver::parse_receiver_listen_addr(base_listen_addr)?;
    let host = cli.host.clone().unwrap_or(base_host);
    let port = cli.port.unwrap_or(base_port);
    Some(format!("{}:{}", host, port))
}

fn log_relay_outcome(outcome: RelayOutcome, route: &str) {
    match outcome {
        RelayOutcome::Disabled | RelayOutcome::AlreadyRelayed => {}
        RelayOutcome::SelfLoopBlocked => {
            warn!("Relay skipped due to self-loop on route {}", route);
        }
        RelayOutcome::Forwarded(target) => {
            info!("Relay forwarded: route={} target={}", route, target);
        }
        RelayOutcome::Failed(error) => {
            warn!("Relay failed on route {}: {}", route, error);
        }
    }
}

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use clap::Parser;
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
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = 4693)]
    port: u16,

    #[arg(long)]
    output_dir: Option<PathBuf>,
}

#[derive(Clone)]
struct AppState {
    output_dir: Arc<PathBuf>,
    seq: Arc<AtomicU64>,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    let output_dir = cli.output_dir.unwrap_or_else(default_output_dir);
    if let Err(e) = fs::create_dir_all(&output_dir) {
        error!(
            "Failed to create output dir {}: {}",
            output_dir.display(),
            e
        );
        std::process::exit(1);
    }

    let ip = parse_ip(&cli.host);
    let addr = SocketAddr::new(ip, cli.port);

    let state = AppState {
        output_dir: Arc::new(output_dir.clone()),
        seq: Arc::new(AtomicU64::new(0)),
    };

    let app = Router::new()
        .route("/notify/request", post(handle_notify_request))
        .route("/notify/response", post(handle_notify_response))
        .route("/", post(handle_root))
        .route("/{*path}", post(handle_any))
        .with_state(state);

    info!("Receiver listening on http://{}", addr);
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

    let direction = fixed_direction
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| guga_ura_fans::infer_direction(route));

    match save_payload_as_json(&state, route, &direction, &headers, &body) {
        Ok(Some(file_path)) => (StatusCode::OK, format!("saved: {}", file_path.display())),
        Ok(None) => (StatusCode::OK, "ignored: non-response payload".to_string()),
        Err(e) => {
            warn!("Decode/save failed on route {}: {}", route, e);
            (StatusCode::BAD_REQUEST, e)
        }
    }
}

fn save_payload_as_json(
    state: &AppState,
    route: &str,
    direction: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<Option<PathBuf>, String> {
    if !guga_ura_fans::should_persist_debug_payload(direction, route) {
        return Ok(None);
    }

    fs::create_dir_all(state.output_dir.as_ref())
        .map_err(|e| format!("create_dir_all failed: {}", e))?;

    let (decoded_as, payload) = guga_ura_fans::decode_payload(body)?;

    let now_ms = guga_ura_fans::now_millis();
    let seq = state.seq.fetch_add(1, Ordering::Relaxed);
    let direction_tag = sanitize_tag(direction);
    let filename = format!("{}_{:06}_{}.json", direction_tag, seq, now_ms);
    let file_path = state.output_dir.join(filename);

    let plugin = header_value(headers, "x-plugin-name").unwrap_or("unknown");
    let content_type = header_value(headers, "content-type").unwrap_or("unknown");

    let fans_settings = guga_ura_fans::resolve_fans_settings_from_exe_config();
    if fans_settings.enabled {
        match guga_ura_fans::upsert_fans_from_decoded_payload(
            &payload,
            direction,
            route,
            now_ms,
            &fans_settings.output_dir,
        ) {
            Ok(Some(path)) => info!("Fans aggregate updated: {}", path.display()),
            Ok(None) => {}
            Err(e) => warn!("Fans aggregate failed on route {}: {}", route, e),
        }
    }

    let wrapped = json!({
        "direction": direction,
        "route": route,
        "received_at_unix_ms": now_ms,
        "payload_size": body.len(),
        "decoded_as": decoded_as,
        "source": {
            "plugin": plugin,
            "content_type": content_type
        },
        "payload": payload
    });

    let json_str = serde_json::to_string_pretty(&wrapped)
        .map_err(|e| format!("to_string_pretty failed: {}", e))?;

    fs::write(&file_path, json_str)
        .map_err(|e| format!("write {} failed: {}", file_path.display(), e))?;

    info!(
        "Saved {} bytes from {} as {} ({})",
        body.len(),
        route,
        file_path.display(),
        decoded_as
    );

    Ok(Some(file_path))
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
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

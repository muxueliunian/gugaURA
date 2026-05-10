//! Tauri 命令层

use crate::bootstrap::BootstrapStateDto;
use crate::state::AppState;
use crate::tool_settings::{
    AppUpdateCheckDto, ToolSettingsActionResultDto, ToolSettingsContextDto,
};
use guga_ura_config_core::config::Config;
use guga_ura_config_core::detector::{
    detect_game_version, is_valid_game_dir, scan_installed_games as scan_installed_games_core,
    DetectedGame, GameVersion,
};
use guga_ura_config_core::installer::{
    check_install_status, install_dll, uninstall_dll, InstallStatus,
};
use guga_ura_config_core::receiver;
use guga_ura_config_core::receiver_pipeline;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};

/// 扫描结果 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedGameDto {
    pub path: String,
    pub version: String,
    pub version_label: String,
}

impl DetectedGameDto {
    fn from_detected_game(game: DetectedGame) -> Self {
        let (version, version_label) = map_game_version(game.version);
        Self {
            path: game.path.display().to_string(),
            version: version.to_string(),
            version_label: version_label.to_string(),
        }
    }
}

/// 手动检测目录结果 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectGameDirResultDto {
    pub path: String,
    pub exists: bool,
    pub is_valid_game_dir: bool,
    pub detected_version: String,
    pub detected_version_label: String,
    pub install_status: String,
    pub install_status_label: String,
}

impl InspectGameDirResultDto {
    fn new(
        game_dir: &std::path::Path,
        exists: bool,
        is_valid_game_dir: bool,
        detected_version: GameVersion,
        install_status: InstallStatus,
    ) -> Self {
        let (version, version_label) = map_game_version(detected_version);
        let (status, status_label) = map_install_status(install_status);

        Self {
            path: game_dir.display().to_string(),
            exists,
            is_valid_game_dir,
            detected_version: version.to_string(),
            detected_version_label: version_label.to_string(),
            install_status: status.to_string(),
            install_status_label: status_label.to_string(),
        }
    }
}

/// DLL 注入页上下文 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DllInjectionContextDto {
    pub path: String,
    pub has_path: bool,
    pub exists: bool,
    pub is_valid_game_dir: bool,
    pub detected_version: String,
    pub detected_version_label: String,
    pub install_status: String,
    pub install_status_label: String,
    pub notifier_host: String,
    pub timeout_ms: u64,
    pub debug_mode: bool,
    pub debug_output_dir: String,
    pub fans_enabled: bool,
    pub fans_output_dir: String,
    pub steam_requirement_note: String,
}

/// DLL 注入页保存输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDllInjectionConfigInput {
    pub path: String,
    pub notifier_host: String,
    pub timeout_ms: u64,
    pub fans_enabled: Option<bool>,
    pub fans_output_dir: Option<String>,
}

/// Debug 模式保存输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDebugModeInput {
    pub path: String,
    pub debug_mode: bool,
}

/// DLL 注入页动作返回
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DllInjectionActionResultDto {
    pub context: DllInjectionContextDto,
    pub notice: String,
}

/// Receiver 运行时设置 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverRuntimeSettingsDto {
    pub receiver_listen_addr: String,
    pub relay_enabled: bool,
    pub relay_target_host: String,
    pub fans_enabled: bool,
    pub fans_output_dir: String,
    pub stallion_output_enabled: bool,
    pub stallion_output_dir: String,
}

/// Receiver 运行时设置保存输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveReceiverRuntimeSettingsInput {
    pub receiver_listen_addr: String,
    pub relay_enabled: bool,
    pub relay_target_host: Option<String>,
    pub fans_enabled: bool,
    pub fans_output_dir: Option<String>,
    pub stallion_output_enabled: bool,
    pub stallion_output_dir: Option<String>,
}

/// Receiver 运行时设置动作返回
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverRuntimeSettingsActionResultDto {
    pub settings: ReceiverRuntimeSettingsDto,
    pub notice: String,
}

/// 终端页快照 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSnapshotDto {
    pub receiver_ready: bool,
    pub receiver_status: String,
    pub receiver_listen_addr: String,
    pub receiver_configured_listen_addr: String,
    pub receiver_listen_addr_source: String,
    pub logs: Vec<String>,
}

/// 游戏设置页上下文 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSettingsContextDto {
    pub path: String,
    pub has_path: bool,
    pub exists: bool,
    pub is_valid_game_dir: bool,
    pub detected_version: String,
    pub detected_version_label: String,
    pub target_fps: i32,
    pub vsync_count: i32,
}

/// 游戏设置页保存输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGameSettingsInput {
    pub path: String,
    pub target_fps: i32,
    pub vsync_count: i32,
}

/// 游戏设置页动作返回
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSettingsActionResultDto {
    pub context: GameSettingsContextDto,
    pub notice: String,
}

/// 开机自启保存输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAutostartEnabledInput {
    pub enabled: bool,
}

/// 获取启动页基础状态
#[tauri::command]
pub fn get_bootstrap_state(state: State<'_, AppState>) -> BootstrapStateDto {
    BootstrapStateDto::from_state(env!("CARGO_PKG_VERSION"), state.inner())
}

/// 获取终端页快照
#[tauri::command]
pub fn get_terminal_snapshot(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> TerminalSnapshotDto {
    let runtime = state.receiver_runtime();

    TerminalSnapshotDto {
        receiver_ready: runtime.ready,
        receiver_status: runtime.status,
        receiver_listen_addr: runtime.listen_addr,
        receiver_configured_listen_addr: runtime.configured_listen_addr,
        receiver_listen_addr_source: runtime.source.as_str().to_string(),
        logs: receiver::snapshot_logs(limit.unwrap_or(600)),
    }
}

/// 清空终端页日志
#[tauri::command]
pub fn clear_terminal_logs() {
    receiver::clear_logs();
}

/// 扫描已安装游戏
#[tauri::command]
pub fn scan_installed_games() -> Vec<DetectedGameDto> {
    scan_installed_games_core()
        .into_iter()
        .map(DetectedGameDto::from_detected_game)
        .collect()
}

/// 手动检测游戏目录
#[tauri::command]
pub fn inspect_game_dir(path: String) -> Result<InspectGameDirResultDto, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("请输入游戏目录".to_string());
    }

    let game_dir = PathBuf::from(trimmed);
    Ok(build_inspect_game_dir_result(&game_dir))
}

/// 打开目录选择框
#[tauri::command]
pub fn pick_directory(title: Option<String>) -> Option<String> {
    let dialog = rfd::FileDialog::new();
    let dialog = match title.as_deref() {
        Some(value) if !value.trim().is_empty() => dialog.set_title(value),
        _ => dialog,
    };

    dialog.pick_folder().map(|path| path.display().to_string())
}

/// 读取 DLL 注入页上下文
#[tauri::command]
pub fn get_dll_injection_context(path: Option<String>) -> DllInjectionContextDto {
    build_dll_injection_context(path.as_deref())
}

/// 读取 Receiver 运行时设置
#[tauri::command]
pub fn get_receiver_runtime_settings() -> ReceiverRuntimeSettingsDto {
    build_receiver_runtime_settings()
}

/// 保存 DLL 注入链路相关配置
#[tauri::command]
pub fn save_dll_injection_config(
    input: SaveDllInjectionConfigInput,
) -> Result<DllInjectionActionResultDto, String> {
    let game_dir = require_valid_game_dir(&input.path)?;
    let mut config = load_effective_config(Some(&game_dir));

    apply_dll_injection_fields(&mut config, &input)?;
    save_config_to_targets(&game_dir, &config)?;

    Ok(DllInjectionActionResultDto {
        context: build_dll_injection_context(Some(input.path.as_str())),
        notice: "配置已保存".to_string(),
    })
}

/// 保存 Receiver 运行时设置
#[tauri::command]
pub fn save_receiver_runtime_settings(
    state: State<'_, AppState>,
    input: SaveReceiverRuntimeSettingsInput,
) -> Result<ReceiverRuntimeSettingsActionResultDto, String> {
    let mut config = Config::load_from_exe_dir();
    apply_receiver_runtime_fields(&mut config, &input)?;
    config.save_to_exe_dir()?;
    let notice = hot_reload_receiver_runtime(state.inner());

    Ok(ReceiverRuntimeSettingsActionResultDto {
        settings: build_receiver_runtime_settings(),
        notice,
    })
}

/// 安装 DLL 并同步注入链路配置
#[tauri::command]
pub fn install_dll_injection(
    input: SaveDllInjectionConfigInput,
) -> Result<DllInjectionActionResultDto, String> {
    let game_dir = require_valid_game_dir(&input.path)?;
    let version = detect_game_version(&game_dir);

    if version == GameVersion::Unknown {
        return Err("无法识别游戏版本".to_string());
    }

    install_dll(&game_dir, version).map_err(|e| format!("安装 DLL 失败: {}", e))?;

    let mut config = load_effective_config(Some(&game_dir));
    apply_dll_injection_fields(&mut config, &input)?;

    let base_notice = if is_jp_steam_game_dir(&game_dir, version) {
        "已完成 Steam JP 代理 DLL 与 FunnyHoney 启动器部署，并已备份原始 UmamusumePrettyDerby_Jpn.exe。"
            .to_string()
    } else {
        "安装成功".to_string()
    };

    let notice = match save_config_to_targets(&game_dir, &config) {
        Ok(()) => base_notice,
        Err(error) => format!("{} 配置同步失败: {}", base_notice, error),
    };

    Ok(DllInjectionActionResultDto {
        context: build_dll_injection_context(Some(input.path.as_str())),
        notice,
    })
}

/// 保存 Debug 模式
#[tauri::command]
pub fn save_debug_mode(input: SaveDebugModeInput) -> Result<DllInjectionActionResultDto, String> {
    let game_dir = require_valid_game_dir(&input.path)?;
    let mut config = load_effective_config(Some(&game_dir));

    apply_debug_mode_field(&mut config, input.debug_mode);
    save_config_to_targets(&game_dir, &config)?;

    Ok(DllInjectionActionResultDto {
        context: build_dll_injection_context(Some(input.path.as_str())),
        notice: if input.debug_mode {
            "Debug 模式已开启".to_string()
        } else {
            "Debug 模式已关闭".to_string()
        },
    })
}

/// 卸载 DLL
#[tauri::command]
pub fn uninstall_dll_injection(path: String) -> Result<DllInjectionActionResultDto, String> {
    let game_dir = require_valid_game_dir(&path)?;
    let version = detect_game_version(&game_dir);

    if version == GameVersion::Unknown {
        return Err("无法识别游戏版本".to_string());
    }

    uninstall_dll(&game_dir, version).map_err(|e| format!("卸载 DLL 失败: {}", e))?;

    Ok(DllInjectionActionResultDto {
        context: build_dll_injection_context(Some(path.as_str())),
        notice: "已卸载".to_string(),
    })
}

/// 读取游戏设置页上下文
#[tauri::command]
pub fn get_game_settings_context(path: Option<String>) -> GameSettingsContextDto {
    build_game_settings_context(path.as_deref())
}

/// 保存游戏设置
#[tauri::command]
pub fn save_game_settings(
    input: SaveGameSettingsInput,
) -> Result<GameSettingsActionResultDto, String> {
    validate_game_settings(&input)?;

    let game_dir = resolve_game_settings_save_dir(&input.path)?;
    let mut config = load_effective_config(Some(&game_dir));
    config.target_fps = input.target_fps;
    config.vsync_count = input.vsync_count;
    save_config_to_targets(&game_dir, &config)?;
    let resolved_path = game_dir.display().to_string();

    Ok(GameSettingsActionResultDto {
        context: build_game_settings_context(Some(resolved_path.as_str())),
        notice: "游戏设置已保存".to_string(),
    })
}

/// 读取工具设置页上下文
#[tauri::command]
pub fn get_tool_settings_context(app: AppHandle) -> Result<ToolSettingsContextDto, String> {
    crate::tool_settings::get_tool_settings_context(&app, env!("CARGO_PKG_VERSION"))
}

/// 设置开机自启
#[tauri::command]
pub fn set_autostart_enabled(
    app: AppHandle,
    input: SetAutostartEnabledInput,
) -> Result<ToolSettingsActionResultDto, String> {
    crate::tool_settings::set_autostart_enabled(&app, env!("CARGO_PKG_VERSION"), input.enabled)
}

/// 检查应用更新
#[tauri::command]
pub async fn check_app_update() -> Result<AppUpdateCheckDto, String> {
    crate::tool_settings::check_app_update(env!("CARGO_PKG_VERSION")).await
}

/// 打开最新 Release 或下载页面
#[tauri::command]
pub fn open_latest_release_page(app: AppHandle, url: Option<String>) -> Result<(), String> {
    crate::tool_settings::open_latest_release_page(&app, url.as_deref())
}

fn map_game_version(version: GameVersion) -> (&'static str, &'static str) {
    match version {
        GameVersion::Steam => ("steam", "Steam 版"),
        GameVersion::DMM => ("dmm", "DMM 版"),
        GameVersion::Unknown => ("unknown", "未知版本"),
    }
}

fn map_install_status(status: InstallStatus) -> (&'static str, &'static str) {
    match status {
        InstallStatus::Installed => ("installed", "已安装"),
        InstallStatus::NotInstalled => ("notInstalled", "未安装"),
        InstallStatus::NeedsUpdate => ("needsUpdate", "需要更新"),
        InstallStatus::Unknown => ("unknown", "未知"),
    }
}

fn build_inspect_game_dir_result(game_dir: &Path) -> InspectGameDirResultDto {
    let exists = game_dir.exists();
    let is_valid = is_valid_game_dir(game_dir);

    let detected_version = if is_valid {
        detect_game_version(game_dir)
    } else {
        GameVersion::Unknown
    };

    let install_status = if is_valid {
        check_install_status(game_dir, detected_version)
    } else {
        InstallStatus::Unknown
    };

    InspectGameDirResultDto::new(game_dir, exists, is_valid, detected_version, install_status)
}

fn build_dll_injection_context(path: Option<&str>) -> DllInjectionContextDto {
    let normalized_path = path.unwrap_or_default().trim().to_string();
    let has_path = !normalized_path.is_empty();
    let game_dir = if has_path {
        Some(PathBuf::from(&normalized_path))
    } else {
        None
    };

    let inspect_game_version = game_dir
        .as_deref()
        .filter(|dir| is_valid_game_dir(dir))
        .map(detect_game_version)
        .unwrap_or(GameVersion::Unknown);

    let inspect = game_dir
        .as_deref()
        .map(build_inspect_game_dir_result)
        .unwrap_or_else(|| {
            InspectGameDirResultDto::new(
                Path::new(""),
                false,
                false,
                GameVersion::Unknown,
                InstallStatus::Unknown,
            )
        });

    let config = load_effective_config(game_dir.as_deref());
    let notifier_host = config.notifier_host.clone();
    let timeout_ms = config.timeout_ms;
    let debug_mode = config.debug_mode;
    let fans_enabled = config.fans_enabled;
    let debug_output_dir = resolve_debug_output_dir(&config);
    let fans_output_dir = resolve_fans_output_dir(&config);
    let steam_requirement_note = game_dir
        .as_deref()
        .map(|dir| build_steam_requirement_note(dir, inspect_game_version))
        .unwrap_or_default();

    DllInjectionContextDto {
        path: normalized_path,
        has_path,
        exists: inspect.exists,
        is_valid_game_dir: inspect.is_valid_game_dir,
        detected_version: inspect.detected_version,
        detected_version_label: inspect.detected_version_label,
        install_status: inspect.install_status,
        install_status_label: inspect.install_status_label,
        notifier_host,
        timeout_ms,
        debug_mode,
        debug_output_dir,
        fans_enabled,
        fans_output_dir,
        steam_requirement_note,
    }
}

fn build_steam_requirement_note(game_dir: &Path, version: GameVersion) -> String {
    if !is_jp_steam_game_dir(game_dir, version) {
        return String::new();
    }

    "Steam JP 版会部署 cri_mana_vpx.dll 代理，并同时备份原始 UmamusumePrettyDerby_Jpn.exe 后写入 FunnyHoney 启动器。游戏更新后如果原始启动器被恢复，请重新执行一次安装。".to_string()
}

fn is_jp_steam_game_dir(game_dir: &Path, version: GameVersion) -> bool {
    version == GameVersion::Steam && game_dir.join("UmamusumePrettyDerby_Jpn.exe").exists()
}

fn build_receiver_runtime_settings() -> ReceiverRuntimeSettingsDto {
    let config = Config::load_from_exe_dir();
    build_receiver_runtime_settings_from_config(&config)
}

fn build_receiver_runtime_settings_from_config(config: &Config) -> ReceiverRuntimeSettingsDto {
    ReceiverRuntimeSettingsDto {
        receiver_listen_addr: receiver::normalize_receiver_listen_addr_input(
            &config.receiver_listen_addr,
        )
        .unwrap_or_else(|| config.receiver_listen_addr.trim().to_string()),
        relay_enabled: config.relay_enabled,
        relay_target_host: resolve_relay_target_host(config),
        fans_enabled: config.fans_enabled,
        fans_output_dir: resolve_fans_output_dir(config),
        stallion_output_enabled: config.stallion_output_enabled,
        stallion_output_dir: resolve_stallion_output_dir(config),
    }
}

fn build_game_settings_context(path: Option<&str>) -> GameSettingsContextDto {
    let requested_path = path.unwrap_or_default().trim().to_string();
    let resolved_game_dir = if requested_path.is_empty() {
        resolve_default_game_dir()
    } else {
        Some(PathBuf::from(&requested_path))
    };
    let normalized_path = resolved_game_dir
        .as_ref()
        .map(|value| value.display().to_string())
        .unwrap_or(requested_path);
    let has_path = !normalized_path.is_empty();

    let inspect = resolved_game_dir
        .as_deref()
        .map(build_inspect_game_dir_result)
        .unwrap_or_else(|| {
            InspectGameDirResultDto::new(
                Path::new(""),
                false,
                false,
                GameVersion::Unknown,
                InstallStatus::Unknown,
            )
        });

    let config = load_effective_config(resolved_game_dir.as_deref());

    GameSettingsContextDto {
        path: normalized_path,
        has_path,
        exists: inspect.exists,
        is_valid_game_dir: inspect.is_valid_game_dir,
        detected_version: inspect.detected_version,
        detected_version_label: inspect.detected_version_label,
        target_fps: config.target_fps,
        vsync_count: config.vsync_count,
    }
}

fn resolve_default_game_dir() -> Option<PathBuf> {
    let detected_games = scan_installed_games_core();
    select_default_game_dir(detected_games)
}

fn select_default_game_dir(detected_games: Vec<DetectedGame>) -> Option<PathBuf> {
    let mut newest_configured_game = None;
    for game in &detected_games {
        let config_path = Config::config_path(&game.path);
        let Ok(modified) = fs::metadata(config_path).and_then(|metadata| metadata.modified())
        else {
            continue;
        };

        if newest_configured_game
            .as_ref()
            .map(|(_, current_modified)| modified > *current_modified)
            .unwrap_or(true)
        {
            newest_configured_game = Some((game.path.clone(), modified));
        }
    }

    newest_configured_game
        .map(|(path, _)| path)
        .or_else(|| detected_games.into_iter().next().map(|game| game.path))
}

fn resolve_game_settings_save_dir(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if !trimmed.is_empty() {
        return require_valid_game_dir(trimmed);
    }

    resolve_default_game_dir().ok_or_else(|| "未检测到可用的游戏目录，请先确认游戏目录".to_string())
}

fn load_effective_config(game_dir: Option<&Path>) -> Config {
    let exe_config = Config::load_from_exe_dir();
    let Some(path) = game_dir else {
        return exe_config;
    };

    if !is_valid_game_dir(path) {
        return exe_config;
    }

    let game_config_exists = Config::config_path(path).exists();
    let game_config_has_fans_enabled = Config::game_config_has_key(path, "fans_enabled");
    let mut config = Config::load_from(path);

    backfill_exe_side_receiver_fields(
        &mut config,
        exe_config,
        game_config_exists,
        game_config_has_fans_enabled,
    );

    config
}

fn backfill_exe_side_receiver_fields(
    config: &mut Config,
    exe_config: Config,
    game_config_exists: bool,
    game_config_has_fans_enabled: bool,
) {
    if !game_config_exists || !game_config_has_fans_enabled {
        config.fans_enabled = exe_config.fans_enabled;
    }

    if config
        .fans_output_dir
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        config.fans_output_dir = exe_config.fans_output_dir;
    }

    config.stallion_output_enabled = exe_config.stallion_output_enabled;
    config.stallion_output_dir = exe_config.stallion_output_dir;
}

fn apply_dll_injection_fields(
    config: &mut Config,
    input: &SaveDllInjectionConfigInput,
) -> Result<(), String> {
    let notifier_host = input.notifier_host.trim();
    if !notifier_host.starts_with("http://") && !notifier_host.starts_with("https://") {
        return Err("转发地址必须以 http:// 或 https:// 开头".to_string());
    }

    if input.timeout_ms == 0 {
        return Err("超时必须大于 0".to_string());
    }

    config.notifier_host = notifier_host.to_string();
    config.timeout_ms = input.timeout_ms;
    if let Some(fans_enabled) = input.fans_enabled {
        config.fans_enabled = fans_enabled;
    }

    if let Some(fans_output_dir) = input.fans_output_dir.as_deref() {
        let fans_output_dir = fans_output_dir.trim().to_string();
        config.fans_output_dir = Some(if fans_output_dir.is_empty() {
            resolve_default_fans_output_dir().display().to_string()
        } else {
            fans_output_dir
        });
    }

    Ok(())
}

fn apply_receiver_runtime_fields(
    config: &mut Config,
    input: &SaveReceiverRuntimeSettingsInput,
) -> Result<(), String> {
    let receiver_listen_addr = input.receiver_listen_addr.trim();
    if receiver_listen_addr.is_empty() {
        return Err("监听地址不能为空".to_string());
    }

    let Some(receiver_listen_addr) =
        receiver::normalize_receiver_listen_addr_input(receiver_listen_addr)
    else {
        return Err(
            "监听地址必须使用 host:port 格式，host 仅支持 localhost 或 IP；也可填写 http://host:port"
                .to_string(),
        );
    };

    let relay_target_host = normalize_optional_input(input.relay_target_host.as_deref());
    if let Some(target) = relay_target_host.as_deref() {
        if !target.starts_with("http://") && !target.starts_with("https://") {
            return Err("Relay 目标地址必须以 http:// 或 https:// 开头".to_string());
        }

        if input.relay_enabled
            && receiver_pipeline::relay_target_would_loop(&receiver_listen_addr, target)
        {
            return Err("Relay 目标地址不能指向当前 Receiver 自身".to_string());
        }
    }

    let fans_output_dir = normalize_optional_input(input.fans_output_dir.as_deref());

    config.receiver_listen_addr = receiver_listen_addr.to_string();
    config.relay_enabled = input.relay_enabled;
    config.relay_target_host = relay_target_host;
    config.fans_enabled = input.fans_enabled;
    config.fans_output_dir = Some(
        fans_output_dir.unwrap_or_else(|| resolve_default_fans_output_dir().display().to_string()),
    );

    let stallion_output_dir = normalize_optional_input(input.stallion_output_dir.as_deref());
    config.stallion_output_enabled = input.stallion_output_enabled;
    config.stallion_output_dir = Some(stallion_output_dir.unwrap_or_else(|| {
        guga_ura_config_core::stallion_output::default_stallion_output_dir()
            .display()
            .to_string()
    }));

    Ok(())
}

fn hot_reload_receiver_runtime(state: &AppState) -> String {
    let current_runtime = state.receiver_runtime();
    let target_resolution = receiver::resolve_receiver_listen_addr(None);
    let should_restart =
        !current_runtime.ready || current_runtime.listen_addr != target_resolution.listen_addr;

    if !should_restart {
        let mut updated_runtime = current_runtime;
        updated_runtime.configured_listen_addr = target_resolution.configured_listen_addr;
        updated_runtime.source = target_resolution.source;
        state.update_receiver_runtime(updated_runtime);
        return "Receiver 设置已保存；监听地址保持不变，后续请求将直接使用新配置".to_string();
    }

    let next_receiver = receiver::start_embedded_receiver_with_resolution(target_resolution);
    if next_receiver.runtime.ready {
        let old_handle = state.replace_receiver(next_receiver);
        if let Some(handle) = old_handle {
            handle.stop();
        }

        let runtime = state.receiver_runtime();
        return format!(
            "Receiver 设置已保存；监听地址已热更新到 {}",
            runtime.listen_addr
        );
    }

    if current_runtime.ready {
        let mut preserved_runtime = current_runtime.clone();
        preserved_runtime.configured_listen_addr =
            next_receiver.runtime.configured_listen_addr.clone();
        preserved_runtime.source = next_receiver.runtime.source;
        state.update_receiver_runtime(preserved_runtime);
        return format!(
            "Receiver 设置已保存，但热更新失败（{}）；当前仍监听 {}，重启后会按新配置接管",
            next_receiver.runtime.status, current_runtime.listen_addr
        );
    }

    state.update_receiver_runtime(next_receiver.runtime.clone());
    format!(
        "Receiver 设置已保存，但当前 Receiver 尚未启动成功（{}）",
        next_receiver.runtime.status
    )
}

fn apply_debug_mode_field(config: &mut Config, debug_mode: bool) {
    config.debug_mode = debug_mode;
    config.debug_output_dir = Some(resolve_default_debug_output_dir().display().to_string());
}

fn save_config_to_targets(game_dir: &Path, config: &Config) -> Result<(), String> {
    config.save_to(game_dir)?;
    config.save_to_exe_dir()?;
    Ok(())
}

fn validate_game_settings(input: &SaveGameSettingsInput) -> Result<(), String> {
    if input.target_fps != -1 && input.target_fps <= 0 {
        return Err("目标 FPS 必须是默认值或正整数".to_string());
    }

    if !matches!(input.vsync_count, -1 | 0 | 1) {
        return Err("VSync 取值必须是默认、关闭或开启".to_string());
    }

    Ok(())
}

fn require_valid_game_dir(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("请先选择游戏目录".to_string());
    }

    let game_dir = PathBuf::from(trimmed);
    if !is_valid_game_dir(&game_dir) {
        return Err("请先检测有效的游戏目录".to_string());
    }

    Ok(game_dir)
}

fn resolve_default_debug_output_dir() -> PathBuf {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        return exe_path.join("debug");
    }

    PathBuf::from("debug")
}

fn resolve_default_fans_output_dir() -> PathBuf {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        return exe_path.join("fans");
    }

    PathBuf::from("fans")
}

fn resolve_debug_output_dir(config: &Config) -> String {
    config
        .debug_output_dir
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| resolve_default_debug_output_dir().display().to_string())
}

fn resolve_fans_output_dir(config: &Config) -> String {
    config
        .fans_output_dir
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| resolve_default_fans_output_dir().display().to_string())
}

fn resolve_relay_target_host(config: &Config) -> String {
    config
        .relay_target_host
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default()
}

fn resolve_stallion_output_dir(config: &Config) -> String {
    config
        .stallion_output_dir
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| {
            guga_ura_config_core::stallion_output::default_stallion_output_dir()
                .display()
                .to_string()
        })
}

fn normalize_optional_input(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_receiver_runtime_fields, backfill_exe_side_receiver_fields,
        build_receiver_runtime_settings_from_config, select_default_game_dir,
        SaveReceiverRuntimeSettingsInput,
    };
    use guga_ura_config_core::config::Config;
    use guga_ura_config_core::detector::{DetectedGame, GameVersion};
    use std::fs;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn apply_receiver_runtime_fields_should_update_exe_side_settings() {
        let mut config = Config::default();
        let input = SaveReceiverRuntimeSettingsInput {
            receiver_listen_addr: "http://127.0.0.1:4700/runtime".to_string(),
            relay_enabled: true,
            relay_target_host: Some("http://127.0.0.1:4800".to_string()),
            fans_enabled: false,
            fans_output_dir: Some("C:\\temp\\fans".to_string()),
            stallion_output_enabled: true,
            stallion_output_dir: Some("C:\\temp\\stallion".to_string()),
        };

        apply_receiver_runtime_fields(&mut config, &input).expect("Receiver 设置保存失败");

        assert_eq!(config.receiver_listen_addr, "127.0.0.1:4700");
        assert!(config.relay_enabled);
        assert_eq!(
            config.relay_target_host.as_deref(),
            Some("http://127.0.0.1:4800")
        );
        assert!(!config.fans_enabled);
        assert_eq!(config.fans_output_dir.as_deref(), Some("C:\\temp\\fans"));
        assert!(config.stallion_output_enabled);
        assert_eq!(
            config.stallion_output_dir.as_deref(),
            Some("C:\\temp\\stallion")
        );
    }

    #[test]
    fn backfill_exe_side_receiver_fields_should_preserve_exe_stallion_runtime_settings() {
        let mut game_config = Config {
            fans_enabled: true,
            fans_output_dir: Some("C:\\game\\fans".to_string()),
            stallion_output_enabled: true,
            stallion_output_dir: Some("C:\\game\\stallion".to_string()),
            ..Config::default()
        };
        let exe_config = Config {
            fans_enabled: false,
            fans_output_dir: Some("C:\\exe\\fans".to_string()),
            stallion_output_enabled: false,
            stallion_output_dir: Some("C:\\exe\\stallion".to_string()),
            ..Config::default()
        };

        backfill_exe_side_receiver_fields(&mut game_config, exe_config, true, true);

        assert!(game_config.fans_enabled);
        assert_eq!(
            game_config.fans_output_dir.as_deref(),
            Some("C:\\game\\fans")
        );
        assert!(!game_config.stallion_output_enabled);
        assert_eq!(
            game_config.stallion_output_dir.as_deref(),
            Some("C:\\exe\\stallion")
        );
    }

    #[test]
    fn backfill_exe_side_receiver_fields_should_use_exe_stallion_defaults() {
        let mut game_config = Config {
            stallion_output_enabled: true,
            stallion_output_dir: Some("C:\\game\\stallion".to_string()),
            ..Config::default()
        };
        let exe_config = Config {
            stallion_output_enabled: false,
            stallion_output_dir: None,
            ..Config::default()
        };

        backfill_exe_side_receiver_fields(&mut game_config, exe_config, true, true);

        assert!(!game_config.stallion_output_enabled);
        assert_eq!(game_config.stallion_output_dir, None);
    }

    #[test]
    fn apply_receiver_runtime_fields_should_reject_invalid_inputs() {
        let mut config = Config::default();
        let invalid_listen = SaveReceiverRuntimeSettingsInput {
            receiver_listen_addr: "ftp://127.0.0.1:4700".to_string(),
            relay_enabled: false,
            relay_target_host: None,
            fans_enabled: true,
            fans_output_dir: None,
            stallion_output_enabled: true,
            stallion_output_dir: None,
        };

        let listen_error = apply_receiver_runtime_fields(&mut config, &invalid_listen)
            .expect_err("应拒绝非法监听地址");
        assert!(listen_error.contains("localhost 或 IP"));

        let invalid_relay = SaveReceiverRuntimeSettingsInput {
            receiver_listen_addr: "127.0.0.1:4700".to_string(),
            relay_enabled: true,
            relay_target_host: Some("127.0.0.1:4800".to_string()),
            fans_enabled: true,
            fans_output_dir: None,
            stallion_output_enabled: true,
            stallion_output_dir: None,
        };

        let relay_error = apply_receiver_runtime_fields(&mut config, &invalid_relay)
            .expect_err("应拒绝非法 relay 地址");
        assert!(relay_error.contains("http://"));

        let self_loop_relay = SaveReceiverRuntimeSettingsInput {
            receiver_listen_addr: "127.0.0.1:4700".to_string(),
            relay_enabled: true,
            relay_target_host: Some("http://localhost:4700/api".to_string()),
            fans_enabled: true,
            fans_output_dir: None,
            stallion_output_enabled: true,
            stallion_output_dir: None,
        };

        let self_loop_error = apply_receiver_runtime_fields(&mut config, &self_loop_relay)
            .expect_err("应拒绝自环 relay 地址");
        assert!(self_loop_error.contains("当前 Receiver 自身"));
    }

    #[test]
    fn build_receiver_runtime_settings_should_fall_back_to_defaults() {
        let config = Config::default();

        let settings = build_receiver_runtime_settings_from_config(&config);

        assert_eq!(settings.receiver_listen_addr, "127.0.0.1:4693");
        assert!(!settings.relay_enabled);
        assert_eq!(settings.relay_target_host, "");
        assert!(settings.fans_enabled);
        assert!(!settings.fans_output_dir.is_empty());
    }

    #[test]
    fn select_default_game_dir_should_prefer_recent_config_file() {
        let root = std::env::temp_dir().join(format!(
            "gugaura_commands_default_dir_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let steam_dir = root.join("steam");
        let dmm_dir = root.join("dmm");
        fs::create_dir_all(&steam_dir).expect("创建 Steam 临时目录失败");
        fs::create_dir_all(&dmm_dir).expect("创建 DMM 临时目录失败");

        Config::default()
            .save_to(&steam_dir)
            .expect("写入 Steam 配置失败");
        thread::sleep(Duration::from_millis(20));
        Config::default()
            .save_to(&dmm_dir)
            .expect("写入 DMM 配置失败");

        let selected = select_default_game_dir(vec![
            DetectedGame {
                path: steam_dir.clone(),
                version: GameVersion::Steam,
            },
            DetectedGame {
                path: dmm_dir.clone(),
                version: GameVersion::DMM,
            },
        ])
        .expect("应选中默认游戏目录");

        assert_eq!(selected, dmm_dir);
        let _ = fs::remove_dir_all(root);
    }
}

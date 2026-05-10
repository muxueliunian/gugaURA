//! 工具设置能力

use reqwest::blocking::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_opener::OpenerExt;

const GITHUB_LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/muxueliunian/gugaURA/releases/latest";
const GITHUB_RELEASES_PAGE: &str = "https://github.com/muxueliunian/gugaURA/releases/latest";
const GITHUB_RELEASES_URL_PREFIX: &str = "https://github.com/muxueliunian/gugaURA/releases/";
const APP_USER_AGENT: &str = "gugaURA-config-tool";
const PREFERRED_INSTALLER_NAME: &str = "gugaURA_installer.exe";

/// 工具设置上下文 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSettingsContextDto {
    pub current_version: String,
    pub autostart_enabled: bool,
}

/// 工具设置动作返回 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSettingsActionResultDto {
    pub context: ToolSettingsContextDto,
    pub notice: String,
}

/// 应用更新检查 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateCheckDto {
    pub current_version: String,
    pub latest_version: String,
    pub version_status: String,
    pub has_update: bool,
    pub release_page_url: String,
    pub download_asset_url: Option<String>,
    pub published_at: String,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseResponse {
    tag_name: String,
    html_url: String,
    published_at: Option<String>,
    body: Option<String>,
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

/// 读取工具设置上下文
pub fn get_tool_settings_context(
    app: &AppHandle,
    current_version: &str,
) -> Result<ToolSettingsContextDto, String> {
    let autostart_enabled = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| format!("读取开机自启状态失败: {}", error))?;

    Ok(ToolSettingsContextDto {
        current_version: current_version.to_string(),
        autostart_enabled,
    })
}

/// 设置开机自启
pub fn set_autostart_enabled(
    app: &AppHandle,
    current_version: &str,
    enabled: bool,
) -> Result<ToolSettingsActionResultDto, String> {
    let autostart = app.autolaunch();
    if enabled {
        autostart
            .enable()
            .map_err(|error| format!("开启开机自启失败: {}", error))?;
    } else {
        autostart
            .disable()
            .map_err(|error| format!("关闭开机自启失败: {}", error))?;
    }

    Ok(ToolSettingsActionResultDto {
        context: get_tool_settings_context(app, current_version)?,
        notice: if enabled {
            "开机自启已开启".to_string()
        } else {
            "开机自启已关闭".to_string()
        },
    })
}

/// 检查应用更新
pub async fn check_app_update(current_version: &'static str) -> Result<AppUpdateCheckDto, String> {
    tauri::async_runtime::spawn_blocking(move || check_app_update_blocking(current_version))
        .await
        .map_err(|error| format!("检查更新失败: {}", error))?
}

/// 打开最新 Release 或下载页面
pub fn open_latest_release_page(app: &AppHandle, target_url: Option<&str>) -> Result<(), String> {
    let url = resolve_open_url(target_url)?;
    app.opener()
        .open_url(url, None::<String>)
        .map_err(|error| format!("打开系统浏览器失败: {}", error))
}

fn check_app_update_blocking(current_version: &str) -> Result<AppUpdateCheckDto, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|error| format!("初始化网络请求失败: {}", error))?;

    let response = client
        .get(GITHUB_LATEST_RELEASE_API)
        .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .map_err(|error| format!("连接 GitHub Release 服务失败: {}", error))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub Release 服务返回异常状态: {}",
            response.status()
        ));
    }

    let release = response
        .json::<GitHubReleaseResponse>()
        .map_err(|error| format!("解析 GitHub Release 信息失败: {}", error))?;

    let current = parse_version(current_version, "当前版本")?;
    let latest = parse_version(&release.tag_name, "最新版本")?;
    let version_status = resolve_version_status(&current, &latest);
    let download_asset_url = select_download_asset_url(&release.assets);

    Ok(AppUpdateCheckDto {
        current_version: current_version.to_string(),
        latest_version: release.tag_name,
        version_status: version_status.to_string(),
        has_update: latest > current,
        release_page_url: release.html_url,
        download_asset_url,
        published_at: release.published_at.unwrap_or_default(),
        summary: summarize_release_body(release.body.as_deref()),
    })
}

fn parse_version(value: &str, label: &str) -> Result<Version, String> {
    let normalized = value.trim().trim_start_matches(['v', 'V']);
    Version::parse(normalized).map_err(|_| format!("{}格式无法识别: {}", label, value.trim()))
}

fn resolve_version_status(current: &Version, latest: &Version) -> &'static str {
    if latest > current {
        return "updateAvailable";
    }

    if latest == current {
        return "latest";
    }

    "ahead"
}

fn select_download_asset_url(assets: &[GitHubReleaseAsset]) -> Option<String> {
    assets
        .iter()
        .find(|asset| asset.name.eq_ignore_ascii_case(PREFERRED_INSTALLER_NAME))
        .or_else(|| {
            assets
                .iter()
                .find(|asset| asset.name.to_ascii_lowercase().ends_with(".exe"))
        })
        .map(|asset| asset.browser_download_url.clone())
}

fn summarize_release_body(body: Option<&str>) -> String {
    let Some(body) = body else {
        return "该版本未提供更新说明。".to_string();
    };

    let summary = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(4)
        .collect::<Vec<_>>()
        .join("\n");

    if summary.is_empty() {
        return "该版本未提供更新说明。".to_string();
    }

    summary.chars().take(360).collect()
}

fn resolve_open_url(target_url: Option<&str>) -> Result<String, String> {
    let value = target_url
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .unwrap_or(GITHUB_RELEASES_PAGE);

    if !value.starts_with(GITHUB_RELEASES_URL_PREFIX) {
        return Err("只能打开 gugaURA 的 GitHub Release 页面".to_string());
    }

    Ok(value.to_string())
}

//! 游戏版本检测

use std::path::{Path, PathBuf};

/// 游戏版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameVersion {
    Steam,
    DMM,
    Unknown,
}

impl GameVersion {
    pub fn display_name(&self) -> &'static str {
        match self {
            GameVersion::Steam => "Steam 版",
            GameVersion::DMM => "DMM 版",
            GameVersion::Unknown => "未知版本",
        }
    }

    /// 获取需要代理的 DLL 名称
    pub fn proxy_dll_name(&self) -> &'static str {
        match self {
            GameVersion::Steam => "cri_mana_vpx.dll",
            GameVersion::DMM => "UnityPlayer.dll",
            GameVersion::Unknown => "",
        }
    }

    /// 获取原始 DLL 备份名称
    pub fn backup_dll_name(&self) -> &'static str {
        match self {
            GameVersion::Steam => "cri_mana_vpx_orig.dll",
            GameVersion::DMM => "UnityPlayer_orig.dll",
            GameVersion::Unknown => "",
        }
    }
}

/// 检测到的游戏安装信息
#[derive(Debug, Clone)]
pub struct DetectedGame {
    pub path: PathBuf,
    pub version: GameVersion,
}

impl DetectedGame {
    pub fn display_string(&self) -> String {
        format!("[{}] {}", self.version.display_name(), self.path.display())
    }
}

/// 游戏可执行文件可能的名称
const GAME_EXE_NAMES: &[&str] = &[
    "umamusume.exe",
    "UmamusumePrettyDerby_Jpn.exe",
    "UmamusumePrettyDerby.exe",
];

const DMM_SCAN_MAX_DEPTH: usize = 3;

const DMM_GAME_DIR_NAMES: &[&str] = &[
    "Umapyoi",
    "umamusume",
    "Umamusume",
    "Umamusume Pretty Derby",
    "UmamusumePrettyDerby",
];

/// 检测游戏版本
pub fn detect_game_version(game_dir: &Path) -> GameVersion {
    // 检查是否是有效的游戏目录
    if !is_valid_game_dir(game_dir) {
        return GameVersion::Unknown;
    }

    // Steam 版特征：存在 steam_api64.dll
    if game_dir.join("steam_api64.dll").exists() {
        return GameVersion::Steam;
    }

    // 检查路径是否包含 Steam 特征
    let path_str = game_dir.to_string_lossy().to_lowercase();
    if path_str.contains("steamapps") || path_str.contains("steamlibrary") {
        return GameVersion::Steam;
    }

    // 检查可执行文件名特征
    for exe_name in GAME_EXE_NAMES {
        if exe_name.to_lowercase().contains("jpn") && game_dir.join(exe_name).exists() {
            return GameVersion::Steam;
        }
    }

    // 默认认为是 DMM 版
    GameVersion::DMM
}

/// 验证游戏目录是否有效
pub fn is_valid_game_dir(game_dir: &Path) -> bool {
    if !game_dir.exists() || !game_dir.is_dir() {
        return false;
    }

    // 检查任一已知的可执行文件是否存在
    for exe_name in GAME_EXE_NAMES {
        if game_dir.join(exe_name).exists() {
            return true;
        }
    }

    // 额外检查：遍历目录查找包含 umamusume 的可执行文件
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.filter_map(Result::ok) {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            if file_name.contains("umamusume") && file_name.ends_with(".exe") {
                return true;
            }
        }
    }

    false
}

/// 辅助函数：添加游戏到列表（去重）
fn add_game_if_new(games: &mut Vec<DetectedGame>, path: PathBuf, version: GameVersion) {
    // 规范化路径用于比较
    let normalized = normalize_path_for_compare(&path);
    let is_duplicate = games
        .iter()
        .any(|g: &DetectedGame| normalize_path_for_compare(&g.path) == normalized);

    if !is_duplicate && is_valid_game_dir(&path) {
        games.push(DetectedGame { path, version });
    }
}

fn normalize_path_for_compare(path: &Path) -> String {
    path.to_string_lossy().to_lowercase().replace("/", "\\")
}

fn dmm_keyword_matches(text: &str) -> bool {
    let text = text.to_lowercase();
    text.contains("umamusume")
        || text.contains("uma musume")
        || text.contains("pretty derby")
        || text.contains("umapyoi")
        || text.contains("ウマ娘")
        || text.contains("プリティーダービー")
}

/// 自动扫描并检测所有已安装的游戏
pub fn scan_installed_games() -> Vec<DetectedGame> {
    let mut games = Vec::new();

    // 扫描 Steam 安装路径
    if let Some(steam_games) = scan_steam_games() {
        for game in steam_games {
            add_game_if_new(&mut games, game.path, game.version);
        }
    }

    // 扫描 DMM 和注册表路径
    if let Some(dmm_games) = scan_dmm_games() {
        for game in dmm_games {
            add_game_if_new(&mut games, game.path, game.version);
        }
    }

    // 扫描其他常见路径
    let common_paths = [
        "C:\\Games\\umamusume",
        "D:\\Games\\umamusume",
        "E:\\Games\\umamusume",
        "G:\\Games\\umamusume",
        "C:\\Program Files\\umamusume",
        "D:\\Program Files\\umamusume",
    ];

    for path in common_paths {
        let path = PathBuf::from(path);
        if is_valid_game_dir(&path) {
            let version = detect_game_version(&path);
            add_game_if_new(&mut games, path, version);
        }
    }

    games
}

/// Steam 游戏可能的目录名列表
const STEAM_GAME_DIRS: &[&str] = &[
    "Umamusume Pretty Derby",
    "UmamusumePrettyDerby",
    "UmamusumePrettyDerby_Jpn",
    "umamusume",
];

/// 在指定的 Steam 库路径中查找游戏
fn find_game_in_steam_library(lib_path: &Path, games: &mut Vec<DetectedGame>) {
    let common_dir = lib_path.join("steamapps").join("common");
    if !common_dir.exists() {
        return;
    }

    // 首先尝试已知的目录名
    for dir_name in STEAM_GAME_DIRS {
        let game_path = common_dir.join(dir_name);
        if is_valid_game_dir(&game_path) {
            if !games.iter().any(|g: &DetectedGame| g.path == game_path) {
                games.push(DetectedGame {
                    path: game_path,
                    version: GameVersion::Steam,
                });
            }
            return;
        }
    }

    // 如果已知目录名都没找到，遍历 common 目录查找
    if let Ok(entries) = std::fs::read_dir(&common_dir) {
        for entry in entries.filter_map(Result::ok) {
            let dir_name = entry.file_name().to_string_lossy().to_lowercase();
            // 匹配包含 "umamusume" 或 "uma" + "derby" 的目录
            if dir_name.contains("umamusume")
                || (dir_name.contains("uma") && dir_name.contains("derby"))
            {
                let game_path = entry.path();
                if is_valid_game_dir(&game_path) {
                    if !games.iter().any(|g: &DetectedGame| g.path == game_path) {
                        games.push(DetectedGame {
                            path: game_path,
                            version: GameVersion::Steam,
                        });
                    }
                }
            }
        }
    }
}

/// 扫描 Steam 游戏库
fn scan_steam_games() -> Option<Vec<DetectedGame>> {
    let mut games = Vec::new();
    let mut library_paths: Vec<PathBuf> = Vec::new();

    // 从注册表获取 Steam 安装路径
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // 尝试 64 位和 32 位路径
        let steam_paths = [
            hklm.open_subkey(r"SOFTWARE\Valve\Steam"),
            hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam"),
            hkcu.open_subkey(r"SOFTWARE\Valve\Steam"),
        ];

        for result in steam_paths {
            if let Ok(key) = result {
                let path: String = match key.get_value("InstallPath") {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let steam_path = PathBuf::from(&path);

                // 添加默认库路径
                library_paths.push(steam_path.clone());

                // 解析 libraryfolders.vdf 获取其他库路径
                let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");
                if let Ok(content) = std::fs::read_to_string(&vdf_path) {
                    for line in content.lines() {
                        // 解析 "path" 行
                        if line.contains("\"path\"") {
                            // 格式: "path"		"G:\\SteamLibrary"
                            let parts: Vec<&str> = line.split('"').collect();
                            if parts.len() >= 4 {
                                let lib_path = parts[3].replace("\\\\", "\\");
                                let lib_pathbuf = PathBuf::from(&lib_path);
                                if lib_pathbuf.exists() && !library_paths.contains(&lib_pathbuf) {
                                    library_paths.push(lib_pathbuf);
                                }
                            }
                        }
                    }
                }

                break;
            }
        }
    }

    // 在所有库路径中查找游戏
    for lib_path in library_paths {
        find_game_in_steam_library(&lib_path, &mut games);
    }

    // 额外扫描常见 Steam 库位置
    let common_steam_libs = [
        r"C:\SteamLibrary",
        r"D:\SteamLibrary",
        r"E:\SteamLibrary",
        r"F:\SteamLibrary",
        r"G:\SteamLibrary",
    ];

    for lib in common_steam_libs {
        let lib_path = PathBuf::from(lib);
        if lib_path.exists() {
            find_game_in_steam_library(&lib_path, &mut games);
        }
    }

    if games.is_empty() {
        None
    } else {
        Some(games)
    }
}

/// 扫描 DMM 游戏路径（也会扫描所有注册表安装记录）
fn scan_dmm_games() -> Option<Vec<DetectedGame>> {
    let mut games = Vec::new();

    scan_dmm_game_player_config(&mut games);

    #[cfg(windows)]
    {
        scan_dmm_uninstall_registry(&mut games);
    }

    scan_common_dmm_paths(&mut games);

    if games.is_empty() {
        None
    } else {
        Some(games)
    }
}

fn scan_dmm_game_player_config(games: &mut Vec<DetectedGame>) {
    for config_path in dmm_game_player_config_paths() {
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };

        for candidate in collect_dmm_config_candidates(&config) {
            add_dmm_candidate_path(games, candidate);
        }
    }
}

fn dmm_game_player_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for env_name in ["APPDATA", "LOCALAPPDATA"] {
        let Some(base) = std::env::var_os(env_name) else {
            continue;
        };
        let base = PathBuf::from(base);
        paths.push(base.join("dmmgameplayer5").join("dmmgame.cnf"));
        paths.push(base.join("dmmgameplayer").join("dmmgame.cnf"));
    }

    paths
}

fn collect_dmm_config_candidates(config: &serde_json::Value) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(default_install_dir) = config.get("defaultInstallDir").and_then(|v| v.as_str()) {
        push_path_candidate(&mut paths, default_install_dir);
    }

    let Some(contents) = config.get("contents").and_then(|v| v.as_array()) else {
        return paths;
    };

    for item in contents {
        if !dmm_config_item_matches_umamusume(item) {
            continue;
        }
        if !dmm_config_item_is_installed(item) {
            continue;
        }

        for field in ["path", "installPath", "gamePath", "directory"] {
            if let Some(path) = item.get(field).and_then(|v| v.as_str()) {
                push_path_candidate(&mut paths, path);
            }
        }

        if let Some(detail) = item.get("detail") {
            for field in ["path", "installPath", "gamePath", "directory"] {
                if let Some(path) = detail.get(field).and_then(|v| v.as_str()) {
                    push_path_candidate(&mut paths, path);
                }
            }
        }
    }

    paths
}

fn dmm_config_item_matches_umamusume(item: &serde_json::Value) -> bool {
    for field in ["productId", "product_id", "name", "title"] {
        if item
            .get(field)
            .and_then(|v| v.as_str())
            .is_some_and(dmm_keyword_matches)
        {
            return true;
        }
    }

    if let Some(detail) = item.get("detail") {
        for field in ["path", "installPath", "gamePath", "directory"] {
            if detail
                .get(field)
                .and_then(|v| v.as_str())
                .is_some_and(dmm_keyword_matches)
            {
                return true;
            }
        }
    }

    false
}

fn dmm_config_item_is_installed(item: &serde_json::Value) -> bool {
    if let Some(installed) = item.get("installed").and_then(|v| v.as_bool()) {
        return installed;
    }
    if let Some(installed) = item
        .get("detail")
        .and_then(|detail| detail.get("installed"))
        .and_then(|v| v.as_bool())
    {
        return installed;
    }

    true
}

#[cfg(windows)]
fn scan_dmm_uninstall_registry(games: &mut Vec<DetectedGame>) {
    use winreg::enums::*;
    use winreg::RegKey;

    let uninstall_paths = [
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegKey::predef(HKEY_CURRENT_USER),
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegKey::predef(HKEY_CURRENT_USER),
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    for (root, uninstall_path) in uninstall_paths {
        let Ok(uninstall_key) = root.open_subkey(uninstall_path) else {
            continue;
        };

        for key_name in uninstall_key.enum_keys().filter_map(Result::ok) {
            let Ok(app_key) = uninstall_key.open_subkey(&key_name) else {
                continue;
            };

            let display_name = registry_string_value(&app_key, "DisplayName");
            let publisher = registry_string_value(&app_key, "Publisher");
            let path_values = registry_path_values(&app_key);

            let matched = dmm_keyword_matches(&key_name)
                || display_name.as_deref().is_some_and(dmm_keyword_matches)
                || publisher.as_deref().is_some_and(dmm_keyword_matches)
                || path_values.iter().any(|value| dmm_keyword_matches(value));

            if !matched {
                continue;
            }

            for value in path_values {
                for candidate in extract_registry_path_candidates(&value) {
                    add_dmm_candidate_path(games, candidate);
                }
            }
        }
    }
}

#[cfg(windows)]
fn registry_string_value(key: &winreg::RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(windows)]
fn registry_path_values(key: &winreg::RegKey) -> Vec<String> {
    let mut values = Vec::new();
    for name in [
        "InstallLocation",
        "DisplayIcon",
        "UninstallString",
        "QuietUninstallString",
    ] {
        let Some(value) = registry_string_value(key, name) else {
            continue;
        };
        values.push(value);
    }
    values
}

fn extract_registry_path_candidates(value: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let Some(path_text) = clean_registry_path_text(value) else {
        return paths;
    };

    push_path_candidate(&mut paths, &path_text);

    let lower = path_text.to_lowercase();
    if lower.ends_with(".exe") || lower.ends_with(".msi") {
        push_parent_path_candidate(&mut paths, &path_text);
    }

    paths
}

fn clean_registry_path_text(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('\0').trim();
    if value.is_empty() {
        return None;
    }

    let mut path_text = if let Some(rest) = value.strip_prefix('"') {
        let Some(end) = rest.find('"') else {
            return None;
        };
        rest[..end].to_string()
    } else {
        truncate_registry_command_path(value)
    };

    if let Some((path, suffix)) = path_text.rsplit_once(',') {
        if suffix.chars().all(|c| c.is_ascii_digit()) {
            path_text = path.trim().to_string();
        }
    }

    path_text = expand_leading_env_var(path_text.trim());
    if path_text.is_empty() {
        None
    } else {
        Some(path_text)
    }
}

fn truncate_registry_command_path(value: &str) -> String {
    let lower = value.to_lowercase();
    for marker in [".exe", ".msi"] {
        if let Some(index) = lower.find(marker) {
            return value[..index + marker.len()].trim().to_string();
        }
    }

    value.trim().to_string()
}

fn expand_leading_env_var(path: &str) -> String {
    let Some(rest) = path.strip_prefix('%') else {
        return path.to_string();
    };
    let Some(end) = rest.find('%') else {
        return path.to_string();
    };

    let env_name = &rest[..end];
    let Ok(value) = std::env::var(env_name) else {
        return path.to_string();
    };

    format!("{}{}", value, &rest[end + 1..])
}

fn scan_common_dmm_paths(games: &mut Vec<DetectedGame>) {
    for drive in ["C", "D", "E", "F", "G"] {
        for root_name in ["DMMGames", "dmm", "DMM"] {
            let root = PathBuf::from(format!(r"{drive}:\{root_name}"));
            for dir_name in DMM_GAME_DIR_NAMES {
                add_game_if_new(games, root.join(dir_name), GameVersion::DMM);
            }
            scan_game_dirs_under(&root, games, GameVersion::DMM, DMM_SCAN_MAX_DEPTH);
        }
    }

    for root in [
        r"C:\Program Files\DMMGamePlayer",
        r"C:\Program Files (x86)\DMMGamePlayer",
        r"C:\Program Files\DMM GAMES",
        r"C:\Program Files (x86)\DMM GAMES",
    ] {
        scan_game_dirs_under(
            &PathBuf::from(root),
            games,
            GameVersion::DMM,
            DMM_SCAN_MAX_DEPTH,
        );
    }
}

fn add_dmm_candidate_path(games: &mut Vec<DetectedGame>, candidate: PathBuf) {
    if is_valid_game_dir(&candidate) {
        add_game_if_new(games, candidate, GameVersion::DMM);
        return;
    }

    if candidate.is_file() {
        if let Some(parent) = candidate.parent() {
            add_dmm_candidate_path(games, parent.to_path_buf());
        }
        return;
    }

    scan_game_dirs_under(&candidate, games, GameVersion::DMM, DMM_SCAN_MAX_DEPTH);
}

fn scan_game_dirs_under(
    root: &Path,
    games: &mut Vec<DetectedGame>,
    version: GameVersion,
    max_depth: usize,
) {
    if !root.exists() || !root.is_dir() {
        return;
    }

    if is_valid_game_dir(root) {
        add_game_if_new(games, root.to_path_buf(), version);
        return;
    }

    if max_depth == 0 {
        return;
    }

    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        scan_game_dirs_under(&entry.path(), games, version, max_depth - 1);
    }
}

fn push_path_candidate(paths: &mut Vec<PathBuf>, path: &str) {
    let path = path.trim();
    if path.is_empty() {
        return;
    }

    let candidate = PathBuf::from(path);
    let normalized = normalize_path_for_compare(&candidate);
    if paths
        .iter()
        .any(|existing| normalize_path_for_compare(existing) == normalized)
    {
        return;
    }

    paths.push(candidate);
}

fn push_parent_path_candidate(paths: &mut Vec<PathBuf>, path: &str) {
    let candidate = PathBuf::from(path);
    if let Some(parent) = candidate.parent() {
        if !parent.as_os_str().is_empty() {
            push_path_candidate(paths, &parent.to_string_lossy());
            return;
        }
    }

    if let Some(index) = path.rfind(['\\', '/']) {
        push_path_candidate(paths, &path[..index]);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_dmm_config_candidates, detect_game_version, extract_registry_path_candidates,
        is_valid_game_dir, scan_game_dirs_under, DetectedGame, GameVersion,
    };
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn is_valid_game_dir_should_accept_known_exe_name() {
        let dir = tempdir().expect("创建临时目录失败");
        fs::write(dir.path().join("umamusume.exe"), []).expect("创建 exe 失败");

        assert!(is_valid_game_dir(dir.path()));
    }

    #[test]
    fn detect_game_version_should_identify_steam_by_dll() {
        let dir = tempdir().expect("创建临时目录失败");
        fs::write(dir.path().join("UmamusumePrettyDerby_Jpn.exe"), []).expect("创建 exe 失败");
        fs::write(dir.path().join("steam_api64.dll"), []).expect("创建 steam_api64.dll 失败");

        assert_eq!(detect_game_version(dir.path()), GameVersion::Steam);
    }

    #[test]
    fn detect_game_version_should_default_to_dmm_for_valid_dir() {
        let dir = tempdir().expect("创建临时目录失败");
        fs::write(dir.path().join("umamusume.exe"), []).expect("创建 exe 失败");

        assert_eq!(detect_game_version(dir.path()), GameVersion::DMM);
    }

    #[test]
    fn collect_dmm_config_candidates_should_read_dmmgameplayer5_paths() {
        let config = serde_json::json!({
            "defaultInstallDir": r"C:\dmm",
            "contents": [
                {
                    "productId": "umamusume",
                    "detail": {
                        "installed": true,
                        "path": r"C:\dmm\Umamusume"
                    }
                },
                {
                    "productId": "other_game",
                    "detail": {
                        "installed": true,
                        "path": r"C:\dmm\OtherGame"
                    }
                }
            ]
        });

        let paths = collect_dmm_config_candidates(&config);

        assert!(paths.contains(&PathBuf::from(r"C:\dmm")));
        assert!(paths.contains(&PathBuf::from(r"C:\dmm\Umamusume")));
        assert!(!paths.contains(&PathBuf::from(r"C:\dmm\OtherGame")));
    }

    #[test]
    fn extract_registry_path_candidates_should_parse_display_icon_path() {
        let paths = extract_registry_path_candidates(r#""C:\dmm\Umamusume\umamusume.exe",0"#);

        assert!(paths.contains(&PathBuf::from(r"C:\dmm\Umamusume\umamusume.exe")));
        assert!(paths.contains(&PathBuf::from(r"C:\dmm\Umamusume")));
    }

    #[test]
    fn scan_game_dirs_under_should_find_nested_dmm_game_dir() {
        let root = tempdir().expect("创建临时目录失败");
        let game_dir = root.path().join("launcher").join("games").join("Umamusume");
        fs::create_dir_all(&game_dir).expect("创建游戏目录失败");
        fs::write(game_dir.join("umamusume.exe"), []).expect("创建 exe 失败");

        let mut games: Vec<DetectedGame> = Vec::new();
        scan_game_dirs_under(root.path(), &mut games, GameVersion::DMM, 3);

        assert_eq!(games.len(), 1);
        assert_eq!(games[0].path, game_dir);
        assert_eq!(games[0].version, GameVersion::DMM);
    }
}

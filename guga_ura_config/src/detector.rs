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
            GameVersion::Steam => "winhttp.dll",
            GameVersion::DMM => "UnityPlayer.dll",
            GameVersion::Unknown => "",
        }
    }
    
    /// 获取原始 DLL 备份名称
    pub fn backup_dll_name(&self) -> &'static str {
        match self {
            GameVersion::Steam => "winhttp_orig.dll",
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
    let normalized = path.to_string_lossy().to_lowercase().replace("/", "\\");
    let is_duplicate = games.iter().any(|g: &DetectedGame| {
        g.path.to_string_lossy().to_lowercase().replace("/", "\\") == normalized
    });
    
    if !is_duplicate && is_valid_game_dir(&path) {
        games.push(DetectedGame { path, version });
    }
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
            if dir_name.contains("umamusume") || 
               (dir_name.contains("uma") && dir_name.contains("derby")) {
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
    
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        // 扫描所有卸载信息
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        let uninstall_paths = [
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ];
        
        for uninstall_path in uninstall_paths {
            if let Ok(uninstall_key) = hklm.open_subkey(uninstall_path) {
                for name in uninstall_key.enum_keys().filter_map(Result::ok) {
                    // 检查 key 名称是否包含相关关键词
                    let name_lower = name.to_lowercase();
                    let name_matches = name_lower.contains("umamusume") || 
                                      name_lower.contains("uma musume") ||
                                      name_lower.contains("pretty derby");
                    
                    if let Ok(app_key) = uninstall_key.open_subkey(&name) {
                        let install_path: String = match app_key.get_value("InstallLocation") {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        
                        // 检查安装路径是否包含相关关键词
                        let path_lower = install_path.to_lowercase();
                        let path_matches = path_lower.contains("umamusume") ||
                                          path_lower.contains("uma musume") ||
                                          path_lower.contains("pretty derby") ||
                                          path_lower.contains("umapyoi");
                        
                        if name_matches || path_matches {
                            let path = PathBuf::from(&install_path);
                            if is_valid_game_dir(&path) {
                                let version = detect_game_version(&path);
                                if !games.iter().any(|g: &DetectedGame| g.path == path) {
                                    games.push(DetectedGame { path, version });
                                }
                            }
                        }
                    }
                }
            }
        }
    }


    
    // DMM 常见安装路径（扩展列表）
    let dmm_paths = [
        // DMMGames 标准路径
        r"C:\DMMGames\Umapyoi",
        r"C:\DMMGames\umamusume",
        r"C:\DMMGames\Umamusume",
        r"D:\DMMGames\Umapyoi",
        r"D:\DMMGames\umamusume",
        r"D:\DMMGames\Umamusume",
        r"E:\DMMGames\Umapyoi",
        r"E:\DMMGames\umamusume",
        r"E:\DMMGames\Umamusume",
        r"G:\DMMGames\Umapyoi",
        r"G:\DMMGames\umamusume",
        r"G:\DMMGames\Umamusume",
        // dmm 小写目录
        r"C:\dmm\Umapyoi",
        r"C:\dmm\umamusume",
        r"C:\dmm\Umamusume",
        r"D:\dmm\Umapyoi",
        r"D:\dmm\umamusume",
        r"D:\dmm\Umamusume",
        r"E:\dmm\Umapyoi",
        r"E:\dmm\umamusume",
        r"E:\dmm\Umamusume",
        r"G:\dmm\Umapyoi",
        r"G:\dmm\umamusume",
        r"G:\dmm\Umamusume",
        // DMM 目录
        r"C:\DMM\Umapyoi",
        r"C:\DMM\umamusume",
        r"C:\DMM\Umamusume",
        r"D:\DMM\Umapyoi",
        r"D:\DMM\umamusume",
        r"D:\DMM\Umamusume",
        r"G:\DMM\Umapyoi",
        r"G:\DMM\umamusume",
        r"G:\DMM\Umamusume",
    ];
    
    for path in dmm_paths {
        let path = PathBuf::from(path);
        if is_valid_game_dir(&path) {
            if !games.iter().any(|g: &DetectedGame| g.path == path) {
                games.push(DetectedGame {
                    path,
                    version: GameVersion::DMM,
                });
            }
        }
    }
    
    if games.is_empty() {
        None
    } else {
        Some(games)
    }
}

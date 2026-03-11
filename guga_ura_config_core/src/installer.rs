//! DLL 安装管理

use crate::detector::GameVersion;
use crate::embedded_dlls;
use std::fs;
use std::path::Path;

/// 安装状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStatus {
    /// 已安装
    Installed,
    /// 未安装
    NotInstalled,
    /// 需要更新
    NeedsUpdate,
    /// 无法确定
    Unknown,
}

impl InstallStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            InstallStatus::Installed => "✅ 已安装",
            InstallStatus::NotInstalled => "❌ 未安装",
            InstallStatus::NeedsUpdate => "🔄 需要更新",
            InstallStatus::Unknown => "❓ 未知",
        }
    }
}

/// 检查安装状态
pub fn check_install_status(game_dir: &Path, version: GameVersion) -> InstallStatus {
    let exe_name = match find_game_exe(game_dir) {
        Some(name) => name,
        None => return InstallStatus::Unknown,
    };

    let data_dir = game_dir.join("guga_ura_data");

    if version == GameVersion::Steam {
        // Steam 版：检查 cri_mana_vpx_orig.dll 备份是否存在于数据目录
        let backup_dll = data_dir.join("cri_mana_vpx_orig.dll");
        if backup_dll.exists() {
            return InstallStatus::Installed;
        }
    } else {
        // DMM 版：检查 .local 文件夹
        let local_dir = game_dir.join(format!("{}.local", exe_name));
        if local_dir.exists() {
            let unity_dll = local_dir.join("UnityPlayer.dll");
            let apphelp_dll = local_dir.join("apphelp.dll");
            if unity_dll.exists() && apphelp_dll.exists() {
                return InstallStatus::Installed;
            }
        }
    }

    InstallStatus::NotInstalled
}

/// 获取当前可执行文件所在目录（用于查找编译好的DLL）
fn get_exe_dir() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(|p| p.to_path_buf())
}

/// 查找可用的外部 DLL 文件
fn find_compiled_dll(dll_name: &str) -> Option<std::path::PathBuf> {
    let exe_dir = get_exe_dir();
    let possible_paths = [
        exe_dir.as_ref().map(|d| d.join(dll_name)),
        exe_dir.as_ref().and_then(|d| {
            d.parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("target").join("release").join(dll_name))
        }),
        Some(std::path::PathBuf::from("target/release").join(dll_name)),
    ];

    for path in possible_paths.iter().flatten() {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

/// 获取 DLL 数据（优先外部文件，其次内嵌）
fn get_dll_data(dll_name: &str) -> Result<Vec<u8>, String> {
    // 优先使用外部 DLL（通常是最新编译产物）
    if let Some(path) = find_compiled_dll(dll_name) {
        return fs::read(&path)
            .map_err(|e| format!("读取 {} 失败 ({}): {}", dll_name, path.display(), e));
    }

    // 回退：使用内嵌 DLL
    if embedded_dlls::has_embedded_dlls() {
        match dll_name {
            "UnityPlayer.dll" => return Ok(embedded_dlls::UNITY_PLAYER_DLL.to_vec()),
            "apphelp.dll" => return Ok(embedded_dlls::APPHELP_DLL.to_vec()),
            _ => {}
        }
    }

    Err(format!(
        "找不到 {}。请确保 {} 与配置工具 EXE 在同目录，或先重新构建配置工具（携带最新内嵌 DLL）",
        dll_name, dll_name
    ))
}

/// 安装 DLL
///
/// - DMM 版：使用 .local 文件夹方式
/// - Steam 版：直接替换 cri_mana_vpx.dll
pub fn install_dll(game_dir: &Path, version: GameVersion) -> Result<(), String> {
    if version == GameVersion::Unknown {
        return Err("未知的游戏版本".to_string());
    }

    // 创建数据目录
    let data_dir = game_dir.join("guga_ura_data");
    fs::create_dir_all(&data_dir).map_err(|e| format!("创建数据目录失败: {}", e))?;

    if version == GameVersion::Steam {
        install_steam(game_dir, &data_dir)
    } else {
        install_dmm(game_dir, &data_dir)
    }
}

/// Steam 版安装：直接替换 cri_mana_vpx.dll
/// 注意：Steam版的cri_mana_vpx.dll位于 *_Data/Plugins/x86_64/ 目录下
fn install_steam(game_dir: &Path, data_dir: &Path) -> Result<(), String> {
    // Steam版的cri_mana_vpx.dll在子目录中
    // 例如: UmamusumePrettyDerby_Jpn_Data/Plugins/x86_64/cri_mana_vpx.dll
    let plugins_dir = find_steam_plugins_dir(game_dir)
        .ok_or("找不到 Plugins/x86_64 目录，可能不是正确的Steam游戏目录")?;

    let orig_dll = plugins_dir.join("cri_mana_vpx.dll");
    let backup_dll = data_dir.join("cri_mana_vpx_orig.dll");

    // 检查原始 DLL 是否存在
    if !orig_dll.exists() {
        return Err(format!(
            "找不到 cri_mana_vpx.dll，路径: {}",
            orig_dll.display()
        ));
    }

    // 备份原始 DLL（如果未备份）
    if !backup_dll.exists() {
        fs::copy(&orig_dll, &backup_dll)
            .map_err(|e| format!("备份 cri_mana_vpx.dll 失败: {}", e))?;
    }

    // 获取 DLL 数据（内嵌或外部文件）
    let dll_data = get_dll_data("UnityPlayer.dll")?;

    // 用我们的 DLL 替换 cri_mana_vpx.dll
    fs::write(&orig_dll, &dll_data).map_err(|e| format!("替换 cri_mana_vpx.dll 失败: {}", e))?;

    // 记录安装信息
    let info = format!("steam\ncri_mana_vpx.dll\n{}", plugins_dir.display());
    fs::write(data_dir.join("install_info.txt"), info)
        .map_err(|e| format!("保存安装信息失败: {}", e))?;

    Ok(())
}

/// 查找Steam版游戏的Plugins/x86_64目录
fn find_steam_plugins_dir(game_dir: &Path) -> Option<std::path::PathBuf> {
    // 尝试常见的Data目录名模式
    let patterns = [
        "UmamusumePrettyDerby_Jpn_Data",
        "UmamusumePrettyDerby_Data",
        "umamusume_Data",
    ];

    for pattern in patterns {
        let plugins_path = game_dir.join(pattern).join("Plugins").join("x86_64");
        if plugins_path.exists() && plugins_path.join("cri_mana_vpx.dll").exists() {
            return Some(plugins_path);
        }
    }

    // 回退：遍历目录查找 *_Data/Plugins/x86_64
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("_Data") {
                let plugins_path = entry.path().join("Plugins").join("x86_64");
                if plugins_path.exists() && plugins_path.join("cri_mana_vpx.dll").exists() {
                    return Some(plugins_path);
                }
            }
        }
    }

    None
}

/// DMM 版安装：使用 .local 文件夹
fn install_dmm(game_dir: &Path, data_dir: &Path) -> Result<(), String> {
    let exe_name = find_game_exe(game_dir).ok_or_else(|| "找不到游戏可执行文件".to_string())?;

    // 创建 .local 文件夹
    let local_dir = game_dir.join(format!("{}.local", exe_name));
    fs::create_dir_all(&local_dir).map_err(|e| format!("创建 .local 目录失败: {}", e))?;

    // 安装 UnityPlayer.dll（内嵌或外部）
    let unity_data = get_dll_data("UnityPlayer.dll")?;
    fs::write(local_dir.join("UnityPlayer.dll"), &unity_data)
        .map_err(|e| format!("写入 UnityPlayer.dll 失败: {}", e))?;

    // 安装 apphelp.dll（内嵌或外部）
    let apphelp_data = get_dll_data("apphelp.dll")?;
    fs::write(local_dir.join("apphelp.dll"), &apphelp_data)
        .map_err(|e| format!("写入 apphelp.dll 失败: {}", e))?;

    // 记录安装信息
    let info = format!("dmm\n{}", exe_name);
    fs::write(data_dir.join("install_info.txt"), info)
        .map_err(|e| format!("保存安装信息失败: {}", e))?;

    Ok(())
}

/// 查找游戏可执行文件名
fn find_game_exe(game_dir: &Path) -> Option<String> {
    let known_exes = [
        "umamusume.exe",
        "UmamusumePrettyDerby_Jpn.exe",
        "UmamusumePrettyDerby.exe",
    ];

    for exe in known_exes {
        if game_dir.join(exe).exists() {
            return Some(exe.to_string());
        }
    }

    // 遍历目录查找
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("umamusume") && name.ends_with(".exe") {
                return Some(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    None
}

/// 卸载 DLL
pub fn uninstall_dll(game_dir: &Path, version: GameVersion) -> Result<(), String> {
    if version == GameVersion::Unknown {
        return Err("未知的游戏版本".to_string());
    }

    let data_dir = game_dir.join("guga_ura_data");

    if version == GameVersion::Steam {
        uninstall_steam(game_dir, &data_dir)
    } else {
        uninstall_dmm(game_dir, &data_dir)
    }
}

/// Steam 版卸载：恢复原始 cri_mana_vpx.dll
fn uninstall_steam(game_dir: &Path, data_dir: &Path) -> Result<(), String> {
    let plugins_dir = find_steam_plugins_dir(game_dir).ok_or("找不到 Plugins/x86_64 目录")?;

    let orig_dll = plugins_dir.join("cri_mana_vpx.dll");
    let backup_dll = data_dir.join("cri_mana_vpx_orig.dll");

    // 恢复原始 DLL
    if backup_dll.exists() {
        fs::copy(&backup_dll, &orig_dll)
            .map_err(|e| format!("恢复 cri_mana_vpx.dll 失败: {}", e))?;
        fs::remove_file(&backup_dll).map_err(|e| format!("删除备份失败: {}", e))?;
    } else {
        return Err("找不到备份文件，无法恢复".to_string());
    }

    // 清理数据目录
    if data_dir.exists() {
        let _ = fs::remove_dir_all(data_dir);
    }

    Ok(())
}

/// DMM 版卸载：删除 .local 文件夹
fn uninstall_dmm(game_dir: &Path, data_dir: &Path) -> Result<(), String> {
    let exe_name = find_game_exe(game_dir).ok_or_else(|| "找不到游戏可执行文件".to_string())?;

    // 删除 .local 文件夹
    let local_dir = game_dir.join(format!("{}.local", exe_name));
    if local_dir.exists() {
        fs::remove_dir_all(&local_dir).map_err(|e| format!("删除 .local 目录失败: {}", e))?;
    }

    // 删除数据目录
    if data_dir.exists() {
        let _ = fs::remove_dir_all(data_dir);
    }

    // 删除配置文件
    let config_file = game_dir.join("guga_ura_config.json");
    if config_file.exists() {
        let _ = fs::remove_file(&config_file);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_install_status, InstallStatus};
    use crate::detector::GameVersion;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn check_install_status_should_detect_dmm_installation() {
        let dir = tempdir().expect("创建临时目录失败");
        fs::write(dir.path().join("umamusume.exe"), []).expect("创建 exe 失败");
        let local_dir = dir.path().join("umamusume.exe.local");
        fs::create_dir_all(&local_dir).expect("创建 .local 目录失败");
        fs::write(local_dir.join("UnityPlayer.dll"), []).expect("创建 UnityPlayer.dll 失败");
        fs::write(local_dir.join("apphelp.dll"), []).expect("创建 apphelp.dll 失败");

        let status = check_install_status(dir.path(), GameVersion::DMM);

        assert_eq!(status, InstallStatus::Installed);
    }

    #[test]
    fn check_install_status_should_detect_steam_backup() {
        let dir = tempdir().expect("创建临时目录失败");
        fs::write(dir.path().join("UmamusumePrettyDerby_Jpn.exe"), []).expect("创建 exe 失败");
        let plugins_dir = dir
            .path()
            .join("UmamusumePrettyDerby_Jpn_Data")
            .join("Plugins")
            .join("x86_64");
        fs::create_dir_all(&plugins_dir).expect("创建 Plugins/x86_64 目录失败");
        fs::write(plugins_dir.join("cri_mana_vpx.dll"), []).expect("创建 cri_mana_vpx.dll 失败");
        let data_dir = dir.path().join("guga_ura_data");
        fs::create_dir_all(&data_dir).expect("创建数据目录失败");
        fs::write(data_dir.join("cri_mana_vpx_orig.dll"), []).expect("创建备份 DLL 失败");

        let status = check_install_status(dir.path(), GameVersion::Steam);

        assert_eq!(status, InstallStatus::Installed);
    }
}

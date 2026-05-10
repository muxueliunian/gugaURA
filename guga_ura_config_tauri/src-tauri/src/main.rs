//! Tauri 2 最小壳入口

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bootstrap;
mod commands;
mod state;
mod tool_settings;

use guga_ura_config_core::receiver;

fn main() {
    let receiver_runtime = receiver::start_embedded_receiver_managed();

    tauri::Builder::default()
        .manage(state::AppState::new(receiver_runtime))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name("gugaURA")
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap_state,
            commands::get_terminal_snapshot,
            commands::clear_terminal_logs,
            commands::scan_installed_games,
            commands::inspect_game_dir,
            commands::pick_directory,
            commands::get_dll_injection_context,
            commands::get_receiver_runtime_settings,
            commands::save_dll_injection_config,
            commands::save_receiver_runtime_settings,
            commands::install_dll_injection,
            commands::save_debug_mode,
            commands::uninstall_dll_injection,
            commands::get_game_settings_context,
            commands::save_game_settings,
            commands::get_tool_settings_context,
            commands::set_autostart_enabled,
            commands::check_app_update,
            commands::open_latest_release_page,
        ])
        .run(tauri::generate_context!("tauri.conf.json"))
        .expect("运行 Tauri 应用失败");
}

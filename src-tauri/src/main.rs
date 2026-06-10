#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod localizer;
mod updater;
mod ccswitch;
mod system_info;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::check_zh_cn_status,
            commands::install_localization,
            commands::uninstall_localization,
            commands::check_for_updates,
            commands::get_versions,
            commands::check_ccswitch_status,
            commands::install_ccswitch,
            commands::get_ccswitch_guide,
            commands::get_api_guide,
            commands::open_ccswitch_site,
            commands::open_ccswitch_releases,
            commands::open_config_file,
            commands::open_url_in_browser,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Claude ZH Helper");
}

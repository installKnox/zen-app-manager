mod startup;
mod services;

use startup::{get_startup_apps, toggle_app, create_app, delete_app};
use services::{get_system_services, toggle_service};
use std::path::PathBuf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_apps, 
            toggle_app_cmd, 
            create_app_cmd, 
            delete_app_cmd,
            get_system_services,
            toggle_service
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_apps() -> Vec<startup::StartupApp> {
    startup::get_startup_apps()
}

#[tauri::command]
fn toggle_app_cmd(path: String, enable: bool) -> Result<(), String> {
    startup::toggle_app(PathBuf::from(path), enable)
}

#[tauri::command]
fn create_app_cmd(name: String, command: String, description: String) -> Result<(), String> {
    startup::create_app(name, command, description)
}

#[tauri::command]
fn delete_app_cmd(path: String) -> Result<(), String> {
    startup::delete_app(PathBuf::from(path))
}

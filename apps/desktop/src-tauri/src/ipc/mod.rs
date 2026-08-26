use tauri::{AppHandle, State};
use crate::app::AppState;
use crate::error::AppErrorWrapper;

#[tauri::command]
pub fn quit_app(app_handle: AppHandle) {
    app_handle.exit(0);
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn get_platform_info() -> String {
    format!("Linux ({})", std::env::consts::ARCH)
}

#[tauri::command]
pub fn get_startup_status(state: State<'_, AppState>) -> bool {
    let core = &state.core;
    let config = core.config_manager.read().unwrap().get().clone();
    config.startup.autostart
}

#[tauri::command]
pub fn set_startup_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), AppErrorWrapper> {
    let core = &state.core;
    core.config_manager
        .write()
        .unwrap()
        .update(|cfg| {
            cfg.startup.autostart = enabled;
        })
        .map_err(AppErrorWrapper::from)
}

#[tauri::command]
pub fn reload_whatsapp(state: State<'_, AppState>) -> Result<(), AppErrorWrapper> {
    state.webview_manager.reload().map_err(AppErrorWrapper::from)
}

#[tauri::command]
pub fn hard_reload_whatsapp(state: State<'_, AppState>) -> Result<(), AppErrorWrapper> {
    state.webview_manager.hard_reload().map_err(AppErrorWrapper::from)
}

#[tauri::command]
pub fn reset_session(state: State<'_, AppState>) -> Result<(), AppErrorWrapper> {
    let core = &state.core;
    let config = core.config_manager.read().unwrap().get().clone();
    let profile_id = &config.accounts.active_profile_id;
    let data_dir = core.storage_manager.get_profile_data_dir(profile_id);
    let cache_dir = core.storage_manager.get_profile_cache_dir(profile_id);

    if data_dir.exists() {
        let _ = std::fs::remove_dir_all(data_dir);
    }
    if cache_dir.exists() {
        let _ = std::fs::remove_dir_all(cache_dir);
    }

    let _ = core.storage_manager.ensure_dirs(profile_id);
    let _ = state.webview_manager.hard_reload();

    Ok(())
}

use tauri::{AppHandle, State};
use whatnull_types::AccountProfile;
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

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> Vec<AccountProfile> {
    let core = &state.core;
    let config = core.config_manager.read().unwrap().get().clone();
    config.accounts.profiles
}

#[tauri::command]
pub fn create_profile(
    state: State<'_, AppState>,
    name: String,
    avatar_color: String,
) -> Result<AccountProfile, AppErrorWrapper> {
    let core = &state.core;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let new_id = format!("profile-{}", now);
    let profile = AccountProfile {
        id: new_id.clone(),
        display_name: name,
        storage_partition: new_id.clone(),
        avatar_color,
        created_at: now,
        last_used_at: now,
    };

    let prof_clone = profile.clone();
    core.config_manager
        .write()
        .unwrap()
        .update(move |cfg| {
            cfg.accounts.profiles.push(prof_clone);
        })
        .map_err(AppErrorWrapper::from)?;

    let _ = core.storage_manager.ensure_dirs(&new_id);
    Ok(profile)
}

#[tauri::command]
pub fn switch_profile(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), AppErrorWrapper> {
    let core = &state.core;
    let pid_clone = profile_id.clone();
    core.config_manager
        .write()
        .unwrap()
        .update(move |cfg| {
            cfg.accounts.active_profile_id = pid_clone;
        })
        .map_err(AppErrorWrapper::from)?;

    let data_dir = core.storage_manager.get_profile_data_dir(&profile_id);
    let _ = core.storage_manager.ensure_dirs(&profile_id);
    let _ = state.webview_manager.switch_account_webview(&app_handle, data_dir);

    Ok(())
}

#[tauri::command]
pub fn delete_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), AppErrorWrapper> {
    let core = &state.core;
    let pid_clone = profile_id.clone();

    core.config_manager
        .write()
        .unwrap()
        .update(move |cfg| {
            cfg.accounts.profiles.retain(|p| p.id != pid_clone);
        })
        .map_err(AppErrorWrapper::from)?;

    let data_dir = core.storage_manager.get_profile_data_dir(&profile_id);
    let cache_dir = core.storage_manager.get_profile_cache_dir(&profile_id);

    if data_dir.exists() {
        let _ = std::fs::remove_dir_all(data_dir);
    }
    if cache_dir.exists() {
        let _ = std::fs::remove_dir_all(cache_dir);
    }

    Ok(())
}

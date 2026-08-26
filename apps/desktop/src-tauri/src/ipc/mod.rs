use crate::app::AppState;
use crate::error::AppErrorWrapper;
use tauri::{AppHandle, Manager, State};
use whatnull_types::{AccountProfile, AppError};

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
    let autostart = whatnull_platform::AutostartManager::new().map_err(AppErrorWrapper::from)?;
    autostart
        .set_enabled(enabled)
        .map_err(AppErrorWrapper::from)?;

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
    state
        .webview_manager
        .reload()
        .map_err(AppErrorWrapper::from)
}

#[tauri::command]
pub fn hard_reload_whatsapp(state: State<'_, AppState>) -> Result<(), AppErrorWrapper> {
    state
        .webview_manager
        .hard_reload()
        .map_err(AppErrorWrapper::from)
}

#[tauri::command]
pub fn set_whatsapp_visible(
    state: State<'_, AppState>,
    visible: bool,
) -> Result<(), AppErrorWrapper> {
    state
        .webview_manager
        .set_whatsapp_visible(visible)
        .map_err(AppErrorWrapper::from)
}

#[tauri::command]
pub fn set_shell_overlay_mode(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    overlay: bool,
) -> Result<(), AppErrorWrapper> {
    let window = app_handle.get_window("main").ok_or_else(|| {
        AppErrorWrapper::from(AppError::Window("Main window not found".to_string()))
    })?;

    state
        .webview_manager
        .set_shell_overlay_mode(&window, overlay)
        .map_err(AppErrorWrapper::from)
}

#[tauri::command]
pub fn reset_session(state: State<'_, AppState>) -> Result<(), AppErrorWrapper> {
    let core = &state.core;
    let config = core.config_manager.read().unwrap().get().clone();
    let profile_id = &config.accounts.active_profile_id;
    ensure_valid_profile_id(profile_id)?;

    let data_dir = core.storage_manager.get_profile_data_dir(profile_id);
    let cache_dir = core.storage_manager.get_profile_cache_dir(profile_id);

    if data_dir.exists() {
        std::fs::remove_dir_all(data_dir).map_err(AppErrorWrapper::from)?;
    }
    if cache_dir.exists() {
        std::fs::remove_dir_all(cache_dir).map_err(AppErrorWrapper::from)?;
    }

    core.storage_manager
        .ensure_dirs(profile_id)
        .map_err(AppErrorWrapper::from)?;
    state
        .webview_manager
        .hard_reload()
        .map_err(AppErrorWrapper::from)?;

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
    let display_name = name.trim();
    if display_name.is_empty() || display_name.len() > 64 {
        return Err(AppErrorWrapper::from(AppError::Account(
            "Profile name must be between 1 and 64 characters".to_string(),
        )));
    }
    if !is_safe_avatar_color(&avatar_color) {
        return Err(AppErrorWrapper::from(AppError::Account(
            "Avatar color must be a hex RGB value".to_string(),
        )));
    }

    let now = unix_timestamp()?;
    let new_id = format!("profile-{}", now);
    let profile = AccountProfile {
        id: new_id.clone(),
        display_name: display_name.to_string(),
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

    core.storage_manager
        .ensure_dirs(&new_id)
        .map_err(AppErrorWrapper::from)?;
    Ok(profile)
}

#[tauri::command]
pub fn switch_profile(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), AppErrorWrapper> {
    let core = &state.core;
    ensure_valid_profile_id(&profile_id)?;

    let exists = core
        .config_manager
        .read()
        .unwrap()
        .get()
        .accounts
        .profiles
        .iter()
        .any(|profile| profile.id == profile_id);
    if !exists {
        return Err(AppErrorWrapper::from(AppError::Account(format!(
            "Profile not found: {}",
            profile_id
        ))));
    }

    let pid_clone = profile_id.clone();
    let now = unix_timestamp()?;
    core.config_manager
        .write()
        .unwrap()
        .update(move |cfg| {
            cfg.accounts.active_profile_id = pid_clone.clone();
            if let Some(profile) = cfg
                .accounts
                .profiles
                .iter_mut()
                .find(|profile| profile.id == pid_clone)
            {
                profile.last_used_at = now;
            }
        })
        .map_err(AppErrorWrapper::from)?;

    let data_dir = core.storage_manager.get_profile_data_dir(&profile_id);
    core.storage_manager
        .ensure_dirs(&profile_id)
        .map_err(AppErrorWrapper::from)?;
    state
        .webview_manager
        .switch_account_webview(&app_handle, data_dir)
        .map_err(AppErrorWrapper::from)?;

    Ok(())
}

#[tauri::command]
pub fn delete_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), AppErrorWrapper> {
    let core = &state.core;
    ensure_valid_profile_id(&profile_id)?;

    let config = core.config_manager.read().unwrap().get().clone();
    if config.accounts.active_profile_id == profile_id {
        return Err(AppErrorWrapper::from(AppError::Account(
            "Cannot delete the active profile".to_string(),
        )));
    }
    if config.accounts.profiles.len() <= 1 {
        return Err(AppErrorWrapper::from(AppError::Account(
            "Cannot delete the last profile".to_string(),
        )));
    }

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
        std::fs::remove_dir_all(data_dir).map_err(AppErrorWrapper::from)?;
    }
    if cache_dir.exists() {
        std::fs::remove_dir_all(cache_dir).map_err(AppErrorWrapper::from)?;
    }

    Ok(())
}

fn ensure_valid_profile_id(profile_id: &str) -> Result<(), AppErrorWrapper> {
    if is_valid_profile_id(profile_id) {
        Ok(())
    } else {
        Err(AppErrorWrapper::from(AppError::Account(
            "Invalid profile id".to_string(),
        )))
    }
}

fn is_valid_profile_id(profile_id: &str) -> bool {
    !profile_id.is_empty()
        && profile_id.len() <= 80
        && profile_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn is_safe_avatar_color(color: &str) -> bool {
    color.len() == 7 && color.starts_with('#') && color[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

fn unix_timestamp() -> Result<u64, AppErrorWrapper> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|e| AppErrorWrapper::from(AppError::Internal(e.to_string())))
}

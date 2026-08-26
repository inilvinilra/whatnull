use tauri::State;
use crate::app::AppState;
use crate::error::AppErrorWrapper;
use whatnull_config::AppConfig;

#[tauri::command]
pub fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, AppErrorWrapper> {
    let core = &state.core;
    let config = core.config_manager.read().unwrap().get().clone();
    Ok(config)
}

#[tauri::command]
pub fn update_app_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), AppErrorWrapper> {
    let core = &state.core;
    core.config_manager
        .write()
        .unwrap()
        .update(|cfg| {
            *cfg = config;
        })
        .map_err(AppErrorWrapper::from)
}

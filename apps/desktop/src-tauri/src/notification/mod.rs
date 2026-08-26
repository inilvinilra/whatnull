use tauri::State;
use whatnull_notification::NotificationFilter;
use whatnull_types::NotificationPayload;
use crate::app::AppState;
use crate::error::AppErrorWrapper;

#[tauri::command]
pub fn dispatch_notification(
    state: State<'_, AppState>,
    payload: NotificationPayload,
) -> Result<bool, AppErrorWrapper> {
    let core = &state.core;
    let config = core.config_manager.read().unwrap().get().notifications.clone();

    if let Some(filtered) = NotificationFilter::process(&payload, &config) {
        let _ = filtered;
        Ok(true)
    } else {
        Ok(false)
    }
}

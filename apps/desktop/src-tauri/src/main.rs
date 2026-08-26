#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::sync::Mutex;
use tauri::Manager;

mod app;
mod config;
mod error;
mod events;
mod ipc;
mod platform;
mod privacy;
mod session;
mod account;
mod notification;
mod security;
mod storage;
mod tray;
mod update;
mod webview;
mod window;

static MAIN_WINDOW: Mutex<Option<tauri::WebviewWindow>> = Mutex::new(None);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct WindowBounds {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    maximized: bool,
}

fn main() {
    let profile_id = "default";

    let _uds_guard = match whatnull_platform::check_single_instance(profile_id, || {
        if let Ok(guard) = MAIN_WINDOW.lock() {
            if let Some(ref window) = *guard {
                let _ = window.show();
                let _ = window.focus();
            }
        }
    }) {
        Ok(whatnull_platform::SingleInstanceResult::Primary(guard)) => guard,
        Ok(whatnull_platform::SingleInstanceResult::Secondary) => {
            std::process::exit(0);
        }
        Err(_) => {
            std::process::exit(1);
        }
    };

    let core = match whatnull_core::AppCore::new() {
        Ok(c) => c,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let state_dir = core.paths.state_dir.clone();
    let config_manager = core.config_manager.clone();
    let storage_manager = core.storage_manager.clone();
    let webview_manager = std::sync::Arc::new(crate::webview::WebViewManager::new());

    tauri::Builder::default()
        .manage(crate::app::AppState {
            core: std::sync::Arc::new(core),
            webview_manager: webview_manager.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            crate::config::get_app_config,
            crate::config::update_app_config,
            crate::ipc::quit_app,
            crate::ipc::get_app_version,
            crate::ipc::get_platform_info,
            crate::ipc::get_startup_status,
            crate::ipc::set_startup_enabled,
            crate::ipc::reload_whatsapp,
            crate::ipc::hard_reload_whatsapp,
            crate::ipc::reset_session,
            crate::notification::dispatch_notification,
        ])
        .setup(move |app| {
            let main_window = app.get_webview_window("main").ok_or_else(|| {
                tauri::Error::WindowNotFound
            })?;

            *MAIN_WINDOW.lock().unwrap() = Some(main_window.clone());

            let _ = crate::tray::init_tray(app);

            let profile_data_dir = storage_manager.get_profile_data_dir(profile_id);
            let _ = storage_manager.ensure_dirs(profile_id);
            let _ = webview_manager.create_whatsapp_webview(app.handle(), profile_data_dir);

            let state_file = state_dir.join("window_bounds.json");
            let config = config_manager.read().unwrap().get().clone();

            if config.general.remember_window_position && state_file.exists() {
                if let Ok(content) = fs::read_to_string(&state_file) {
                    if let Ok(bounds) = serde_json::from_str::<WindowBounds>(&content) {
                        let factor = main_window.scale_factor().unwrap_or(1.0);
                        let _ = main_window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                            bounds.width,
                            bounds.height,
                        )));
                        let _ = main_window.set_position(tauri::Position::Logical(
                            tauri::LogicalPosition::new(bounds.x, bounds.y),
                        ));
                        if bounds.maximized {
                            let _ = main_window.maximize();
                        }
                    }
                }
            }

            let main_window_clone = main_window.clone();
            let state_file_clone = state_file.clone();
            let config_manager_clone = config_manager.clone();

            main_window.on_window_event(move |event| match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let cfg = config_manager_clone.read().unwrap().get().clone();
                    match cfg.general.close_behavior {
                        whatnull_types::CloseBehavior::HideToTray => {
                            api.prevent_close();
                            let _ = main_window_clone.hide();
                        }
                        whatnull_types::CloseBehavior::Quit => {}
                        whatnull_types::CloseBehavior::Ask => {
                            api.prevent_close();
                            let _ = main_window_clone.hide();
                        }
                    }

                    if cfg.general.remember_window_position {
                        if let Ok(factor) = main_window_clone.scale_factor() {
                            if let Ok(physical_size) = main_window_clone.inner_size() {
                                if let Ok(physical_pos) = main_window_clone.outer_position() {
                                    let logical_size = physical_size.to_logical(factor);
                                    let logical_pos = physical_pos.to_logical(factor);
                                    let maximized = main_window_clone.is_maximized().unwrap_or(false);

                                    let bounds = WindowBounds {
                                        width: logical_size.width,
                                        height: logical_size.height,
                                        x: logical_pos.x,
                                        y: logical_pos.y,
                                        maximized,
                                    };

                                    if let Ok(serialized) = serde_json::to_string(&bounds) {
                                        let _ = fs::create_dir_all(&state_dir);
                                        let _ = fs::write(&state_file_clone, serialized);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

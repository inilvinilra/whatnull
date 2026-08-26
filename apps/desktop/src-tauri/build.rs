fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_app_config",
            "update_app_config",
            "quit_app",
            "get_app_version",
            "get_platform_info",
            "get_startup_status",
            "set_startup_enabled",
            "reload_whatsapp",
            "hard_reload_whatsapp",
            "set_overlay_visible",
            "request_shell_action",
            "reset_session",
            "list_profiles",
            "create_profile",
            "switch_profile",
            "delete_profile",
            "dispatch_notification",
            "strip_file_metadata",
            "inspect_file_metadata",
            "sanitize_upload_files",
        ]),
    ))
    .expect("failed to run tauri build script");
}

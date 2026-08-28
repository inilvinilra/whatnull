use std::sync::{Arc, RwLock};

use webkit2gtk::glib::translate::{from_glib, ToGlibPtr};
use webkit2gtk::glib::Cast;
use webkit2gtk::{
    PermissionRequest, PermissionRequestExt, SettingsExt, UserMediaPermissionRequest,
    UserMediaPermissionRequestExt, WebView, WebViewExt,
};
use whatnull_config::{ConfigManager, PermissionsConfig};

pub fn configure(webview: &WebView, config_manager: Arc<RwLock<ConfigManager>>) {
    if let Some(settings) = WebViewExt::settings(webview) {
        settings.set_enable_media_stream(true);
        settings.set_enable_webrtc(true);
        settings.set_enable_mediasource(true);
        settings.set_enable_encrypted_media(true);
    }

    webview.connect_permission_request(move |_, request| decide(&config_manager, request));
}

fn decide(config_manager: &Arc<RwLock<ConfigManager>>, request: &PermissionRequest) -> bool {
    let permissions = match config_manager.read() {
        Ok(manager) => manager.get().permissions,
        Err(_) => PermissionsConfig {
            microphone: false,
            camera: false,
            screen_share: false,
        },
    };

    let granted = match request.downcast_ref::<UserMediaPermissionRequest>() {
        Some(media) => evaluate_user_media(media, &permissions),
        None => false,
    };

    if granted {
        request.allow();
    } else {
        request.deny();
    }

    true
}

fn evaluate_user_media(
    request: &UserMediaPermissionRequest,
    permissions: &PermissionsConfig,
) -> bool {
    if is_for_display_device(request) {
        return permissions.screen_share;
    }

    let wants_audio = request.is_for_audio_device();
    let wants_video = request.is_for_video_device();

    if !wants_audio && !wants_video {
        return false;
    }

    (!wants_audio || permissions.microphone) && (!wants_video || permissions.camera)
}

fn is_for_display_device(request: &UserMediaPermissionRequest) -> bool {
    unsafe {
        from_glib(
            webkit2gtk::ffi::webkit_user_media_permission_is_for_display_device(
                request.to_glib_none().0,
            ),
        )
    }
}

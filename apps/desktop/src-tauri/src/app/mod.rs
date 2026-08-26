use std::sync::Arc;
use whatnull_core::AppCore;
use crate::webview::WebViewManager;

pub struct AppState {
    pub core: Arc<AppCore>,
    pub webview_manager: Arc<WebViewManager>,
}

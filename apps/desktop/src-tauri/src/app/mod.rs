use crate::webview::WebViewManager;
use std::sync::Arc;
use whatnull_core::AppCore;

pub struct AppState {
    pub core: Arc<AppCore>,
    pub webview_manager: Arc<WebViewManager>,
}

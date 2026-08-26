use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use whatnull_security::{NavigationDecision, NavigationPolicy};
use whatnull_types::AppError;

pub struct WebViewManager {
    remote_window: Arc<Mutex<Option<WebviewWindow>>>,
}

impl WebViewManager {
    pub fn new() -> Self {
        Self {
            remote_window: Arc::new(Mutex::new(None)),
        }
    }

    pub fn create_whatsapp_webview(
        &self,
        app: &AppHandle,
        data_dir: PathBuf,
    ) -> Result<WebviewWindow, AppError> {
        if let Some(existing) = app.get_webview_window("whatsapp_remote") {
            let _ = existing.show();
            let _ = existing.focus();
            return Ok(existing);
        }

        let target_url = "https://web.whatsapp.com".parse().map_err(|e| {
            AppError::WebView(format!("Invalid WhatsApp target URL: {}", e))
        })?;

        let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

        let builder = WebviewWindowBuilder::new(
            app,
            "whatsapp_remote",
            WebviewUrl::External(target_url),
        )
        .title("WhatsApp Web")
        .user_agent(user_agent)
        .data_directory(data_dir)
        .on_navigation(|url| {
            let decision = NavigationPolicy::evaluate(url.as_str());
            match decision {
                NavigationDecision::Allow => true,
                NavigationDecision::OpenExternally => {
                    let _ = open::that(url.as_str());
                    false
                }
                NavigationDecision::Reject => false,
            }
        });

        let window = builder.build().map_err(|e| {
            AppError::WebView(format!("Failed to build WhatsApp remote webview: {}", e))
        })?;

        if let Ok(mut guard) = self.remote_window.lock() {
            *guard = Some(window.clone());
        }

        Ok(window)
    }

    pub fn reload(&self) -> Result<(), AppError> {
        if let Ok(guard) = self.remote_window.lock() {
            if let Some(ref window) = *guard {
                let _ = window.eval("window.location.reload();");
                return Ok(());
            }
        }
        Err(AppError::WebView("WhatsApp webview instance not found".to_string()))
    }

    pub fn hard_reload(&self) -> Result<(), AppError> {
        if let Ok(guard) = self.remote_window.lock() {
            if let Some(ref window) = *guard {
                let target_url = "https://web.whatsapp.com".parse().map_err(|e| {
                    AppError::WebView(format!("Invalid URL: {}", e))
                })?;
                let _ = window.navigate(target_url);
                return Ok(());
            }
        }
        Err(AppError::WebView("WhatsApp webview instance not found".to_string()))
    }
}

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Url, Webview, WebviewBuilder, WebviewUrl};
use whatnull_security::{NavigationDecision, NavigationPolicy};
use whatnull_types::AppError;

/// Manages the WhatsApp Web webview embedded within the main window.
///
/// Instead of spawning a separate window, the WhatsApp webview is
/// added as a child of the main application window. This provides
/// a unified single-window experience.
pub struct WebViewManager {
    remote_webview: Arc<Mutex<Option<Webview>>>,
}

impl WebViewManager {
    pub fn new() -> Self {
        Self {
            remote_webview: Arc::new(Mutex::new(None)),
        }
    }

    /// Create the WhatsApp webview as a child of the main window.
    ///
    /// The webview fills the entire main window area. The React UI
    /// (sidebar, overlays) renders on top via the main webview with
    /// transparent background regions.
    pub fn create_whatsapp_webview(
        &self,
        app: &AppHandle,
        data_dir: PathBuf,
    ) -> Result<(), AppError> {
        // If it already exists, just show it
        if let Some(existing) = app.get_webview("whatsapp_remote") {
            let _ = existing.set_focus();
            return Ok(());
        }

        let target_url: Url = "https://web.whatsapp.com".parse().map_err(|e| {
            AppError::WebView(format!("Invalid WhatsApp target URL: {}", e))
        })?;

        let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

        // Get the main window (Window, not WebviewWindow)
        let main_window = app.get_window("main").ok_or_else(|| {
            AppError::Window("Main window not found".to_string())
        })?;

        // Get current window dimensions for positioning
        let scale_factor = main_window.scale_factor().unwrap_or(1.0);
        let physical_size = main_window.inner_size().map_err(|e| {
            AppError::Window(format!("Failed to get window size: {}", e))
        })?;
        let logical_size = physical_size.to_logical::<f64>(scale_factor);

        // Build the WhatsApp webview
        let webview_builder = WebviewBuilder::new(
            "whatsapp_remote",
            WebviewUrl::External(target_url),
        )
        .user_agent(user_agent)
        .data_directory(data_dir)
        .auto_resize()
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

        let sidebar_width = 60.0;
        let webview_width = (logical_size.width - sidebar_width).max(100.0);

        // Add WhatsApp as a child webview of the main window, filling the right pane
        let webview = main_window.add_child(
            webview_builder,
            tauri::LogicalPosition::new(sidebar_width, 0.0),
            tauri::LogicalSize::new(webview_width, logical_size.height),
        ).map_err(|e| {
            AppError::WebView(format!("Failed to create embedded WhatsApp webview: {}", e))
        })?;

        if let Ok(mut guard) = self.remote_webview.lock() {
            *guard = Some(webview);
        }

        Ok(())
    }

    /// Switch to a different account by destroying and recreating the webview
    pub fn switch_account_webview(
        &self,
        app: &AppHandle,
        data_dir: PathBuf,
    ) -> Result<(), AppError> {
        // Close existing webview
        if let Ok(mut guard) = self.remote_webview.lock() {
            if let Some(existing) = guard.take() {
                let _ = existing.close();
            }
        } else if let Some(existing) = app.get_webview("whatsapp_remote") {
            let _ = existing.close();
        }

        self.create_whatsapp_webview(app, data_dir)
    }

    /// Soft reload — re-evaluate the current page
    pub fn reload(&self) -> Result<(), AppError> {
        if let Ok(guard) = self.remote_webview.lock() {
            if let Some(ref webview) = *guard {
                let _ = webview.eval("window.location.reload();");
                return Ok(());
            }
        }
        Err(AppError::WebView("WhatsApp webview instance not found".to_string()))
    }

    /// Hard reload — navigate back to the WhatsApp Web URL
    pub fn hard_reload(&self) -> Result<(), AppError> {
        if let Ok(guard) = self.remote_webview.lock() {
            if let Some(ref webview) = *guard {
                let target_url: Url = "https://web.whatsapp.com".parse().map_err(|e| {
                    AppError::WebView(format!("Invalid URL: {}", e))
                })?;
                let _ = webview.navigate(target_url);
                return Ok(());
            }
        }
        Err(AppError::WebView("WhatsApp webview instance not found".to_string()))
    }
}

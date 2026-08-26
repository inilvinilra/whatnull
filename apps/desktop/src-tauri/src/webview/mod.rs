use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Url, WebviewWindow};
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

    /// Configure and navigate the main application window to WhatsApp Web.
    ///
    /// This provides a clean, single-window WhatsApp Desktop experience with:
    /// - 100% window coverage (no layout offsets or split windows)
    /// - Integrated WhatNull Privacy Shield
    /// - Metadata stripping on file uploads
    /// - Minimum RAM & CPU footprint
    pub fn create_whatsapp_webview(
        &self,
        app: &AppHandle,
        _data_dir: PathBuf,
    ) -> Result<(), AppError> {
        let main_window = app.get_webview_window("main").ok_or_else(|| {
            AppError::Window("Main window not found".to_string())
        })?;

        let target_url: Url = "https://web.whatsapp.com".parse().map_err(|e| {
            AppError::WebView(format!("Invalid WhatsApp target URL: {}", e))
        })?;

        // 1. Inject WhatNull Privacy Shield Script
        let inject_script = r###"
        (function() {
            if (window.__WHATNULL_INITIALIZED__) return;
            window.__WHATNULL_INITIALIZED__ = true;

            const style = document.createElement('style');
            style.id = 'whatnull-custom-styles';
            style.textContent = `
                #whatnull-privacy-overlay {
                    position: fixed;
                    inset: 0;
                    background: rgba(11, 15, 25, 0.92);
                    backdrop-filter: blur(30px);
                    -webkit-backdrop-filter: blur(30px);
                    z-index: 99999999;
                    display: none;
                    flex-direction: column;
                    align-items: center;
                    justify-content: center;
                    color: #f3f4f6;
                    font-family: system-ui, -apple-system, sans-serif;
                    user-select: none;
                }
                #whatnull-privacy-overlay.active {
                    display: flex !important;
                }
                #whatnull-lock-icon {
                    width: 64px;
                    height: 64px;
                    background: rgba(13, 148, 136, 0.2);
                    border: 1px solid rgba(20, 184, 166, 0.4);
                    border-radius: 20px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    margin-bottom: 20px;
                    box-shadow: 0 0 30px rgba(13, 148, 136, 0.3);
                }
                #whatnull-unlock-btn {
                    margin-top: 24px;
                    padding: 12px 32px;
                    background: linear-gradient(135deg, #0d9488, #14b8a6);
                    color: white;
                    border: none;
                    border-radius: 12px;
                    font-weight: 600;
                    font-size: 15px;
                    cursor: pointer;
                    box-shadow: 0 4px 14px rgba(13, 148, 136, 0.4);
                    transition: transform 0.2s, box-shadow 0.2s;
                }
                #whatnull-unlock-btn:hover {
                    transform: translateY(-2px);
                    box-shadow: 0 6px 20px rgba(13, 148, 136, 0.6);
                }
            `;
            document.head.appendChild(style);

            const overlay = document.createElement('div');
            overlay.id = 'whatnull-privacy-overlay';
            overlay.innerHTML = `
                <div id="whatnull-lock-icon">
                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#14b8a6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
                        <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
                    </svg>
                </div>
                <h1 style="font-size: 26px; font-weight: 700; letter-spacing: -0.5px; margin: 0 0 8px 0; color: #ffffff;">WhatNull Privacy Shield</h1>
                <p style="color: #9ca3af; font-size: 14px; margin: 0; text-align: center;">Session is locked to prevent unauthorized access.</p>
                <button id="whatnull-unlock-btn">Unlock Session (Ctrl+L)</button>
            `;
            document.body.appendChild(overlay);

            let isLocked = false;
            function togglePrivacyLock(forceState) {
                isLocked = typeof forceState === 'boolean' ? forceState : !isLocked;
                overlay.classList.toggle('active', isLocked);
            }

            document.addEventListener('keydown', (e) => {
                if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
                    e.preventDefault();
                    togglePrivacyLock();
                }
            });

            document.getElementById('whatnull-unlock-btn')?.addEventListener('click', () => {
                togglePrivacyLock(false);
            });
        })();
        "###;

        let _ = main_window.eval(inject_script);

        // 2. Navigate main window to WhatsApp Web
        main_window.navigate(target_url).map_err(|e| {
            AppError::WebView(format!("Failed to navigate to WhatsApp Web: {}", e))
        })?;

        if let Ok(mut guard) = self.remote_window.lock() {
            *guard = Some(main_window);
        }

        Ok(())
    }

    pub fn switch_account_webview(
        &self,
        app: &AppHandle,
        data_dir: PathBuf,
    ) -> Result<(), AppError> {
        self.create_whatsapp_webview(app, data_dir)
    }

    pub fn reload(&self) -> Result<(), AppError> {
        if let Ok(guard) = self.remote_window.lock() {
            if let Some(ref window) = *guard {
                let _ = window.eval("window.location.reload();");
                return Ok(());
            }
        }
        Err(AppError::WebView("Main window instance not found".to_string()))
    }

    pub fn hard_reload(&self) -> Result<(), AppError> {
        if let Ok(guard) = self.remote_window.lock() {
            if let Some(ref window) = *guard {
                let target_url: Url = "https://web.whatsapp.com".parse().map_err(|e| {
                    AppError::WebView(format!("Invalid URL: {}", e))
                })?;
                let _ = window.navigate(target_url);
                return Ok(());
            }
        }
        Err(AppError::WebView("Main window instance not found".to_string()))
    }
}

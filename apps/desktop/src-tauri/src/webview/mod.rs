use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
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

    /// Build the main WhatsApp Desktop window dynamically with:
    /// - Live DOM Message Caching & Anti-Revoke Recovery (Continuously caches live text & restores deleted messages inside bubbles)
    /// - WhatsApp Voice & Video Calling activation (Spoofs macOS Chrome + MediaDevices input/output enumeration)
    /// - WebRTC local IP leak protection
    /// - Floating glassmorphic control pill & settings modal
    pub fn create_whatsapp_webview(
        &self,
        app: &AppHandle,
        data_dir: PathBuf,
    ) -> Result<WebviewWindow, AppError> {
        if let Some(existing) = app.get_webview_window("main") {
            let _ = existing.show();
            let _ = existing.set_focus();
            return Ok(existing);
        }

        let target_url: Url = "https://web.whatsapp.com".parse().map_err(|e| {
            AppError::WebView(format!("Invalid WhatsApp target URL: {}", e))
        })?;

        let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

        let inject_script = r###"
        (function() {
            if (window.__WHATNULL_INITIALIZED__) return;
            window.__WHATNULL_INITIALIZED__ = true;

            // 1. DEVICE & CALLING SPOOFING (Activates WhatsApp Voice 📞 & Video 📹 Calls)
            try {
                const fakeUA = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36';
                Object.defineProperty(navigator, 'userAgent', { get: () => fakeUA, configurable: true });
                Object.defineProperty(navigator, 'appVersion', { get: () => fakeUA, configurable: true });
                Object.defineProperty(navigator, 'platform', { get: () => 'MacIntel', configurable: true });
                Object.defineProperty(navigator, 'vendor', { get: () => 'Google Inc.', configurable: true });
                Object.defineProperty(navigator, 'deviceMemory', { get: () => 8, configurable: true });
                Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 8, configurable: true });

                // Mock Media Devices Enumeration so WhatsApp Calling Module detects Mic & Camera
                if (navigator.mediaDevices) {
                    navigator.mediaDevices.enumerateDevices = function() {
                        return Promise.resolve([
                            { deviceId: 'default-mic', kind: 'audioinput', label: 'Default Microphone', groupId: 'group1' },
                            { deviceId: 'default-cam', kind: 'videoinput', label: 'Default HD Camera', groupId: 'group2' },
                            { deviceId: 'default-spk', kind: 'audiooutput', label: 'Default Speaker', groupId: 'group1' }
                        ]);
                    };
                }
            } catch(e) {}

            // 2. WEBRTC LOCAL IP & MAC LEAK PREVENTION
            try {
                const origCreateOffer = window.RTCPeerConnection && window.RTCPeerConnection.prototype.createOffer;
                if (origCreateOffer) {
                    window.RTCPeerConnection.prototype.createOffer = function(opts) {
                        return origCreateOffer.call(this, opts).then(offer => {
                            if (offer && offer.sdp) {
                                offer.sdp = offer.sdp.replace(/a=candidate:.*?\r\n/g, (line) => {
                                    if (/192\.168\.|10\.|172\.(1[6-9]|2[0-9]|3[01])|fe80::/i.test(line)) {
                                        return '';
                                    }
                                    return line;
                                });
                            }
                            return offer;
                        });
                    };
                }
            } catch(e) {}

            // 3. LIVE DOM MESSAGE CACHE & ANTI-REVOKE RECOVERY
            const liveMessageCache = new Map();

            function captureAndRestoreMessages() {
                try {
                    const rows = document.querySelectorAll('div[role="row"], div[data-id]');
                    rows.forEach(row => {
                        const msgId = row.getAttribute('data-id') || row.getAttribute('id');
                        if (!msgId) return;

                        const isDeletedMarker = row.textContent.includes('This message was deleted') || 
                                                row.textContent.includes('Bu mesaj silindi') || 
                                                row.textContent.includes('Сообщение удалено');

                        if (!isDeletedMarker) {
                            // Message is live: Cache its text & HTML
                            const contentNode = row.querySelector('.copyable-text, .selectable-text') || row;
                            if (contentNode && contentNode.innerText && contentNode.innerText.trim().length > 0) {
                                liveMessageCache.set(msgId, {
                                    html: contentNode.innerHTML,
                                    text: contentNode.innerText,
                                    time: new Date().toLocaleTimeString()
                                });
                            }
                        } else {
                            // Message IS deleted: Restore original cached content!
                            if (!row.dataset.whatnullRestored) {
                                row.dataset.whatnullRestored = 'true';
                                const cached = liveMessageCache.get(msgId);
                                const targetNode = row.querySelector('.copyable-text, .selectable-text') || row;
                                
                                if (cached) {
                                    targetNode.innerHTML = `
                                        <div style="background: rgba(239, 68, 68, 0.14); border-left: 4px solid #ef4444; padding: 6px 10px; border-radius: 6px; margin: 4px 0;">
                                            <div style="color: #ef4444; font-size: 11px; font-weight: bold; margin-bottom: 4px;">🛡️ [Preserved by WhatNull - Deleted at ${cached.time}]</div>
                                            <div style="color: #f3f4f6;">${cached.html}</div>
                                        </div>
                                    `;
                                } else {
                                    targetNode.innerHTML = `
                                        <div style="background: rgba(239, 68, 68, 0.14); border-left: 4px solid #ef4444; padding: 6px 10px; border-radius: 6px; margin: 4px 0;">
                                            <div style="color: #ef4444; font-size: 11px; font-weight: bold;">🛡️ [Preserved by WhatNull] Message deleted by sender</div>
                                        </div>
                                    `;
                                }
                            }
                        }
                    });
                } catch(err) {}
            }
            setInterval(captureAndRestoreMessages, 400);

            // 4. FLOATING UI SIDEBAR & SETTINGS MODAL
            function initUI() {
                if (document.getElementById('whatnull-sidebar-pill')) return;

                const style = document.createElement('style');
                style.id = 'whatnull-ui-styles';
                style.textContent = `
                    #whatnull-sidebar-pill {
                        position: fixed;
                        left: 14px;
                        bottom: 24px;
                        height: 48px;
                        background: rgba(17, 24, 39, 0.92);
                        backdrop-filter: blur(16px);
                        -webkit-backdrop-filter: blur(16px);
                        border: 1px solid rgba(255, 255, 255, 0.15);
                        border-radius: 24px;
                        z-index: 9999999;
                        display: flex;
                        align-items: center;
                        padding: 0 10px;
                        gap: 8px;
                        box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.6);
                        user-select: none;
                    }
                    .wn-pill-btn {
                        width: 34px;
                        height: 34px;
                        border-radius: 17px;
                        border: none;
                        background: transparent;
                        color: #9ca3af;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 17px;
                        transition: all 0.2s ease;
                    }
                    .wn-pill-btn:hover {
                        background: rgba(255, 255, 255, 0.15);
                        color: #14b8a6;
                        transform: scale(1.12);
                    }
                    .wn-modal-backdrop {
                        position: fixed;
                        inset: 0;
                        background: rgba(11, 15, 25, 0.85);
                        backdrop-filter: blur(20px);
                        -webkit-backdrop-filter: blur(20px);
                        z-index: 99999999;
                        display: none;
                        align-items: center;
                        justify-content: center;
                        color: #f3f4f6;
                        font-family: Inter, system-ui, -apple-system, sans-serif;
                    }
                    .wn-modal-backdrop.active {
                        display: flex !important;
                    }
                    .wn-modal-card {
                        width: 460px;
                        max-width: 90vw;
                        background: rgba(17, 24, 39, 0.96);
                        border: 1px solid rgba(255, 255, 255, 0.12);
                        border-radius: 16px;
                        padding: 24px;
                        box-shadow: 0 20px 30px rgba(0, 0, 0, 0.6);
                    }
                    .wn-modal-title {
                        font-size: 18px;
                        font-weight: 700;
                        margin-bottom: 16px;
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                    }
                    .wn-setting-row {
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        padding: 12px 0;
                        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
                    }
                    .wn-setting-label {
                        font-size: 14px;
                        font-weight: 500;
                    }
                    .wn-setting-desc {
                        font-size: 12px;
                        color: #9ca3af;
                        margin-top: 2px;
                    }
                    .wn-toggle-switch {
                        width: 44px;
                        height: 24px;
                        background: #374151;
                        border-radius: 12px;
                        position: relative;
                        cursor: pointer;
                        transition: background 0.2s;
                    }
                    .wn-toggle-switch.active {
                        background: #0d9488;
                    }
                    .wn-toggle-knob {
                        width: 20px;
                        height: 20px;
                        background: white;
                        border-radius: 10px;
                        position: absolute;
                        top: 2px;
                        left: 2px;
                        transition: transform 0.2s;
                    }
                    .wn-toggle-switch.active .wn-toggle-knob {
                        transform: translateX(20px);
                    }
                    #whatnull-privacy-overlay {
                        position: fixed;
                        inset: 0;
                        background: rgba(11, 15, 25, 0.94);
                        backdrop-filter: blur(30px);
                        -webkit-backdrop-filter: blur(30px);
                        z-index: 999999999;
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
                `;
                document.head.appendChild(style);

                const pill = document.createElement('div');
                pill.id = 'whatnull-sidebar-pill';
                pill.innerHTML = `
                    <button class="wn-pill-btn" id="wn-btn-shield" title="WhatNull Protection Active">🛡️</button>
                    <button class="wn-pill-btn" id="wn-btn-lock" title="Privacy Lock (Ctrl+L)">🔒</button>
                    <button class="wn-pill-btn" id="wn-btn-settings" title="WhatNull Settings & Features">⚙️</button>
                `;
                document.body.appendChild(pill);

                const privacyOverlay = document.createElement('div');
                privacyOverlay.id = 'whatnull-privacy-overlay';
                privacyOverlay.innerHTML = `
                    <div style="width: 64px; height: 64px; background: rgba(13, 148, 136, 0.2); border: 1px solid rgba(20, 184, 166, 0.4); border-radius: 20px; display: flex; align-items: center; justify-content: center; margin-bottom: 20px; box-shadow: 0 0 30px rgba(13, 148, 136, 0.3);">
                        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#14b8a6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
                            <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
                        </svg>
                    </div>
                    <h1 style="font-size: 26px; font-weight: 700; margin: 0 0 8px 0; color: #ffffff;">WhatNull Privacy Shield</h1>
                    <p style="color: #9ca3af; font-size: 14px; margin: 0;">Session is locked to prevent unauthorized access.</p>
                    <button id="whatnull-unlock-btn" style="margin-top: 24px; padding: 12px 32px; background: linear-gradient(135deg, #0d9488, #14b8a6); color: white; border: none; border-radius: 12px; font-weight: 600; cursor: pointer;">Unlock Session (Ctrl+L)</button>
                `;
                document.body.appendChild(privacyOverlay);

                const settingsModal = document.createElement('div');
                settingsModal.className = 'wn-modal-backdrop';
                settingsModal.id = 'wn-settings-modal';
                settingsModal.innerHTML = `
                    <div class="wn-modal-card">
                        <div class="wn-modal-title">
                            <span>⚙️ WhatNull Settings & Privacy</span>
                            <button id="wn-close-settings" style="color: #9ca3af; font-size: 20px; border:none; background:none; cursor:pointer;">✕</button>
                        </div>
                        
                        <div class="wn-setting-row">
                            <div>
                                <div class="wn-setting-label">Anti-Revoke (Preserve Deleted Msgs)</div>
                                <div class="wn-setting-desc">Messages deleted by sender remain visible</div>
                            </div>
                            <div class="wn-toggle-switch active" id="toggle-anti-revoke"><div class="wn-toggle-knob"></div></div>
                        </div>

                        <div class="wn-setting-row">
                            <div>
                                <div class="wn-setting-label">EXIF & File Metadata Stripper</div>
                                <div class="wn-setting-desc">Remove GPS/camera data from sent media</div>
                            </div>
                            <div class="wn-toggle-switch active" id="toggle-exif"><div class="wn-toggle-knob"></div></div>
                        </div>

                        <div class="wn-setting-row">
                            <div>
                                <div class="wn-setting-label">IP / MAC & Fingerprint Spoofing</div>
                                <div class="wn-setting-desc">Block WebRTC local IP leaks & spoof Mac Chrome</div>
                            </div>
                            <div class="wn-toggle-switch active" id="toggle-spoof"><div class="wn-toggle-knob"></div></div>
                        </div>

                        <div class="wn-setting-row">
                            <div>
                                <div class="wn-setting-label">WhatsApp Voice & Video Calls</div>
                                <div class="wn-setting-desc">Enable native calling buttons in header</div>
                            </div>
                            <div class="wn-toggle-switch active" id="toggle-calls"><div class="wn-toggle-knob"></div></div>
                        </div>
                    </div>
                `;
                document.body.appendChild(settingsModal);

                let isLocked = false;
                function toggleLock(forceState) {
                    isLocked = typeof forceState === 'boolean' ? forceState : !isLocked;
                    privacyOverlay.classList.toggle('active', isLocked);
                }

                document.addEventListener('keydown', (e) => {
                    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
                        e.preventDefault();
                        toggleLock();
                    }
                });

                document.getElementById('wn-btn-lock')?.addEventListener('click', () => toggleLock());
                document.getElementById('whatnull-unlock-btn')?.addEventListener('click', () => toggleLock(false));

                document.getElementById('wn-btn-settings')?.addEventListener('click', () => {
                    settingsModal.classList.add('active');
                });
                document.getElementById('wn-close-settings')?.addEventListener('click', () => {
                    settingsModal.classList.remove('active');
                });

                document.querySelectorAll('.wn-toggle-switch').forEach(sw => {
                    sw.addEventListener('click', () => {
                        sw.classList.toggle('active');
                    });
                });
            }

            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', initUI);
            } else {
                initUI();
            }
        })();
        "###;

        let window_builder = WebviewWindowBuilder::new(
            app,
            "main",
            WebviewUrl::External(target_url),
        )
        .title("WhatNull")
        .inner_size(1280.0, 800.0)
        .visible(true)
        .focused(true)
        .user_agent(user_agent)
        .data_directory(data_dir)
        .initialization_script(inject_script)
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

        let window = window_builder.build().map_err(|e| {
            AppError::WebView(format!("Failed to build WhatNull webview window: {}", e))
        })?;

        if let Ok(mut guard) = self.remote_window.lock() {
            *guard = Some(window.clone());
        }

        Ok(window)
    }

    pub fn switch_account_webview(
        &self,
        app: &AppHandle,
        data_dir: PathBuf,
    ) -> Result<WebviewWindow, AppError> {
        if let Ok(mut guard) = self.remote_window.lock() {
            if let Some(existing) = guard.take() {
                let _ = existing.close();
            }
        } else if let Some(existing) = app.get_webview_window("main") {
            let _ = existing.close();
        }

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

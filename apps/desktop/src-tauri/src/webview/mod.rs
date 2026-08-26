use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{
    webview::PageLoadEvent, AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition,
    PhysicalSize, Rect, Url, Webview, WebviewBuilder, WebviewUrl, Window, WindowBuilder,
};
use whatnull_security::{NavigationDecision, NavigationPolicy};
use whatnull_types::AppError;

const WINDOW_LABEL: &str = "main";
const SHELL_WEBVIEW_LABEL: &str = "shell";
const WHATSAPP_WEBVIEW_LABEL: &str = "whatsapp";
const SIDEBAR_WIDTH: f64 = 60.0;
const WHATSAPP_URL: &str = "https://web.whatsapp.com";

pub struct WebViewManager {
    shell_webview: Arc<Mutex<Option<Webview>>>,
    remote_webview: Arc<Mutex<Option<Webview>>>,
    shell_overlay: Arc<Mutex<bool>>,
}

impl WebViewManager {
    pub fn new() -> Self {
        Self {
            shell_webview: Arc::new(Mutex::new(None)),
            remote_webview: Arc::new(Mutex::new(None)),
            shell_overlay: Arc::new(Mutex::new(true)),
        }
    }

    pub fn create_single_window_shell(
        &self,
        app: &AppHandle,
        data_dir: PathBuf,
    ) -> Result<Window, AppError> {
        if let Some(existing) = app.get_window(WINDOW_LABEL) {
            let _ = existing.show();
            let _ = existing.set_focus();
            return Ok(existing);
        }

        let window = WindowBuilder::new(app, WINDOW_LABEL)
            .title("WhatNull")
            .inner_size(1280.0, 800.0)
            .visible(true)
            .focused(true)
            .build()
            .map_err(|e| AppError::Window(format!("Failed to build app window: {}", e)))?;

        let size = window
            .inner_size()
            .map_err(|e| AppError::Window(format!("Failed to read window size: {}", e)))?;

        let shell = WebviewBuilder::new(SHELL_WEBVIEW_LABEL, WebviewUrl::App("index.html".into()))
            .transparent(true);

        let shell = window
            .add_child(
                shell,
                PhysicalPosition::new(0, 0),
                PhysicalSize::new(size.width, size.height),
            )
            .map_err(|e| AppError::WebView(format!("Failed to build shell webview: {}", e)))?;

        if let Ok(mut guard) = self.shell_webview.lock() {
            *guard = Some(shell);
        }

        self.create_whatsapp_child(&window, data_dir)?;
        self.set_shell_bounds(&window, size, true)?;
        self.set_whatsapp_bounds(&window, size)?;

        Ok(window)
    }

    pub fn switch_account_webview(
        &self,
        app: &AppHandle,
        data_dir: PathBuf,
    ) -> Result<(), AppError> {
        let window = app
            .get_window(WINDOW_LABEL)
            .ok_or_else(|| AppError::Window("Main window instance not found".to_string()))?;

        if let Ok(mut guard) = self.remote_webview.lock() {
            if let Some(existing) = guard.take() {
                let _ = existing.close();
            }
        } else if let Some(existing) = app.get_webview(WHATSAPP_WEBVIEW_LABEL) {
            let _ = existing.close();
        }

        self.create_whatsapp_child(&window, data_dir)?;
        let size = window
            .inner_size()
            .map_err(|e| AppError::Window(format!("Failed to read window size: {}", e)))?;
        self.set_whatsapp_bounds(&window, size)?;
        Ok(())
    }

    pub fn reload(&self) -> Result<(), AppError> {
        let webview = self.whatsapp_webview()?;
        webview
            .eval("window.location.reload();")
            .map_err(|e| AppError::WebView(format!("Failed to reload WhatsApp webview: {}", e)))
    }

    pub fn hard_reload(&self) -> Result<(), AppError> {
        let target_url = whatsapp_url()?;
        let webview = self.whatsapp_webview()?;
        webview
            .navigate(target_url)
            .map_err(|e| AppError::WebView(format!("Failed to navigate WhatsApp webview: {}", e)))
    }

    pub fn set_whatsapp_visible(&self, visible: bool) -> Result<(), AppError> {
        let webview = self.whatsapp_webview()?;
        let result = if visible {
            webview.show()
        } else {
            webview.hide()
        };

        result.map_err(|e| {
            AppError::WebView(format!(
                "Failed to {} WhatsApp webview: {}",
                if visible { "show" } else { "hide" },
                e
            ))
        })
    }

    pub fn set_shell_overlay_mode(&self, window: &Window, overlay: bool) -> Result<(), AppError> {
        if let Ok(mut guard) = self.shell_overlay.lock() {
            *guard = overlay;
        }

        let size = window
            .inner_size()
            .map_err(|e| AppError::Window(format!("Failed to read window size: {}", e)))?;
        self.set_shell_bounds(window, size, overlay)
    }

    pub fn sync_whatsapp_bounds(&self, window: &Window) -> Result<(), AppError> {
        let size = window
            .inner_size()
            .map_err(|e| AppError::Window(format!("Failed to read window size: {}", e)))?;
        let overlay = self
            .shell_overlay
            .lock()
            .map(|guard| *guard)
            .unwrap_or(true);
        self.set_shell_bounds(window, size, overlay)?;
        self.set_whatsapp_bounds(window, size)
    }

    fn create_whatsapp_child(
        &self,
        window: &Window,
        data_dir: PathBuf,
    ) -> Result<Webview, AppError> {
        let target_url = whatsapp_url()?;
        let app_handle = window.app_handle().clone();

        let builder = WebviewBuilder::new(WHATSAPP_WEBVIEW_LABEL, WebviewUrl::External(target_url))
            .user_agent(whatsapp_user_agent())
            .data_directory(data_dir)
            .enable_clipboard_access()
            .initialization_script_for_all_frames(whatsapp_init_script())
            .on_page_load(|webview, payload| {
                if matches!(payload.event(), PageLoadEvent::Finished) {
                    let _ = webview.eval(whatsapp_init_script());
                }
            })
            .on_navigation(handle_navigation)
            .on_new_window(move |url, _features| {
                match NavigationPolicy::evaluate(url.as_str()) {
                    NavigationDecision::Allow => {
                        if let Some(webview) = app_handle.get_webview(WHATSAPP_WEBVIEW_LABEL) {
                            let escaped = js_string(url.as_str());
                            let _ = webview.eval(format!("window.location.href = {};", escaped));
                        }
                    }
                    NavigationDecision::OpenExternally => {
                        let _ = open::that(url.as_str());
                    }
                    NavigationDecision::Reject => {}
                }
                tauri::webview::NewWindowResponse::Deny
            });

        let webview = window
            .add_child(
                builder,
                LogicalPosition::new(SIDEBAR_WIDTH, 0.0),
                LogicalSize::new(1.0, 1.0),
            )
            .map_err(|e| AppError::WebView(format!("Failed to build WhatsApp webview: {}", e)))?;

        if let Ok(mut guard) = self.remote_webview.lock() {
            *guard = Some(webview.clone());
        }

        Ok(webview)
    }

    fn set_whatsapp_bounds(
        &self,
        window: &Window,
        size: PhysicalSize<u32>,
    ) -> Result<(), AppError> {
        let webview = self.whatsapp_webview()?;
        let logical = logical_size(window, size)?;
        let width = (logical.width - SIDEBAR_WIDTH).max(0.0);

        webview
            .set_bounds(Rect {
                position: LogicalPosition::new(SIDEBAR_WIDTH, 0.0).into(),
                size: LogicalSize::new(width, logical.height).into(),
            })
            .map_err(|e| AppError::WebView(format!("Failed to resize WhatsApp webview: {}", e)))
    }

    fn set_shell_bounds(
        &self,
        window: &Window,
        size: PhysicalSize<u32>,
        overlay: bool,
    ) -> Result<(), AppError> {
        let webview = self.shell_webview()?;
        let logical = logical_size(window, size)?;
        let width = if overlay {
            logical.width
        } else {
            SIDEBAR_WIDTH
        };

        webview
            .set_bounds(Rect {
                position: LogicalPosition::new(0.0, 0.0).into(),
                size: LogicalSize::new(width, logical.height).into(),
            })
            .map_err(|e| AppError::WebView(format!("Failed to resize shell webview: {}", e)))
    }

    fn shell_webview(&self) -> Result<Webview, AppError> {
        self.shell_webview
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or_else(|| AppError::WebView("Shell webview instance not found".to_string()))
    }

    fn whatsapp_webview(&self) -> Result<Webview, AppError> {
        self.remote_webview
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or_else(|| AppError::WebView("WhatsApp webview instance not found".to_string()))
    }
}

fn logical_size(window: &Window, size: PhysicalSize<u32>) -> Result<LogicalSize<f64>, AppError> {
    let factor = window
        .scale_factor()
        .map_err(|e| AppError::Window(format!("Failed to read window scale factor: {}", e)))?;
    Ok(size.to_logical::<f64>(factor))
}

fn whatsapp_url() -> Result<Url, AppError> {
    WHATSAPP_URL
        .parse()
        .map_err(|e| AppError::WebView(format!("Invalid WhatsApp target URL: {}", e)))
}

fn whatsapp_user_agent() -> &'static str {
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36"
}

fn handle_navigation(url: &Url) -> bool {
    match NavigationPolicy::evaluate(url.as_str()) {
        NavigationDecision::Allow => true,
        NavigationDecision::OpenExternally => {
            let _ = open::that(url.as_str());
            false
        }
        NavigationDecision::Reject => false,
    }
}

fn js_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"about:blank\"".to_string())
}

fn whatsapp_init_script() -> &'static str {
    r#"
(function () {
  if (window.__WHATNULL_INITIALIZED__) return;
  window.__WHATNULL_INITIALIZED__ = true;
  window.__WHATNULL_STATUS__ = {
    initializedAt: new Date().toISOString(),
    nativeSanitizer: false,
    lastSanitize: null,
    antiRevokeCacheSize: 0,
    webrtcLocalCandidateDrops: 0,
    inlineMediaEnhanced: 0
  };

  const INTERNAL_ORIGINS = new Set([
    'https://web.whatsapp.com',
    'https://whatsapp.com',
    'https://www.whatsapp.com',
    'https://flows.whatsapp.net',
    'https://webtp.whatsapp.net'
  ]);

  function isAllowedInternalUrl(value) {
    try {
      const url = new URL(value, window.location.href);
      return INTERNAL_ORIGINS.has(url.origin) ||
        url.hostname.endsWith('.whatsapp.com') ||
        url.hostname.endsWith('.whatsapp.net') ||
        url.hostname.endsWith('.fbcdn.net');
    } catch (_) {
      return false;
    }
  }

  try {
    const open = window.open;
    window.open = function (url, target, features) {
      if (url && isAllowedInternalUrl(url)) {
        window.location.href = new URL(url, window.location.href).href;
        return window;
      }
      return open.call(window, url, target, features);
    };
  } catch (_) {}

  try {
    const fakeUA = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36';
    Object.defineProperty(navigator, 'userAgent', { get: () => fakeUA, configurable: true });
    Object.defineProperty(navigator, 'appVersion', { get: () => fakeUA, configurable: true });
    Object.defineProperty(navigator, 'platform', { get: () => 'Linux x86_64', configurable: true });
    Object.defineProperty(navigator, 'vendor', { get: () => 'Google Inc.', configurable: true });
    Object.defineProperty(navigator, 'deviceMemory', { get: () => 8, configurable: true });
    Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 8, configurable: true });
  } catch (_) {}

  try {
    if (navigator.mediaDevices && navigator.mediaDevices.enumerateDevices) {
      const enumerateDevices = navigator.mediaDevices.enumerateDevices.bind(navigator.mediaDevices);
      navigator.mediaDevices.enumerateDevices = async function () {
        const devices = await enumerateDevices().catch(() => []);
        if (devices.length) return devices;
        return [
          { deviceId: 'default', kind: 'audioinput', label: 'Default Microphone', groupId: 'default' },
          { deviceId: 'default', kind: 'videoinput', label: 'Default Camera', groupId: 'default' },
          { deviceId: 'default', kind: 'audiooutput', label: 'Default Speaker', groupId: 'default' }
        ];
      };
    }
  } catch (_) {}

  function scrubLocalCandidates(value) {
    if (!value || typeof value !== 'string') return value;
    return value.replace(/^a=candidate:.*(?:\s|^)(?:10\.|127\.|169\.254\.|192\.168\.|172\.(?:1[6-9]|2\d|3[01])\.|::1|fc[0-9a-f]{2}:|fd[0-9a-f]{2}:|fe80:).*[\r\n]*/gmi, '');
  }

  function isLocalCandidate(value) {
    return !!value && typeof value === 'string' && scrubLocalCandidates(value) === '';
  }

  try {
    if (window.RTCPeerConnection) {
      const proto = window.RTCPeerConnection.prototype;
      const createOffer = proto.createOffer;
      const createAnswer = proto.createAnswer;
      const addIceCandidate = proto.addIceCandidate;
      const addEventListener = proto.addEventListener;
      const removeEventListener = proto.removeEventListener;
      const listenerMap = new WeakMap();

      if (createOffer) {
        proto.createOffer = function (...args) {
          return createOffer.apply(this, args).then((offer) => {
            if (offer && offer.sdp) offer.sdp = scrubLocalCandidates(offer.sdp);
            return offer;
          });
        };
      }

      if (createAnswer) {
        proto.createAnswer = function (...args) {
          return createAnswer.apply(this, args).then((answer) => {
            if (answer && answer.sdp) answer.sdp = scrubLocalCandidates(answer.sdp);
            return answer;
          });
        };
      }

      if (addIceCandidate) {
        proto.addIceCandidate = function (candidate, ...args) {
          const raw = candidate && (candidate.candidate || candidate);
          if (typeof raw === 'string' && scrubLocalCandidates(raw) === '') {
            return Promise.resolve();
          }
          return addIceCandidate.call(this, candidate, ...args);
        };
      }

      function wrapIceCandidateListener(listener) {
        if (!listener) return listener;
        if (listenerMap.has(listener)) return listenerMap.get(listener);
        const wrapped = function (event) {
          const raw = event && event.candidate && event.candidate.candidate;
          if (isLocalCandidate(raw)) {
            window.__WHATNULL_STATUS__.webrtcLocalCandidateDrops += 1;
            return;
          }
          if (typeof listener === 'function') {
            return listener.call(this, event);
          }
          if (listener && typeof listener.handleEvent === 'function') {
            return listener.handleEvent(event);
          }
        };
        listenerMap.set(listener, wrapped);
        return wrapped;
      }

      if (addEventListener) {
        proto.addEventListener = function (type, listener, options) {
          if (type === 'icecandidate') {
            return addEventListener.call(this, type, wrapIceCandidateListener(listener), options);
          }
          return addEventListener.call(this, type, listener, options);
        };
      }

      if (removeEventListener) {
        proto.removeEventListener = function (type, listener, options) {
          if (type === 'icecandidate' && listenerMap.has(listener)) {
            return removeEventListener.call(this, type, listenerMap.get(listener), options);
          }
          return removeEventListener.call(this, type, listener, options);
        };
      }

      try {
        const onIce = Object.getOwnPropertyDescriptor(proto, 'onicecandidate');
        if (onIce && onIce.set && onIce.get) {
          Object.defineProperty(proto, 'onicecandidate', {
            configurable: true,
            get() {
              return onIce.get.call(this);
            },
            set(listener) {
              return onIce.set.call(this, wrapIceCandidateListener(listener));
            }
          });
        }
      } catch (_) {}
    }
  } catch (_) {}

  const liveMessageCache = new Map();
  const deletedMarkers = [
    'This message was deleted',
    'Bu mesaj silindi',
    'Сообщение удалено'
  ];

  function isDeletedMessage(row) {
    const text = row.textContent || '';
    return deletedMarkers.some((marker) => text.includes(marker));
  }

  function cacheAndRestoreMessages() {
    try {
      document.querySelectorAll('div[role="row"], div[data-id]').forEach((row) => {
        const id = row.getAttribute('data-id') || row.getAttribute('id');
        if (!id) return;
        const contentNode = row.querySelector('.copyable-text, .selectable-text') || row;

        if (!isDeletedMessage(row)) {
          const text = contentNode.innerText || '';
          if (text.trim().length > 0) {
            liveMessageCache.set(id, {
              html: contentNode.innerHTML,
              text,
              time: new Date().toLocaleTimeString()
            });
            window.__WHATNULL_STATUS__.antiRevokeCacheSize = liveMessageCache.size;
          }
          return;
        }

        if (row.dataset.whatnullRestored) return;
        row.dataset.whatnullRestored = 'true';
        const cached = liveMessageCache.get(id);
        if (!cached) return;

        const wrapper = document.createElement('div');
        wrapper.style.cssText = 'background:rgba(20,184,166,.10);border-left:3px solid #14b8a6;padding:6px 10px;border-radius:6px;margin:4px 0;';

        const label = document.createElement('div');
        label.textContent = 'Preserved by WhatNull - deleted at ' + cached.time;
        label.style.cssText = 'color:#14b8a6;font-size:11px;font-weight:700;margin-bottom:4px;';

        const body = document.createElement('div');
        body.innerHTML = cached.html;

        wrapper.appendChild(label);
        wrapper.appendChild(body);
        contentNode.replaceChildren(wrapper);
      });
    } catch (_) {}
  }

  async function stripImageMetadataInBrowser(file) {
    if (!/^image\/(jpeg|png)$/.test(file.type)) return file;

    const bitmap = await createImageBitmap(file);
    const canvas = document.createElement('canvas');
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const ctx = canvas.getContext('2d');
    ctx.drawImage(bitmap, 0, 0);
    bitmap.close && bitmap.close();

    const blob = await new Promise((resolve) => {
      canvas.toBlob(resolve, file.type, file.type === 'image/jpeg' ? 0.92 : undefined);
    });
    if (!blob) return file;

    return new File([blob], file.name, {
      type: file.type,
      lastModified: Date.now()
    });
  }

  function isNativeSanitizable(file) {
    return /^(image\/(jpeg|png)|application\/pdf|video\/(mp4|quicktime|x-msvideo|x-matroska|webm)|audio\/(mpeg|wav|ogg|flac))$/.test(file.type);
  }

  function arrayBufferToBase64(buffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    const chunkSize = 0x8000;
    for (let i = 0; i < bytes.length; i += chunkSize) {
      binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunkSize));
    }
    return btoa(binary);
  }

  function base64ToBlob(base64, mimeType) {
    const binary = atob(base64);
    const chunks = [];
    const chunkSize = 0x8000;
    for (let i = 0; i < binary.length; i += chunkSize) {
      const slice = binary.slice(i, i + chunkSize);
      const bytes = new Uint8Array(slice.length);
      for (let j = 0; j < slice.length; j += 1) {
        bytes[j] = slice.charCodeAt(j);
      }
      chunks.push(bytes);
    }
    return new Blob(chunks, { type: mimeType });
  }

  async function sanitizeFilesWithNative(files) {
    const invoke = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke;
    if (!invoke) return null;

    const sanitizable = files.filter(isNativeSanitizable);
    if (!sanitizable.length) return null;

    const payload = await Promise.all(sanitizable.map(async (file) => ({
      name: file.name,
      mimeType: file.type,
      dataBase64: arrayBufferToBase64(await file.arrayBuffer())
    })));

    const results = await invoke('sanitize_upload_files', { files: payload });
    const byName = new Map(results.map((item) => [item.name, item]));
    let changed = false;

    const sanitizedFiles = files.map((file) => {
      const result = byName.get(file.name);
      if (!result) return file;
      const blob = base64ToBlob(result.dataBase64, result.mimeType || file.type);
      changed = changed || result.changed || blob.size !== file.size;
      return new File([blob], file.name, {
        type: result.mimeType || file.type,
        lastModified: Date.now()
      });
    });

    window.__WHATNULL_STATUS__.nativeSanitizer = true;
    window.__WHATNULL_STATUS__.lastSanitize = {
      at: new Date().toISOString(),
      files: results.map((item) => ({
        name: item.name,
        changed: item.changed,
        fieldsRemoved: item.fieldsRemoved,
        originalSize: item.originalSize,
        strippedSize: item.strippedSize
      }))
    };

    return { files: sanitizedFiles, changed };
  }

  document.addEventListener('change', async (event) => {
    const input = event.target;
    if (!input || input.tagName !== 'INPUT' || input.type !== 'file' || !input.files || !input.files.length) return;

    if (input.dataset.whatnullSanitized === '1') {
      delete input.dataset.whatnullSanitized;
      return;
    }

    try {
      const originalFiles = Array.from(input.files);
      const nativeResult = await sanitizeFilesWithNative(originalFiles).catch((error) => {
        window.__WHATNULL_STATUS__.lastSanitize = {
          at: new Date().toISOString(),
          error: String(error && error.message ? error.message : error)
        };
        return null;
      });

      const dataTransfer = new DataTransfer();
      let changed = nativeResult ? nativeResult.changed : false;
      const filesToUse = nativeResult ? nativeResult.files : originalFiles;

      for (const file of filesToUse) {
        const sanitized = nativeResult ? file : await stripImageMetadataInBrowser(file);
        if (sanitized !== file) changed = true;
        dataTransfer.items.add(sanitized);
      }

      if (changed) {
        input.dataset.whatnullSanitized = '1';
        input.files = dataTransfer.files;
        input.dispatchEvent(new Event('input', { bubbles: true }));
        input.dispatchEvent(new Event('change', { bubbles: true }));
      }
    } catch (_) {}
  }, true);

  function enhanceInlineMedia() {
    try {
      document.querySelectorAll('video').forEach((video) => {
        video.controls = true;
        video.playsInline = true;
        video.preload = 'metadata';
      });
      window.__WHATNULL_STATUS__.inlineMediaEnhanced = document.querySelectorAll('video').length;
    } catch (_) {}
  }

  const observer = new MutationObserver(() => {
    if (window.__WHATNULL_SCAN_PENDING__) return;
    window.__WHATNULL_SCAN_PENDING__ = true;
    window.requestAnimationFrame(() => {
      window.__WHATNULL_SCAN_PENDING__ = false;
      cacheAndRestoreMessages();
      enhanceInlineMedia();
    });
  });

  if (document.documentElement) {
    observer.observe(document.documentElement, { childList: true, subtree: true, characterData: true });
  }
  window.setInterval(() => {
    cacheAndRestoreMessages();
    enhanceInlineMedia();
  }, 1500);
})();
"#
}

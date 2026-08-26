use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Security error: {0}")]
    Security(String),
    #[error("Navigation error: {0}")]
    Navigation(String),
    #[error("WebView error: {0}")]
    WebView(String),
    #[error("Window error: {0}")]
    Window(String),
    #[error("Platform error: {0}")]
    Platform(String),
    #[error("Notification error: {0}")]
    Notification(String),
    #[error("Update error: {0}")]
    Update(String),
    #[error("Session error: {0}")]
    Session(String),
    #[error("Account error: {0}")]
    Account(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "event", content = "data")]
pub enum AppEvent {
    ConfigChanged(serde_json::Value),
    AccountChanged(String),
    ConnectionChanged(String),
    PrivacyChanged(bool),
    UpdateAvailable(String),
    WebViewFailed(String),
    WindowStateChanged(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Uninitialized,
    Loading,
    AuthenticationRequired,
    Authenticated,
    Offline,
    Reconnecting,
    Expired,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AccountProfile {
    pub id: String,
    pub display_name: String,
    pub storage_partition: String,
    pub created_at: u64,
    pub last_used_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CloseBehavior {
    Quit,
    HideToTray,
    Ask,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPrivacy {
    FullPreview,
    SenderOnly,
    Generic,
    Disabled,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub tag: Option<String>,
}

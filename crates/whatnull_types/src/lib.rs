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
    pub avatar_color: String,
    pub created_at: u64,
    pub last_used_at: u64,
}

pub const MAX_PROFILE_ID_LEN: usize = 80;
pub const MAX_DISPLAY_NAME_LEN: usize = 64;

pub fn is_valid_profile_id(profile_id: &str) -> bool {
    !profile_id.is_empty()
        && profile_id.len() <= MAX_PROFILE_ID_LEN
        && profile_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn is_valid_display_name(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty() && trimmed.chars().count() <= MAX_DISPLAY_NAME_LEN
}

pub fn is_valid_avatar_color(color: &str) -> bool {
    color.len() == 7 && color.starts_with('#') && color[1..].bytes().all(|b| b.is_ascii_hexdigit())
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
    Dark,
    Light,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct PrivacyState {
    pub is_blurred: bool,
    pub is_locked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_ids_accept_safe_characters_only() {
        assert!(is_valid_profile_id("default"));
        assert!(is_valid_profile_id("profile-1787774426"));
        assert!(is_valid_profile_id("work_2"));
        assert!(!is_valid_profile_id(""));
        assert!(!is_valid_profile_id("../escape"));
        assert!(!is_valid_profile_id("has space"));
        assert!(!is_valid_profile_id("slash/inside"));
        assert!(!is_valid_profile_id(&"a".repeat(MAX_PROFILE_ID_LEN + 1)));
    }

    #[test]
    fn display_names_are_measured_in_characters_not_bytes() {
        assert!(is_valid_display_name("İş"));
        assert!(is_valid_display_name(&"ğ".repeat(MAX_DISPLAY_NAME_LEN)));
        assert!(!is_valid_display_name(
            &"ğ".repeat(MAX_DISPLAY_NAME_LEN + 1)
        ));
        assert!(!is_valid_display_name("   "));
        assert!(!is_valid_display_name(""));
    }

    #[test]
    fn avatar_colors_require_hex_rgb() {
        assert!(is_valid_avatar_color("#10b981"));
        assert!(is_valid_avatar_color("#ABCDEF"));
        assert!(!is_valid_avatar_color("10b981"));
        assert!(!is_valid_avatar_color("#10b98"));
        assert!(!is_valid_avatar_color("#10b98g"));
        assert!(!is_valid_avatar_color("red"));
    }
}

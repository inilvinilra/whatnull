use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use whatnull_types::{
    is_valid_avatar_color, is_valid_display_name, is_valid_profile_id, AccountProfile, AppError,
    CloseBehavior, NotificationPrivacy, Theme,
};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const MIN_ZOOM_LEVEL: f64 = 0.5;
pub const MAX_ZOOM_LEVEL: f64 = 3.0;
pub const MAX_LOCK_TIMEOUT_MINS: u32 = 1440;
pub const MAX_LANGUAGE_LEN: usize = 16;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppConfig {
    pub schema_version: u32,
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub privacy: PrivacyConfig,
    pub notifications: NotificationsConfig,
    pub downloads: DownloadsConfig,
    pub startup: StartupConfig,
    pub accounts: AccountsConfig,
    pub advanced: AdvancedConfig,
    #[serde(default)]
    pub permissions: PermissionsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct PermissionsConfig {
    pub microphone: bool,
    pub camera: bool,
    pub screen_share: bool,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self {
            microphone: true,
            camera: true,
            screen_share: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GeneralConfig {
    pub close_behavior: CloseBehavior,
    pub start_minimized: bool,
    pub remember_window_position: bool,
    pub zoom_level: f64,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppearanceConfig {
    pub theme: Theme,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrivacyConfig {
    pub telemetry: bool,
    pub analytics: bool,
    pub crash_upload: bool,
    pub message_logging: bool,
    pub contact_logging: bool,
    pub privacy_mode_enabled: bool,
    pub blur_on_unfocus: bool,
    pub blur_on_minimize: bool,
    pub lock_timeout_mins: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NotificationsConfig {
    pub enabled: bool,
    pub privacy: NotificationPrivacy,
    pub dnd_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DownloadsConfig {
    pub ask_every_time: bool,
    pub default_directory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StartupConfig {
    pub autostart: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AccountsConfig {
    pub active_profile_id: String,
    pub profiles: Vec<AccountProfile>,
}

impl Default for AccountsConfig {
    fn default() -> Self {
        Self {
            active_profile_id: "default".to_string(),
            profiles: vec![AccountProfile {
                id: "default".to_string(),
                display_name: "Primary Account".to_string(),
                storage_partition: "default".to_string(),
                avatar_color: "#10b981".to_string(),
                created_at: 0,
                last_used_at: 0,
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdvancedConfig {
    pub hardware_acceleration: bool,
    pub enable_dev_tools: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            general: GeneralConfig {
                close_behavior: CloseBehavior::HideToTray,
                start_minimized: false,
                remember_window_position: true,
                zoom_level: 1.0,
                language: "en".to_string(),
            },
            appearance: AppearanceConfig {
                theme: Theme::System,
            },
            privacy: PrivacyConfig {
                telemetry: false,
                analytics: false,
                crash_upload: false,
                message_logging: false,
                contact_logging: false,
                privacy_mode_enabled: false,
                blur_on_unfocus: true,
                blur_on_minimize: true,
                lock_timeout_mins: 15,
            },
            notifications: NotificationsConfig {
                enabled: true,
                privacy: NotificationPrivacy::FullPreview,
                dnd_enabled: false,
            },
            downloads: DownloadsConfig {
                ask_every_time: true,
                default_directory: None,
            },
            startup: StartupConfig { autostart: false },
            accounts: AccountsConfig::default(),
            advanced: AdvancedConfig {
                hardware_acceleration: true,
                enable_dev_tools: false,
            },
            permissions: PermissionsConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(AppError::Config(format!(
                "Unsupported schema version: {}",
                self.schema_version
            )));
        }

        if self.accounts.profiles.is_empty() {
            return Err(AppError::Config(
                "At least one account profile is required".to_string(),
            ));
        }

        let mut seen_ids = HashSet::new();
        for profile in &self.accounts.profiles {
            if !is_valid_profile_id(&profile.id) {
                return Err(AppError::Config(format!(
                    "Invalid profile id: {}",
                    profile.id
                )));
            }
            if !seen_ids.insert(profile.id.as_str()) {
                return Err(AppError::Config(format!(
                    "Duplicate profile id: {}",
                    profile.id
                )));
            }
            if !is_valid_profile_id(&profile.storage_partition) {
                return Err(AppError::Config(format!(
                    "Invalid storage partition for profile {}",
                    profile.id
                )));
            }
            if !is_valid_display_name(&profile.display_name) {
                return Err(AppError::Config(format!(
                    "Profile name must be between 1 and {} characters",
                    whatnull_types::MAX_DISPLAY_NAME_LEN
                )));
            }
            if !is_valid_avatar_color(&profile.avatar_color) {
                return Err(AppError::Config(format!(
                    "Avatar color must be a hex RGB value for profile {}",
                    profile.id
                )));
            }
        }

        if !seen_ids.contains(self.accounts.active_profile_id.as_str()) {
            return Err(AppError::Config(format!(
                "Active profile does not exist: {}",
                self.accounts.active_profile_id
            )));
        }

        let zoom = self.general.zoom_level;
        if !zoom.is_finite() || !(MIN_ZOOM_LEVEL..=MAX_ZOOM_LEVEL).contains(&zoom) {
            return Err(AppError::Config(format!(
                "Zoom level must be between {} and {}",
                MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL
            )));
        }

        if self.privacy.lock_timeout_mins > MAX_LOCK_TIMEOUT_MINS {
            return Err(AppError::Config(format!(
                "Lock timeout must not exceed {} minutes",
                MAX_LOCK_TIMEOUT_MINS
            )));
        }

        let language = self.general.language.as_str();
        if language.is_empty()
            || language.len() > MAX_LANGUAGE_LEN
            || !language
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(AppError::Config(format!(
                "Invalid language tag: {}",
                language
            )));
        }

        if let Some(directory) = &self.downloads.default_directory {
            if !std::path::Path::new(directory).is_absolute() {
                return Err(AppError::Config(
                    "Download directory must be an absolute path".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn repair(&mut self) {
        if self.accounts.profiles.is_empty() {
            self.accounts = AccountsConfig::default();
        }
        if !self
            .accounts
            .profiles
            .iter()
            .any(|profile| profile.id == self.accounts.active_profile_id)
        {
            self.accounts.active_profile_id = self.accounts.profiles[0].id.clone();
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    current_config: AppConfig,
}

impl ConfigManager {
    pub fn load(config_path: PathBuf) -> Result<Self, AppError> {
        if !config_path.exists() {
            let default_config = AppConfig::default();
            let manager = Self {
                config_path: config_path.clone(),
                current_config: default_config,
            };
            manager.save()?;
            return Ok(manager);
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;

        let raw_val: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse config JSON: {}", e)))?;

        let schema_version = raw_val
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let mut migrated_config = if schema_version < 1 {
            return Err(AppError::Config(
                "Unsupported config schema version".to_string(),
            ));
        } else if schema_version > 1 {
            return Err(AppError::Config(
                "Config schema version is from a newer app version".to_string(),
            ));
        } else {
            serde_json::from_value::<AppConfig>(raw_val)
                .map_err(|e| AppError::Config(format!("Invalid config structure: {}", e)))?
        };

        migrated_config.repair();
        migrated_config.validate()?;

        Ok(Self {
            config_path,
            current_config: migrated_config,
        })
    }

    pub fn get(&self) -> &AppConfig {
        &self.current_config
    }

    pub fn update<F>(&mut self, updater: F) -> Result<(), AppError>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut new_config = self.current_config.clone();
        updater(&mut new_config);
        new_config.validate()?;
        self.current_config = new_config;
        self.save()
    }

    pub fn save(&self) -> Result<(), AppError> {
        let parent = self.config_path.parent().ok_or_else(|| {
            AppError::Config("Config path does not have a parent directory".to_string())
        })?;

        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Config(format!("Failed to create config folder: {}", e)))?;
        }

        let temp_path = self.config_path.with_extension("tmp");
        let serialized = serde_json::to_string_pretty(&self.current_config)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;

        {
            let mut temp_file = fs::File::create(&temp_path).map_err(|e| {
                AppError::Config(format!("Failed to create temp config file: {}", e))
            })?;

            temp_file
                .write_all(serialized.as_bytes())
                .map_err(|e| AppError::Config(format!("Failed to write config content: {}", e)))?;

            temp_file
                .sync_all()
                .map_err(|e| AppError::Config(format!("Failed to sync config to disk: {}", e)))?;
        }

        fs::rename(&temp_path, &self.config_path).map_err(|e| {
            AppError::Config(format!("Failed to atomically rename config file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.schema_version, 1);
        assert!(!config.privacy.telemetry);
        assert!(!config.privacy.analytics);
        assert!(!config.privacy.crash_upload);
        assert_eq!(config.general.close_behavior, CloseBehavior::HideToTray);
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("whatnull_test_config.json");
        if config_path.exists() {
            let _ = fs::remove_file(&config_path);
        }

        let mut manager = ConfigManager::load(config_path.clone()).unwrap();
        assert!(!manager.get().privacy.telemetry);

        manager
            .update(|cfg| {
                cfg.general.language = "tr".to_string();
                cfg.privacy.telemetry = true;
            })
            .unwrap();

        let reloaded = ConfigManager::load(config_path.clone()).unwrap();
        assert_eq!(reloaded.get().general.language, "tr");
        assert!(reloaded.get().privacy.telemetry);

        let _ = fs::remove_file(&config_path);
    }

    #[test]
    fn validate_accepts_defaults() {
        assert!(AppConfig::default().validate().is_ok());
    }

    #[test]
    fn validate_rejects_dangling_active_profile() {
        let mut config = AppConfig::default();
        config.accounts.active_profile_id = "does-not-exist".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_duplicate_profile_ids() {
        let mut config = AppConfig::default();
        let duplicate = config.accounts.profiles[0].clone();
        config.accounts.profiles.push(duplicate);
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_unsafe_profile_id() {
        let mut config = AppConfig::default();
        config.accounts.profiles[0].id = "../../etc".to_string();
        config.accounts.active_profile_id = "../../etc".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_out_of_range_zoom() {
        let mut config = AppConfig::default();
        config.general.zoom_level = 42.0;
        assert!(config.validate().is_err());
        config.general.zoom_level = f64::NAN;
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_relative_download_directory() {
        let mut config = AppConfig::default();
        config.downloads.default_directory = Some("relative/path".to_string());
        assert!(config.validate().is_err());
        config.downloads.default_directory = Some("/tmp/downloads".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn repair_points_active_profile_at_an_existing_one() {
        let mut config = AppConfig::default();
        config.accounts.active_profile_id = "missing".to_string();
        config.repair();
        assert_eq!(config.accounts.active_profile_id, "default");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn repair_restores_an_empty_profile_list() {
        let mut config = AppConfig::default();
        config.accounts.profiles.clear();
        config.repair();
        assert!(!config.accounts.profiles.is_empty());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn update_refuses_to_persist_an_invalid_config() {
        let config_path = std::env::temp_dir().join("whatnull_test_invalid_update.json");
        let _ = fs::remove_file(&config_path);

        let mut manager = ConfigManager::load(config_path.clone()).unwrap();
        let result = manager.update(|cfg| {
            cfg.accounts.active_profile_id = "ghost".to_string();
        });

        assert!(result.is_err());
        assert_eq!(manager.get().accounts.active_profile_id, "default");

        let reloaded = ConfigManager::load(config_path.clone()).unwrap();
        assert_eq!(reloaded.get().accounts.active_profile_id, "default");

        let _ = fs::remove_file(&config_path);
    }

    #[test]
    fn permissions_default_to_calls_on_and_screen_share_off() {
        let permissions = AppConfig::default().permissions;
        assert!(permissions.microphone);
        assert!(permissions.camera);
        assert!(!permissions.screen_share);
    }

    #[test]
    fn a_config_written_before_permissions_existed_still_loads() {
        let mut value = serde_json::to_value(AppConfig::default()).unwrap();
        value.as_object_mut().unwrap().remove("permissions");

        let restored: AppConfig = serde_json::from_value(value).unwrap();
        assert_eq!(restored.permissions, PermissionsConfig::default());
        assert!(restored.validate().is_ok());
    }
}

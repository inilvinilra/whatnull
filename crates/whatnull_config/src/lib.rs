use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use whatnull_types::{AccountProfile, AppError, CloseBehavior, NotificationPrivacy, Theme};

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
            startup: StartupConfig {
                autostart: false,
            },
            accounts: AccountsConfig::default(),
            advanced: AdvancedConfig {
                hardware_acceleration: true,
                enable_dev_tools: false,
            },
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

        let migrated_config = if schema_version < 1 {
            return Err(AppError::Config("Unsupported config schema version".to_string()));
        } else if schema_version > 1 {
            return Err(AppError::Config("Config schema version is from a newer app version".to_string()));
        } else {
            serde_json::from_value::<AppConfig>(raw_val)
                .map_err(|e| AppError::Config(format!("Invalid config structure: {}", e)))?
        };

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
            let mut temp_file = fs::File::create(&temp_path)
                .map_err(|e| AppError::Config(format!("Failed to create temp config file: {}", e)))?;

            temp_file
                .write_all(serialized.as_bytes())
                .map_err(|e| AppError::Config(format!("Failed to write config content: {}", e)))?;

            temp_file
                .sync_all()
                .map_err(|e| AppError::Config(format!("Failed to sync config to disk: {}", e)))?;
        }

        fs::rename(&temp_path, &self.config_path)
            .map_err(|e| AppError::Config(format!("Failed to atomically rename config file: {}", e)))?;

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
        assert_eq!(config.privacy.telemetry, false);
        assert_eq!(config.privacy.analytics, false);
        assert_eq!(config.privacy.crash_upload, false);
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
        assert_eq!(manager.get().privacy.telemetry, false);

        manager.update(|cfg| {
            cfg.general.language = "tr".to_string();
            cfg.privacy.telemetry = true;
        }).unwrap();

        let reloaded = ConfigManager::load(config_path.clone()).unwrap();
        assert_eq!(reloaded.get().general.language, "tr");
        assert_eq!(reloaded.get().privacy.telemetry, true);

        let _ = fs::remove_file(&config_path);
    }
}

use std::fs;
use std::path::{Path, PathBuf};
use whatnull_platform::XdgPaths;
use whatnull_types::AppError;

#[derive(Clone)]
pub struct StorageManager {
    base_data_dir: PathBuf,
    base_cache_dir: PathBuf,
}

impl StorageManager {
    pub fn new(paths: &XdgPaths) -> Self {
        Self {
            base_data_dir: paths.data_dir.clone(),
            base_cache_dir: paths.cache_dir.clone(),
        }
    }

    pub fn get_profile_data_dir(&self, profile_id: &str) -> PathBuf {
        self.base_data_dir.join("profiles").join(profile_id)
    }

    pub fn get_profile_cache_dir(&self, profile_id: &str) -> PathBuf {
        self.base_cache_dir.join("profiles").join(profile_id)
    }

    pub fn ensure_dirs(&self, profile_id: &str) -> Result<(), AppError> {
        let data = self.get_profile_data_dir(profile_id);
        let cache = self.get_profile_cache_dir(profile_id);

        if !data.exists() {
            fs::create_dir_all(&data).map_err(|e| {
                AppError::Storage(format!("Failed to create profile data directory: {}", e))
            })?;
        }

        if !cache.exists() {
            fs::create_dir_all(&cache).map_err(|e| {
                AppError::Storage(format!("Failed to create profile cache directory: {}", e))
            })?;
        }

        Ok(())
    }
}

pub trait SecretStore {
    fn get(&self, key: &str) -> Result<Option<String>, AppError>;
    fn set(&self, key: &str, value: &str) -> Result<(), AppError>;
    fn delete(&self, key: &str) -> Result<(), AppError>;
}

pub struct FileSecretStore {
    path: PathBuf,
}

impl FileSecretStore {
    pub fn new(base_dir: &Path) -> Self {
        Self {
            path: base_dir.join("secrets.json"),
        }
    }
}

impl SecretStore for FileSecretStore {
    fn get(&self, key: &str) -> Result<Option<String>, AppError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|e| AppError::Storage(format!("Failed to read secrets file: {}", e)))?;
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Storage(format!("Failed to parse secrets JSON: {}", e)))?;

        Ok(json
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), AppError> {
        let mut json = if self.path.exists() {
            let content = fs::read_to_string(&self.path)
                .map_err(|e| AppError::Storage(format!("Failed to read secrets file: {}", e)))?;
            serde_json::from_str(&content)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()))
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                key.to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }

        let serialized = serde_json::to_string_pretty(&json)
            .map_err(|e| AppError::Storage(format!("Failed to serialize secrets: {}", e)))?;

        fs::write(&self.path, serialized)
            .map_err(|e| AppError::Storage(format!("Failed to write secrets file: {}", e)))?;

        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), AppError> {
        if !self.path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|e| AppError::Storage(format!("Failed to read secrets file: {}", e)))?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Storage(format!("Failed to parse secrets JSON: {}", e)))?;

        if let Some(obj) = json.as_object_mut() {
            obj.remove(key);
        }

        let serialized = serde_json::to_string_pretty(&json)
            .map_err(|e| AppError::Storage(format!("Failed to serialize secrets: {}", e)))?;

        fs::write(&self.path, serialized)
            .map_err(|e| AppError::Storage(format!("Failed to write secrets file: {}", e)))?;

        Ok(())
    }
}

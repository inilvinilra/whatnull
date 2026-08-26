use std::sync::{Arc, RwLock};
use whatnull_config::ConfigManager;
use whatnull_platform::XdgPaths;
use whatnull_storage::StorageManager;
use whatnull_types::{AppError, SessionState};

pub struct AppCore {
    pub paths: XdgPaths,
    pub config_manager: Arc<RwLock<ConfigManager>>,
    pub storage_manager: StorageManager,
    session_state: Arc<RwLock<SessionState>>,
}

impl AppCore {
    pub fn new() -> Result<Self, AppError> {
        let paths = XdgPaths::resolve()?;
        let config_path = paths.config_dir.join("config.json");
        let config_manager = Arc::new(RwLock::new(ConfigManager::load(config_path)?));
        let storage_manager = StorageManager::new(&paths);

        Ok(Self {
            paths,
            config_manager,
            storage_manager,
            session_state: Arc::new(RwLock::new(SessionState::Uninitialized)),
        })
    }

    pub fn get_session_state(&self) -> SessionState {
        *self.session_state.read().unwrap()
    }

    pub fn set_session_state(&self, state: SessionState) {
        *self.session_state.write().unwrap() = state;
    }
}

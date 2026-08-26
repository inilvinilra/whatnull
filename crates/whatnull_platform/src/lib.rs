use std::env;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use whatnull_types::AppError;

pub struct XdgPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub state_dir: PathBuf,
}

impl XdgPaths {
    pub fn resolve() -> Result<Self, AppError> {
        let home = env::var("HOME").map_err(|_| {
            AppError::Platform("HOME environment variable not set".to_string())
        })?;
        let home_path = Path::new(&home);

        let config_dir = env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_path.join(".config"))
            .join("whatnull");

        let data_dir = env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_path.join(".local/share"))
            .join("whatnull");

        let cache_dir = env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_path.join(".cache"))
            .join("whatnull");

        let state_dir = env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_path.join(".local/state"))
            .join("whatnull");

        Ok(Self {
            config_dir,
            data_dir,
            cache_dir,
            state_dir,
        })
    }

    pub fn get_download_dir(&self) -> PathBuf {
        let home = env::var("HOME").unwrap_or_default();
        if home.is_empty() {
            return env::temp_dir();
        }
        let home_path = PathBuf::from(&home);
        let user_dirs_path = home_path.join(".config/user-dirs.dirs");

        if let Ok(content) = fs::read_to_string(user_dirs_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("XDG_DOWNLOAD_DIR=") {
                    let parts: Vec<&str> = trimmed.split('=').collect();
                    if parts.len() == 2 {
                        let path_val = parts[1].trim_matches('"');
                        let replaced = path_val
                            .replace("$HOME", &home)
                            .replace("${HOME}", &home);
                        return PathBuf::from(replaced);
                    }
                }
            }
        }
        home_path.join("Downloads")
    }
}

pub struct AutostartManager {
    autostart_dir: PathBuf,
}

impl AutostartManager {
    pub fn new() -> Result<Self, AppError> {
        let home = env::var("HOME").map_err(|_| {
            AppError::Platform("HOME environment variable not set".to_string())
        })?;
        let autostart_dir = PathBuf::from(home).join(".config/autostart");
        Ok(Self { autostart_dir })
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), AppError> {
        let desktop_file_path = self.autostart_dir.join("whatnull.desktop");

        if !enabled {
            if desktop_file_path.exists() {
                fs::remove_file(desktop_file_path).map_err(|e| {
                    AppError::Platform(format!("Failed to remove autostart entry: {}", e))
                })?;
            }
            return Ok(());
        }

        if !self.autostart_dir.exists() {
            fs::create_dir_all(&self.autostart_dir).map_err(|e| {
                AppError::Platform(format!("Failed to create autostart directory: {}", e))
            })?;
        }

        let current_exe = env::current_exe().map_err(|e| {
            AppError::Platform(format!("Failed to get current executable path: {}", e))
        })?;

        let entry_content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Version=1.0\n\
             Name=WhatNull\n\
             Comment=WhatsApp Desktop Client\n\
             Exec={} --started-at-boot\n\
             Icon=whatnull\n\
             Terminal=false\n\
             Categories=Network;InstantMessaging;\n\
             X-GNOME-Autostart-enabled=true\n",
            current_exe.to_string_lossy()
        );

        fs::write(desktop_file_path, entry_content).map_err(|e| {
            AppError::Platform(format!("Failed to write autostart desktop file: {}", e))
        })?;

        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.autostart_dir.join("whatnull.desktop").exists()
    }
}

pub enum SingleInstanceResult {
    Primary(SingleInstanceGuard),
    Secondary,
}

pub struct SingleInstanceGuard {
    socket_path: PathBuf,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket_path);
    }
}

pub fn check_single_instance<F>(
    profile_id: &str,
    on_focus_signal: F,
) -> Result<SingleInstanceResult, AppError>
where
    F: Fn() + Send + Sync + 'static,
{
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir());

    let socket_path = runtime_dir.join(format!("whatnull-{}.sock", profile_id));
    let on_focus = std::sync::Arc::new(on_focus_signal);

    match UnixListener::bind(&socket_path) {
        Ok(listener) => {
            let guard = SingleInstanceGuard {
                socket_path: socket_path.clone(),
            };
            spawn_uds_listener(listener, on_focus);
            Ok(SingleInstanceResult::Primary(guard))
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            match UnixStream::connect(&socket_path) {
                Ok(mut stream) => {
                    let _ = stream.write_all(b"focus");
                    Ok(SingleInstanceResult::Secondary)
                }
                Err(_) => {
                    let _ = fs::remove_file(&socket_path);
                    match UnixListener::bind(&socket_path) {
                        Ok(listener) => {
                            let guard = SingleInstanceGuard {
                                socket_path: socket_path.clone(),
                            };
                            spawn_uds_listener(listener, on_focus);
                            Ok(SingleInstanceResult::Primary(guard))
                        }
                        Err(err) => Err(AppError::Platform(format!(
                            "Failed to bind to socket after removing stale file: {}",
                            err
                        ))),
                    }
                }
            }
        }
        Err(e) => Err(AppError::Platform(format!("Failed to bind to socket: {}", e))),
    }
}

fn spawn_uds_listener(
    listener: UnixListener,
    on_focus: std::sync::Arc<impl Fn() + Send + Sync + 'static>,
) {
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = [0; 5];
                if let Ok(n) = stream.read(&mut buffer) {
                    if &buffer[..n] == b"focus" {
                        on_focus();
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdg_paths_resolve() {
        let resolved = XdgPaths::resolve();
        assert!(resolved.is_ok());
        let paths = resolved.unwrap();
        assert!(paths.config_dir.to_string_lossy().contains("whatnull"));
        assert!(paths.data_dir.to_string_lossy().contains("whatnull"));
        assert!(paths.cache_dir.to_string_lossy().contains("whatnull"));
        assert!(paths.state_dir.to_string_lossy().contains("whatnull"));
    }
}

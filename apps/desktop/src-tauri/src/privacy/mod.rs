use crate::error::AppErrorWrapper;
use whatnull_types::AppError;

#[tauri::command]
pub fn strip_file_metadata(path: String) -> Result<whatnull_privacy::StripResult, AppErrorWrapper> {
    let file_path = std::path::Path::new(&path);

    if !file_path.exists() {
        return Err(AppErrorWrapper::from(AppError::Io(
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found").to_string(),
        )));
    }

    whatnull_privacy::strip_metadata(file_path).map_err(|e| {
        AppErrorWrapper::from(AppError::Internal(format!("Metadata strip failed: {}", e)))
    })
}

#[tauri::command]
pub fn inspect_file_metadata(
    path: String,
) -> Result<whatnull_privacy::MetadataInfo, AppErrorWrapper> {
    let file_path = std::path::Path::new(&path);

    if !file_path.exists() {
        return Err(AppErrorWrapper::from(AppError::Io(
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found").to_string(),
        )));
    }

    whatnull_privacy::inspect_metadata(file_path).map_err(|e| {
        AppErrorWrapper::from(AppError::Internal(format!(
            "Metadata inspect failed: {}",
            e
        )))
    })
}

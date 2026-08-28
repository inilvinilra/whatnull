use crate::error::AppErrorWrapper;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use whatnull_types::AppError;

const MAX_UPLOAD_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFilePayload {
    name: String,
    mime_type: String,
    data_base64: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SanitizedUploadFile {
    index: usize,
    name: String,
    mime_type: String,
    data_base64: Option<String>,
    changed: bool,
    error: Option<String>,
    fields_removed: Vec<String>,
    original_size: u64,
    stripped_size: u64,
}

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

#[tauri::command]
pub fn sanitize_upload_files(files: Vec<UploadFilePayload>) -> Vec<SanitizedUploadFile> {
    files
        .into_iter()
        .enumerate()
        .map(|(index, file)| {
            let name = file.name.clone();
            let mime_type = file.mime_type.clone();
            sanitize_upload_file(index, file).unwrap_or_else(|error| SanitizedUploadFile {
                index,
                name,
                mime_type,
                data_base64: None,
                changed: false,
                error: Some(error),
                fields_removed: Vec::new(),
                original_size: 0,
                stripped_size: 0,
            })
        })
        .collect()
}

fn sanitize_upload_file(
    index: usize,
    file: UploadFilePayload,
) -> Result<SanitizedUploadFile, String> {
    let decoded = BASE64
        .decode(file.data_base64.as_bytes())
        .map_err(|e| format!("Upload decode failed: {}", e))?;

    if decoded.len() > MAX_UPLOAD_BYTES {
        return Err(format!(
            "Upload is too large to sanitize safely: {} bytes",
            decoded.len()
        ));
    }

    let extension = safe_extension(&file.name, &file.mime_type).ok_or_else(|| {
        format!(
            "Unsupported upload type for metadata stripping: {}",
            file.mime_type
        )
    })?;

    let input_path = temp_upload_path(index, &extension)?;
    fs::write(&input_path, decoded).map_err(|e| format!("Upload staging failed: {}", e))?;

    let result = whatnull_privacy::strip_metadata(&input_path).map_err(|e| {
        let _ = fs::remove_file(&input_path);
        format!("Upload metadata strip failed: {}", e)
    })?;

    let sanitized = fs::read(&input_path).map_err(|e| format!("Upload readback failed: {}", e))?;
    let _ = fs::remove_file(&input_path);

    Ok(SanitizedUploadFile {
        index,
        name: file.name,
        mime_type: file.mime_type,
        data_base64: Some(BASE64.encode(sanitized)),
        changed: result.stripped_size != result.original_size || !result.fields_removed.is_empty(),
        error: None,
        fields_removed: result.fields_removed,
        original_size: result.original_size,
        stripped_size: result.stripped_size,
    })
}

fn safe_extension(name: &str, mime_type: &str) -> Option<String> {
    let from_name = Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .filter(|ext| {
            matches!(
                ext.as_str(),
                "jpg"
                    | "jpeg"
                    | "png"
                    | "pdf"
                    | "mp4"
                    | "mov"
                    | "avi"
                    | "mkv"
                    | "webm"
                    | "mp3"
                    | "wav"
                    | "ogg"
                    | "flac"
            )
        });

    from_name.or_else(|| match mime_type {
        "image/jpeg" => Some("jpg".to_string()),
        "image/png" => Some("png".to_string()),
        "application/pdf" => Some("pdf".to_string()),
        "video/mp4" => Some("mp4".to_string()),
        "video/quicktime" => Some("mov".to_string()),
        "video/x-msvideo" => Some("avi".to_string()),
        "video/x-matroska" => Some("mkv".to_string()),
        "video/webm" => Some("webm".to_string()),
        "audio/mpeg" => Some("mp3".to_string()),
        "audio/wav" => Some("wav".to_string()),
        "audio/ogg" => Some("ogg".to_string()),
        "audio/flac" => Some("flac".to_string()),
        _ => None,
    })
}

fn temp_upload_path(index: usize, extension: &str) -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();

    Ok(std::env::temp_dir().join(format!(
        "whatnull-upload-{}-{}-{}.{}",
        std::process::id(),
        nanos,
        index,
        extension
    )))
}

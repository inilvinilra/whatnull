use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use lopdf::{Document, Object};

const JPEG_APP0_JFIF: u8 = 0xE0;
const JPEG_APP1_EXIF_XMP: u8 = 0xE1;
const JPEG_APP2_ICC: u8 = 0xE2;
const JPEG_APP15: u8 = 0xEF;
const JPEG_COMMENT: u8 = 0xFE;

const EXIF_SEGMENT_PREFIX: &[u8] = b"Exif\x00";
const XMP_SEGMENT_PREFIX: &[u8] = b"http://ns.adobe.com/xap/";
const EXIF_GPS_IFD_TAG: [u8; 2] = [0x88, 0x25];

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image processing error: {0}")]
    ImageProcessing(String),
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetadataInfo {
    pub file_path: String,
    pub file_type: String,
    pub has_exif: bool,
    pub has_gps: bool,
    pub metadata_size_bytes: usize,
    pub fields_found: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StripResult {
    pub file_path: String,
    pub original_size: u64,
    pub stripped_size: u64,
    pub bytes_removed: u64,
    pub fields_removed: Vec<String>,
}

fn detect_file_type(path: &Path) -> Option<String> {
    path.extension().and_then(|ext| ext.to_str()).map(|ext| {
        let lower = ext.to_lowercase();
        match lower.as_str() {
            "jpg" | "jpeg" => "jpeg".to_string(),
            "png" => "png".to_string(),
            "webp" => "webp".to_string(),
            "gif" => "gif".to_string(),
            "pdf" => "pdf".to_string(),
            "mp4" | "mov" | "avi" | "mkv" | "webm" => "video".to_string(),
            "mp3" | "ogg" | "flac" | "wav" | "m4a" => "audio".to_string(),
            _ => lower,
        }
    })
}

pub fn strip_jpeg_metadata(path: &Path) -> Result<StripResult, MetadataError> {
    let original_bytes = fs::read(path)?;
    let original_size = original_bytes.len() as u64;

    let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(original_bytes.into())
        .map_err(|e| MetadataError::ImageProcessing(format!("Failed to parse JPEG: {}", e)))?;

    let mut fields_removed = Vec::new();

    for segment in jpeg.segments_mut() {
        let marker = segment.marker();
        match marker {
            JPEG_APP1_EXIF_XMP => {
                let bytes = segment.contents();
                if bytes.starts_with(EXIF_SEGMENT_PREFIX) {
                    fields_removed.push("EXIF (GPS, camera, timestamps)".to_string());
                } else if bytes.starts_with(XMP_SEGMENT_PREFIX) {
                    fields_removed.push("XMP (Adobe metadata)".to_string());
                } else {
                    fields_removed.push("APP1 segment".to_string());
                }
                *segment = img_parts::jpeg::JpegSegment::new_with_contents(
                    marker,
                    img_parts::Bytes::new(),
                );
            }
            JPEG_APP2_ICC..=JPEG_APP15 => {
                if marker == JPEG_APP2_ICC {
                    fields_removed.push("ICC Color Profile".to_string());
                } else {
                    fields_removed.push(format!("APP{} segment", marker - JPEG_APP0_JFIF));
                }
                *segment = img_parts::jpeg::JpegSegment::new_with_contents(
                    marker,
                    img_parts::Bytes::new(),
                );
            }
            JPEG_COMMENT => {
                fields_removed.push("JPEG Comment".to_string());
                *segment = img_parts::jpeg::JpegSegment::new_with_contents(
                    marker,
                    img_parts::Bytes::new(),
                );
            }
            _ => {}
        }
    }

    jpeg.segments_mut()
        .retain(|seg| !seg.contents().is_empty() || seg.marker() < JPEG_APP1_EXIF_XMP);

    let mut output = Vec::new();
    jpeg.encoder()
        .write_to(&mut Cursor::new(&mut output))
        .map_err(|e| MetadataError::ImageProcessing(format!("Failed to write JPEG: {}", e)))?;

    let stripped_size = output.len() as u64;
    fs::write(path, &output)?;

    Ok(StripResult {
        file_path: path.to_string_lossy().to_string(),
        original_size,
        stripped_size,
        bytes_removed: original_size.saturating_sub(stripped_size),
        fields_removed,
    })
}

pub fn strip_png_metadata(path: &Path) -> Result<StripResult, MetadataError> {
    let original_bytes = fs::read(path)?;
    let original_size = original_bytes.len() as u64;

    let mut png = img_parts::png::Png::from_bytes(original_bytes.into())
        .map_err(|e| MetadataError::ImageProcessing(format!("Failed to parse PNG: {}", e)))?;

    let mut fields_removed = Vec::new();

    let metadata_chunk_types: &[&[u8; 4]] = &[b"tEXt", b"iTXt", b"zTXt", b"eXIf", b"iCCP", b"tIME"];

    for chunk in png.chunks() {
        let kind_bytes = chunk.kind();
        for mt in metadata_chunk_types {
            if kind_bytes == **mt {
                fields_removed.push(format!(
                    "PNG {} chunk",
                    std::str::from_utf8(*mt).unwrap_or("????")
                ));
            }
        }
    }

    png.chunks_mut().retain(|chunk| {
        let kind = chunk.kind();
        !metadata_chunk_types.iter().any(|mt| kind == **mt)
    });

    let mut output = Vec::new();
    png.encoder()
        .write_to(&mut Cursor::new(&mut output))
        .map_err(|e| MetadataError::ImageProcessing(format!("Failed to write PNG: {}", e)))?;

    let stripped_size = output.len() as u64;
    fs::write(path, &output)?;

    Ok(StripResult {
        file_path: path.to_string_lossy().to_string(),
        original_size,
        stripped_size,
        bytes_removed: original_size.saturating_sub(stripped_size),
        fields_removed,
    })
}

pub fn strip_metadata(path: &Path) -> Result<StripResult, MetadataError> {
    let file_type = detect_file_type(path)
        .ok_or_else(|| MetadataError::UnsupportedFileType("Unknown file extension".to_string()))?;

    match file_type.as_str() {
        "jpeg" => strip_jpeg_metadata(path),
        "png" => strip_png_metadata(path),
        "pdf" => strip_pdf_metadata(path),
        "video" | "audio" => strip_av_metadata(path),
        other => Err(MetadataError::UnsupportedFileType(format!(
            "No metadata stripper for type: {}",
            other
        ))),
    }
}

fn sibling_output_path(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("whatnull");
    let name = match path.extension().and_then(|value| value.to_str()) {
        Some(extension) => format!("{}.whatnull-stripped.{}", stem, extension),
        None => format!("{}.whatnull-stripped", stem),
    };
    path.with_file_name(name)
}

pub fn strip_pdf_metadata(path: &Path) -> Result<StripResult, MetadataError> {
    let original_size = fs::metadata(path)?.len();
    let mut doc = Document::load(path)
        .map_err(|e| MetadataError::ImageProcessing(format!("Failed to parse PDF: {}", e)))?;
    let mut fields_removed = Vec::new();

    if doc.trailer.remove(b"Info").is_some() {
        fields_removed.push("PDF trailer Info dictionary".to_string());
    }

    let mut metadata_refs = Vec::new();
    let metadata_keys = [
        b"Author".as_slice(),
        b"Creator".as_slice(),
        b"Producer".as_slice(),
        b"CreationDate".as_slice(),
        b"ModDate".as_slice(),
        b"Title".as_slice(),
        b"Subject".as_slice(),
        b"Keywords".as_slice(),
    ];

    for object in doc.objects.values_mut() {
        match object {
            Object::Dictionary(dict) => {
                if let Some(metadata) = dict.remove(b"Metadata") {
                    if let Object::Reference(id) = metadata {
                        metadata_refs.push(id);
                    }
                    fields_removed.push("PDF XMP Metadata reference".to_string());
                }

                for key in metadata_keys {
                    if dict.remove(key).is_some() {
                        fields_removed.push(format!("PDF {} field", String::from_utf8_lossy(key)));
                    }
                }
            }
            Object::Stream(stream) => {
                if stream.dict.remove(b"Metadata").is_some() {
                    fields_removed.push("PDF stream Metadata reference".to_string());
                }

                let is_metadata_stream = stream
                    .dict
                    .get(b"Type")
                    .ok()
                    .and_then(|value| value.as_name().ok())
                    == Some(b"Metadata");

                if is_metadata_stream {
                    stream.content.clear();
                    stream.dict.set("Length", 0);
                    fields_removed.push("PDF XMP Metadata stream".to_string());
                }

                for key in metadata_keys {
                    if stream.dict.remove(key).is_some() {
                        fields_removed.push(format!("PDF {} field", String::from_utf8_lossy(key)));
                    }
                }
            }
            _ => {}
        }
    }

    for id in metadata_refs {
        if doc.objects.remove(&id).is_some() {
            fields_removed.push("PDF referenced Metadata object".to_string());
        }
    }

    let output_path = sibling_output_path(path);
    doc.compress();
    doc.save(&output_path)
        .map_err(|e| MetadataError::ImageProcessing(format!("Failed to write PDF: {}", e)))?;
    let stripped_size = fs::metadata(&output_path)?.len();
    fs::rename(&output_path, path)?;

    Ok(StripResult {
        file_path: path.to_string_lossy().to_string(),
        original_size,
        stripped_size,
        bytes_removed: original_size.saturating_sub(stripped_size),
        fields_removed,
    })
}

pub fn strip_av_metadata(path: &Path) -> Result<StripResult, MetadataError> {
    let original_size = fs::metadata(path)?.len();
    let output_path = sibling_output_path(path);

    let status = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            &path.to_string_lossy(),
            "-map_metadata",
            "-1",
            "-c",
            "copy",
            "-y",
            &output_path.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => {
            let stripped_size = fs::metadata(&output_path)?.len();
            fs::rename(&output_path, path)?;

            Ok(StripResult {
                file_path: path.to_string_lossy().to_string(),
                original_size,
                stripped_size,
                bytes_removed: original_size.saturating_sub(stripped_size),
                fields_removed: vec![
                    "Video/Audio global metadata".to_string(),
                    "Container metadata tags".to_string(),
                    "Chapter metadata".to_string(),
                ],
            })
        }
        Ok(_) => {
            let _ = fs::remove_file(&output_path);
            Err(MetadataError::ImageProcessing(
                "ffmpeg exited with non-zero status".to_string(),
            ))
        }
        Err(_) => Err(MetadataError::ImageProcessing(
            "ffmpeg not found. Install ffmpeg for video/audio metadata stripping.".to_string(),
        )),
    }
}

pub fn inspect_metadata(path: &Path) -> Result<MetadataInfo, MetadataError> {
    let file_type = detect_file_type(path)
        .ok_or_else(|| MetadataError::UnsupportedFileType("Unknown extension".to_string()))?;

    let bytes = fs::read(path)?;
    let mut has_exif = false;
    let mut has_gps = false;
    let mut metadata_size: usize = 0;
    let mut fields_found = Vec::new();

    match file_type.as_str() {
        "jpeg" => {
            if let Ok(jpeg) = img_parts::jpeg::Jpeg::from_bytes(bytes.into()) {
                for segment in jpeg.segments() {
                    match segment.marker() {
                        JPEG_APP1_EXIF_XMP => {
                            let contents = segment.contents();
                            metadata_size += contents.len();
                            if contents.starts_with(EXIF_SEGMENT_PREFIX) {
                                has_exif = true;
                                fields_found.push("EXIF data".to_string());
                                if contents.windows(2).any(|w| w == EXIF_GPS_IFD_TAG) {
                                    has_gps = true;
                                    fields_found.push("GPS coordinates".to_string());
                                }
                            }
                            if contents.starts_with(XMP_SEGMENT_PREFIX) {
                                fields_found.push("XMP metadata".to_string());
                            }
                        }
                        JPEG_APP2_ICC..=JPEG_APP15 => {
                            metadata_size += segment.contents().len();
                            fields_found
                                .push(format!("APP{} data", segment.marker() - JPEG_APP0_JFIF));
                        }
                        JPEG_COMMENT => {
                            metadata_size += segment.contents().len();
                            fields_found.push("Comment".to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
        "png" => {
            if let Ok(png) = img_parts::png::Png::from_bytes(bytes.into()) {
                for chunk in png.chunks() {
                    let kind = chunk.kind();
                    if kind == *b"eXIf" {
                        has_exif = true;
                        metadata_size += chunk.contents().len();
                        fields_found.push("EXIF data".to_string());
                    } else if kind == *b"tEXt" || kind == *b"iTXt" || kind == *b"zTXt" {
                        metadata_size += chunk.contents().len();
                        fields_found.push(format!(
                            "{} text chunk",
                            std::str::from_utf8(&kind).unwrap_or("????")
                        ));
                    } else if kind == *b"tIME" {
                        metadata_size += chunk.contents().len();
                        fields_found.push("Modification time".to_string());
                    }
                }
            }
        }
        "pdf" => {
            let content = String::from_utf8_lossy(&bytes);
            let metadata_keys = [
                "/Author",
                "/Creator",
                "/Producer",
                "/CreationDate",
                "/ModDate",
                "/Title",
                "/Subject",
                "/Keywords",
                "/Metadata",
            ];

            for key in metadata_keys {
                if content.contains(key) {
                    metadata_size += key.len();
                    fields_found.push(format!("PDF {}", key));
                }
            }
        }
        _ => {
            fields_found.push("Metadata inspection not available for this type".to_string());
        }
    }

    Ok(MetadataInfo {
        file_path: path.to_string_lossy().to_string(),
        file_type,
        has_exif,
        has_gps,
        metadata_size_bytes: metadata_size,
        fields_found,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(
            detect_file_type(Path::new("photo.jpg")).as_deref(),
            Some("jpeg")
        );
        assert_eq!(
            detect_file_type(Path::new("photo.JPEG")).as_deref(),
            Some("jpeg")
        );
        assert_eq!(
            detect_file_type(Path::new("image.png")).as_deref(),
            Some("png")
        );
        assert_eq!(
            detect_file_type(Path::new("video.mp4")).as_deref(),
            Some("video")
        );
        assert_eq!(
            detect_file_type(Path::new("doc.pdf")).as_deref(),
            Some("pdf")
        );
        assert_eq!(detect_file_type(Path::new("noext")), None);
    }

    fn minimal_jpeg_with_exif() -> Vec<u8> {
        let soi: &[u8] = &[0xFF, 0xD8];
        let app0_jfif: &[u8] = &[
            0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x00,
        ];
        let app1_exif: &[u8] = &[0xFF, 0xE1, 0x00, 0x08, b'E', b'x', b'i', b'f', 0x00, 0x00];
        let sof0: &[u8] = &[
            0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00,
        ];
        let sos: &[u8] = &[
            0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, 0x00,
        ];
        let eoi: &[u8] = &[0xFF, 0xD9];

        let mut bytes = Vec::new();
        for part in [soi, app0_jfif, app1_exif, sof0, sos, eoi] {
            bytes.extend_from_slice(part);
        }
        bytes
    }

    fn has_marker(bytes: &[u8], marker: u8) -> bool {
        img_parts::jpeg::Jpeg::from_bytes(bytes.to_vec().into())
            .unwrap()
            .segments()
            .iter()
            .any(|segment| segment.marker() == marker)
    }

    #[test]
    fn strip_jpeg_metadata_removes_the_exif_segment_and_keeps_jfif() {
        let source = minimal_jpeg_with_exif();
        assert!(has_marker(&source, JPEG_APP1_EXIF_XMP));

        let dir = std::env::temp_dir().join("whatnull_test_strip_jpeg");
        let _ = fs::create_dir_all(&dir);
        let file = dir.join("exif.jpg");
        fs::write(&file, &source).unwrap();

        let result = strip_jpeg_metadata(&file).unwrap();
        let stripped = fs::read(&file).unwrap();

        let _ = fs::remove_file(&file);
        let _ = fs::remove_dir(&dir);

        assert!(!has_marker(&stripped, JPEG_APP1_EXIF_XMP));
        assert!(has_marker(&stripped, JPEG_APP0_JFIF));
        assert!(result
            .fields_removed
            .iter()
            .any(|field| field.contains("EXIF")));
    }

    #[test]
    fn test_unsupported_file_type() {
        let result = strip_metadata(Path::new("/tmp/test.xyz"));
        assert!(result.is_err());
        match result {
            Err(MetadataError::UnsupportedFileType(_)) => {}
            _ => panic!("Expected UnsupportedFileType error"),
        }
    }

    fn ffmpeg_available() -> bool {
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[test]
    fn sibling_output_path_keeps_the_container_extension() {
        assert_eq!(
            sibling_output_path(Path::new("/tmp/clip.mp4")),
            PathBuf::from("/tmp/clip.whatnull-stripped.mp4")
        );
        assert_eq!(
            sibling_output_path(Path::new("/tmp/song.MP3")),
            PathBuf::from("/tmp/song.whatnull-stripped.MP3")
        );
        assert_eq!(
            sibling_output_path(Path::new("/tmp/noext")),
            PathBuf::from("/tmp/noext.whatnull-stripped")
        );
    }

    #[test]
    fn strip_av_metadata_removes_container_tags() {
        if !ffmpeg_available() {
            return;
        }

        let dir = std::env::temp_dir().join("whatnull_test_av");
        let _ = fs::create_dir_all(&dir);
        let file = dir.join("tagged.mp3");
        let _ = fs::remove_file(&file);

        let marker = "WhatNullSecretTitle";
        let created = std::process::Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1",
                "-metadata",
                &format!("title={}", marker),
                "-metadata",
                "artist=WhatNullSecretArtist",
                "-y",
                &file.to_string_lossy(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);

        if !created {
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        let before = fs::read(&file).unwrap();
        assert!(
            String::from_utf8_lossy(&before).contains(marker),
            "fixture should carry the tag"
        );

        let result = strip_av_metadata(&file).unwrap();
        let after = fs::read(&file).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert!(!String::from_utf8_lossy(&after).contains(marker));
        assert!(!result.fields_removed.is_empty());
        assert!(!after.is_empty());
    }
}

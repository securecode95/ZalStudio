use chrono::NaiveDateTime;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Photo {
    pub path: PathBuf,
    pub file_name: String,
    pub dimensions: Option<(u32, u32)>,
    pub file_size: u64,
    pub date_taken: Option<NaiveDateTime>,
}

pub fn discover_photos(dir: &Path) -> Vec<Photo> {
    if !dir.exists() {
        return vec![];
    }

    let mut photos = Vec::new();
    let exts = [
        "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif", "heic", "heif",
    ];

    for entry in WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if exts.contains(&ext.as_str()) {
                    let file_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    // Skip temp edit files and temp rename files
                    if file_name.starts_with("edit_") || file_name.ends_with(".tmp") {
                        continue;
                    }
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let date_taken = extract_date_taken(path);
                    photos.push(Photo {
                        path: path.to_path_buf(),
                        file_name,
                        dimensions: None,
                        file_size,
                        date_taken,
                    });
                }
            }
        }
    }

    // Default sort: newest first (by date, fallback to filename)
    photos.sort_by(|a, b| match (b.date_taken, a.date_taken) {
        (Some(d1), Some(d2)) => d1.cmp(&d2),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b.file_name.cmp(&a.file_name),
    });
    photos
}

fn extract_date_taken(path: &Path) -> Option<NaiveDateTime> {
    // Try EXIF DateTimeOriginal first
    if let Ok(file) = std::fs::File::open(path) {
        let mut bufreader = std::io::BufReader::new(&file);
        let exifreader = exif::Reader::new();
        if let Ok(exif) = exifreader.read_from_container(&mut bufreader) {
            if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
                let date_str = field.display_value().to_string();
                let date_str = date_str.trim_matches('"');
                if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S") {
                    return Some(dt);
                }
            }
        }
    }
    // Fallback to file modification time
    if let Ok(metadata) = std::fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                return NaiveDateTime::from_timestamp_opt(duration.as_secs() as i64, 0);
            }
        }
    }
    None
}

pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

pub fn format_dimensions(dim: Option<(u32, u32)>) -> String {
    match dim {
        Some((w, h)) => format!("{}x{}", w, h),
        None => "Unknown".to_string(),
    }
}

pub fn format_date(date: Option<NaiveDateTime>) -> String {
    match date {
        Some(d) => d.format("%Y-%m-%d %H:%M").to_string(),
        None => "—".to_string(),
    }
}

pub fn format_date_short(date: Option<NaiveDateTime>) -> String {
    match date {
        Some(d) => d.format("%Y-%m-%d").to_string(),
        None => "—".to_string(),
    }
}

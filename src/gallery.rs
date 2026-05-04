use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Photo {
    pub path: PathBuf,
    pub file_name: String,
    pub dimensions: Option<(u32, u32)>,
    pub file_size: u64,
}

pub fn discover_photos(dir: &Path) -> Vec<Photo> {
    if !dir.exists() {
        return vec![];
    }

    let mut photos = Vec::new();
    let exts = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif", "heic", "heif"];

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
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let dimensions = image::image_dimensions(path).ok();
                    photos.push(Photo {
                        path: path.to_path_buf(),
                        file_name,
                        dimensions,
                        file_size,
                    });
                }
            }
        }
    }

    photos.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    photos
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

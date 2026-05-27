use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IMAGE_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif", "heic", "heif",
];

#[derive(Debug, Clone)]
pub struct PhonePhoto {
    pub path: PathBuf,
    pub file_name: String,
    pub file_size: u64,
}

/// Try multiple methods to find connected phones
pub fn find_phone_mounts() -> Vec<PathBuf> {
    let mut mounts = Vec::new();

    // 1. GVFS MTP / PTP / AFC mounts (most common on modern Linux desktops)
    // Check all /run/user/*/gvfs directories dynamically
    if let Ok(entries) = std::fs::read_dir("/run/user") {
        for entry in entries.flatten() {
            let gvfs = entry.path().join("gvfs");
            if let Ok(sub) = std::fs::read_dir(&gvfs) {
                for subentry in sub.flatten() {
                    let name = subentry.file_name().to_string_lossy().to_string();
                    // mtp:host = Android MTP
                    // ptp:host = PTP mode (some Samsung, cameras)
                    // gphoto2:host = older cameras / some Android
                    // afc:host = iPhone via GVFS-afc
                    if name.starts_with("mtp:host")
                        || name.starts_with("ptp:host")
                        || name.starts_with("gphoto2:host")
                        || name.starts_with("afc:host")
                    {
                        mounts.push(subentry.path());
                    }
                }
            }
        }
    }

    // 2. Fallback: hardcoded common UIDs if /run/user scan fails
    for base in ["/run/user/1000/gvfs", "/run/user/1001/gvfs"] {
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("mtp:host")
                    || name.starts_with("ptp:host")
                    || name.starts_with("gphoto2:host")
                    || name.starts_with("afc:host")
                {
                    mounts.push(entry.path());
                }
            }
        }
    }

    // 3. FUSE-based MTP mount points (jmtpfs, simple-mtpfs)
    for fuse_base in ["/mnt/mtp", "/media/mtp", "/tmp/mtp"] {
        let p = PathBuf::from(fuse_base);
        if p.exists() && has_photo_folders(&p) {
            mounts.push(p);
        }
    }

    // 4. Mass storage mounts under /run/media
    if let Ok(entries) = std::fs::read_dir("/run/media") {
        for entry in entries.flatten() {
            if let Ok(sub) = std::fs::read_dir(&entry.path()) {
                for subentry in sub.flatten() {
                    let path = subentry.path();
                    if has_photo_folders(&path) {
                        mounts.push(path);
                    }
                }
            }
        }
    }

    // 5. Fallback /media
    if let Ok(entries) = std::fs::read_dir("/media") {
        for entry in entries.flatten() {
            if let Ok(sub) = std::fs::read_dir(&entry.path()) {
                for subentry in sub.flatten() {
                    let path = subentry.path();
                    if has_photo_folders(&path) {
                        mounts.push(path);
                    }
                }
            }
        }
    }

    // 6. Check if gvfs is available but empty — maybe phone needs unlock
    // We still return empty to signal "try again after unlocking phone"

    mounts
}

/// Check if a path contains known photo folders
fn has_photo_folders(path: &Path) -> bool {
    ["DCIM", "Pictures", "Camera", "Download", "Photos"]
        .iter()
        .any(|name| path.join(name).exists())
}

/// Check if GVFS MTP daemon is installed
pub fn is_gvfs_mtp_available() -> bool {
    std::path::Path::new("/usr/lib/gvfs/gvfsd-mtp").exists()
        || std::process::Command::new("which")
            .arg("gvfs-mount")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        || std::process::Command::new("which")
            .arg("gio")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

/// Scan a phone mount for photos
pub fn list_phone_photos(mount: &Path) -> Vec<PhonePhoto> {
    let mut photos = Vec::new();

    let search_paths = [
        mount.join("DCIM"),
        mount.join("Pictures"),
        mount.join("Camera"),
        mount.join("Photos"),
        mount.join("Download"),
        mount.join("Internal shared storage/DCIM"),
        mount.join("Phone/DCIM"),
        mount.join("Card/DCIM"),
        mount.join("storage/self/primary/DCIM"),
        mount.join("storage/self/primary/Pictures"),
        mount.join("mnt/media/DCIM"),
    ];

    for search_path in search_paths {
        if !search_path.exists() {
            continue;
        }
        for entry in WalkDir::new(&search_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if IMAGE_EXTS.contains(&ext.as_str()) {
                        let file_name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        photos.push(PhonePhoto {
                            path: path.to_path_buf(),
                            file_name,
                            file_size,
                        });
                    }
                }
            }
        }
    }

    // Sort by filename descending (newest first, typical phone naming)
    photos.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    photos
}

/// Import phone photos to temp dir, converting HEIF/HEIC to JPEG
pub fn import_phone_photos(
    photos: &[&PhonePhoto],
    dest_dir: &Path,
) -> Result<(usize, usize), String> {
    std::fs::create_dir_all(dest_dir).map_err(|e| format!("Kunde inte skapa temp-mapp: {}", e))?;

    let mut count = 0;
    let mut converted = 0;

    for photo in photos {
        let ext = photo
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_heif = ext == "heic" || ext == "heif";

        let dest = if is_heif {
            // Convert to JPEG
            let file_stem = photo.path.file_stem().unwrap_or_default().to_string_lossy();
            let jpeg_name = format!("{}_{}.jpg", file_stem, count);
            let jpeg_path = dest_dir.join(&jpeg_name);

            match convert_heif_to_jpeg(&photo.path, &jpeg_path) {
                Ok(_) => {
                    converted += 1;
                    jpeg_path
                }
                Err(e) => {
                    eprintln!("HEIF conversion failed for {}: {}", photo.file_name, e);
                    // Fallback: copy as-is
                    dest_dir.join(&photo.file_name)
                }
            }
        } else {
            dest_dir.join(&photo.file_name)
        };

        // Handle name collisions
        let dest = if dest.exists() && !is_heif {
            let stem = photo.path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = photo.path.extension().unwrap_or_default().to_string_lossy();
            dest_dir.join(format!("{}_{}.{}", stem, count, ext))
        } else {
            dest
        };

        if is_heif && dest.exists() {
            // Already converted above
            count += 1;
        } else if std::fs::copy(&photo.path, dest).is_ok() {
            count += 1;
        }
    }

    Ok((count, converted))
}

/// Convert HEIF/HEIC to JPEG using ImageMagick
fn convert_heif_to_jpeg(input: &Path, output: &Path) -> Result<(), String> {
    // Try ImageMagick 7 (magick)
    if let Ok(result) = std::process::Command::new("magick")
        .arg(input)
        .arg(output)
        .output()
    {
        if result.status.success() {
            return Ok(());
        }
    }

    // Try ImageMagick 6 (convert)
    if let Ok(result) = std::process::Command::new("convert")
        .arg(input)
        .arg(output)
        .output()
    {
        if result.status.success() {
            return Ok(());
        }
    }

    // Try heif-convert (libheif-examples)
    if let Ok(result) = std::process::Command::new("heif-convert")
        .arg(input)
        .arg(output)
        .output()
    {
        if result.status.success() {
            return Ok(());
        }
    }

    Err("Ingen HEIF-konverterare hittades. Installera ImageMagick (magick/convert) eller heif-convert.".to_string())
}

/// Clear temp import directory
pub fn clear_temp_imports(temp_dir: &Path) -> Result<(), String> {
    if temp_dir.exists() {
        for entry in std::fs::read_dir(temp_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                let _ = std::fs::remove_file(&path);
            } else if path.is_dir() {
                let _ = std::fs::remove_dir_all(&path);
            }
        }
    }
    Ok(())
}

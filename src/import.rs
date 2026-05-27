use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif"];

/// Hitta alla monterade lagringsenheter
pub fn find_mounted_devices() -> Vec<PathBuf> {
    let mut devices = Vec::new();

    // Vanliga monteringspunkter för USB-minnen / kortläsare
    let candidates = ["/run/media", "/media", "/mnt", "/run/user"];

    for base in &candidates {
        let base_path = Path::new(base);
        if !base_path.exists() {
            continue;
        }

        // /run/media/<användare>/<enhet>
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    // Om det är en användarmapp (t.ex. /run/media/johndoe), leta djupare
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub in sub_entries.filter_map(|e| e.ok()) {
                            let sub_path = sub.path();
                            if sub_path.is_dir() {
                                devices.push(sub_path);
                            }
                        }
                    }
                }
            }
        }
    }

    devices
}

/// Importera bilder från USB / kameraminne / kortläsare
/// Försöker först hitta DCIM-mappar (kamerastruktur), annars alla bilder
pub fn import_from_storage(
    source_dir: &Path,
    dest_dir: &Path,
    progress: Option<&dyn Fn(usize, usize, &str)>,
) -> Result<usize, String> {
    // Om source_dir inte finns, försök hitta monterade enheter
    if !source_dir.exists() || source_dir == Path::new("/media") {
        let devices = find_mounted_devices();
        if devices.is_empty() {
            return Err("Inga USB-enheter eller minneskort hittades.\nSätt i ett USB-minne eller minneskort och försök igen.".to_string());
        }

        let mut total = 0;
        let mut found_any = false;
        for device in devices {
            match import_from_storage_single(&device, dest_dir, progress) {
                Ok(count) => {
                    total += count;
                    found_any = true;
                }
                Err(_) => continue,
            }
        }

        if !found_any {
            return Err("Inga bilder hittades på någon ansluten enhet.".to_string());
        }
        return Ok(total);
    }

    import_from_storage_single(source_dir, dest_dir, progress)
}

fn import_from_storage_single(
    source_dir: &Path,
    dest_dir: &Path,
    progress: Option<&dyn Fn(usize, usize, &str)>,
) -> Result<usize, String> {
    if !source_dir.exists() {
        return Err(format!("Sökvägen hittades inte: {}", source_dir.display()));
    }

    // Leta efter DCIM-mappar (kamerastruktur)
    let dcim_paths: Vec<PathBuf> = WalkDir::new(source_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_dir() && e.file_name().to_string_lossy().to_uppercase() == "DCIM"
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if dcim_paths.is_empty() {
        // Fallback: skanna hela enheten efter bilder
        copy_images(source_dir, dest_dir, false, progress)
    } else {
        let mut total = 0;
        for dcim in dcim_paths {
            total += copy_images(&dcim, dest_dir, false, progress)?;
        }
        Ok(total)
    }
}

fn copy_images(
    source: &Path,
    dest: &Path,
    overwrite: bool,
    progress: Option<&dyn Fn(usize, usize, &str)>,
) -> Result<usize, String> {
    let _ = fs::create_dir_all(dest);

    // First pass: collect all image files to get a total count
    let files: Vec<std::path::PathBuf> = WalkDir::new(source)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if !path.is_file() {
                return false;
            }
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| IMAGE_EXTS.contains(&e.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = files.len();
    let mut copied = 0;
    for path in files {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let file_name = path.file_name().unwrap_or_default();
        let dest_path = dest.join(file_name);
        let src_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        if !dest_path.exists() || overwrite {
            if fs::copy(&path, &dest_path).is_ok() {
                copied += 1;
            }
        } else {
            // Skip if already exists with same size
            let dest_size = fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(0);
            if src_size == dest_size {
                if let Some(ref cb) = progress {
                    let fname = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    cb(copied, total, &fname);
                }
                continue;
            }
            // Byt namn om konflikt
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let new_name = format!("{}_{}.{}", stem, copied, ext);
            let dest_path = dest.join(&new_name);
            if fs::copy(&path, &dest_path).is_ok() {
                copied += 1;
            }
        }

        if let Some(ref cb) = progress {
            let fname = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            cb(copied, total, &fname);
        }
    }

    Ok(copied)
}

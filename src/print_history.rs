use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrintRecord {
    pub timestamp: String,
    pub photo_name: String,
    pub paper_size: String,
    pub copies: u32,
    pub printer: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrintHistoryEntry {
    pub folder_name: String,
    pub folder_path: PathBuf,
    pub record: PrintRecord,
    pub photo_path: PathBuf,
}

const MAX_HISTORY: usize = 30;

pub fn history_dir(base: &Path) -> PathBuf {
    base.join("print_history")
}

pub fn save_print(
    base_dir: &Path,
    photo_path: &Path,
    photo_name: &str,
    paper_size: &str,
    copies: u32,
    printer: &str,
    status: &str,
    error: Option<&str>,
) -> Result<PathBuf, String> {
    let hist_dir = history_dir(base_dir);
    let _ = fs::create_dir_all(&hist_dir);

    let folder_name = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let folder = hist_dir.join(&folder_name);
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;

    // Copy the photo into the history folder
    let ext = photo_path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let hist_photo = folder.join(format!("print.{}", ext));
    fs::copy(photo_path, &hist_photo).map_err(|e| e.to_string())?;

    let record = PrintRecord {
        timestamp: chrono::Local::now().to_rfc3339(),
        photo_name: photo_name.to_string(),
        paper_size: paper_size.to_string(),
        copies,
        printer: printer.to_string(),
        status: status.to_string(),
        error: error.map(|s| s.to_string()),
    };

    let meta_path = folder.join("record.json");
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(&meta_path, json).map_err(|e| e.to_string())?;

    // Keep only the latest MAX_HISTORY entries
    cleanup_old(&hist_dir);

    Ok(folder)
}

pub fn update_print_status(folder: &Path, status: &str, error: Option<&str>) -> Result<(), String> {
    let meta_path = folder.join("record.json");
    if !meta_path.exists() {
        return Ok(());
    }
    let contents = fs::read_to_string(&meta_path).map_err(|e| e.to_string())?;
    let mut record: PrintRecord = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
    record.status = status.to_string();
    record.error = error.map(|s| s.to_string());
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(&meta_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_history(base_dir: &Path) -> Vec<PrintHistoryEntry> {
    let hist_dir = history_dir(base_dir);
    let mut entries = Vec::new();

    let folders = match fs::read_dir(&hist_dir) {
        Ok(iter) => iter,
        Err(_) => return entries,
    };

    for entry in folders.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let folder_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let record_path = path.join("record.json");
        if !record_path.exists() {
            continue;
        }
        let record: PrintRecord = match fs::read_to_string(&record_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            Some(r) => r,
            None => continue,
        };

        // Find photo file in folder
        let photo_path = fs::read_dir(&path)
            .ok()
            .and_then(|mut iter| {
                iter.find(|e| {
                    e.as_ref()
                        .ok()
                        .map(|f| f.file_name().to_string_lossy().starts_with("print."))
                        .unwrap_or(false)
                })
                .and_then(|e| e.ok().map(|f| f.path()))
            })
            .unwrap_or_else(|| path.join("print.jpg"));

        entries.push(PrintHistoryEntry {
            folder_name,
            folder_path: path,
            record,
            photo_path,
        });
    }

    // Sort newest first
    entries.sort_by(|a, b| b.folder_name.cmp(&a.folder_name));
    entries
}

fn cleanup_old(hist_dir: &Path) {
    let mut folders: Vec<_> = match fs::read_dir(hist_dir) {
        Ok(iter) => iter.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).collect(),
        Err(_) => return,
    };
    folders.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    for entry in folders.iter().skip(MAX_HISTORY) {
        let _ = fs::remove_dir_all(entry.path());
    }
}

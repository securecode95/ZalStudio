use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub photo_directory: PathBuf,
    pub paper_sizes: Vec<String>,
    pub default_paper_size: usize,
    pub copies_default: u32,
    pub fit_to_page: bool,
    /// Which CUPS printer to use for each paper size
    pub printer_for_size: HashMap<String, String>,
    /// Path to scan for USB / camera memory imports
    pub usb_import_path: PathBuf,
    /// Google OAuth client ID
    pub google_client_id: String,
    /// Google OAuth client secret
    pub google_client_secret: String,
    /// Temp directory for customer imports (cleared after order)
    pub temp_directory: PathBuf,
    /// WiFi network name for customers to connect to (shown on QR screen)
    pub wifi_ssid: String,
    /// WiFi password for customers (shown on QR screen, can be empty for open networks)
    pub wifi_password: String,
    /// Server port for mobile upload
    pub server_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        let mut printer_for_size = HashMap::new();
        printer_for_size.insert("10x15".to_string(), "CP-9550DW-S".to_string());
        printer_for_size.insert("15x23".to_string(), "CP-9810DW-S".to_string());

        Self {
            photo_directory: PathBuf::from("./photos"),
            paper_sizes: vec!["10x15".to_string(), "15x23".to_string()],
            default_paper_size: 0,
            copies_default: 1,
            fit_to_page: true,
            printer_for_size,
            usb_import_path: PathBuf::from("/media"),
            google_client_id: String::new(),
            google_client_secret: String::new(),
            temp_directory: PathBuf::from("./temp_imports"),
            wifi_ssid: String::from("ZalStudio"),
            wifi_password: String::from(""),
            server_port: 8080,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        // Search paths in priority order
        let candidates = [
            // 1: Same folder as the running EXE
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("config.toml"))),
            // 2: Working directory (project root during `cargo run`)
            Some(PathBuf::from("config.toml")),
            // 3: OS config directory
            dirs::config_dir().map(|p| p.join("zalstudio").join("config.toml")),
        ];

        for candidate in candidates.iter().flatten() {
            eprintln!("[ZalStudio] Checking config: {}", candidate.display());
            if candidate.exists() {
                eprintln!("[ZalStudio] Found config file!");
                match std::fs::read_to_string(candidate) {
                    Ok(contents) => match toml::from_str::<Config>(&contents) {
                        Ok(config) => {
                            eprintln!("[ZalStudio] Loaded config from: {}", candidate.display());
                            eprintln!("[ZalStudio] google_client_id len = {}", config.google_client_id.len());
                            eprintln!("[ZalStudio] google_client_secret len = {}", config.google_client_secret.len());
                            return config;
                        }
                        Err(e) => {
                            eprintln!("[ZalStudio] PARSE ERROR in config.toml: {}", e);
                            eprintln!("[ZalStudio] Falling back to defaults...");
                            return Config::default();
                        }
                    },
                    Err(e) => {
                        eprintln!("[ZalStudio] READ ERROR: {}. Trying next location...", e);
                    }
                }
            }
        }

        // Nothing found — create a default config in OS config dir
        let os_config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zalstudio")
            .join("config.toml");
        let config = Config::default();
        let _ = std::fs::create_dir_all(os_config_path.parent().unwrap());
        let _ = std::fs::write(&os_config_path, toml::to_string_pretty(&config).unwrap_or_default());
        eprintln!("[ZalStudio] No config found. Created default at: {}", os_config_path.display());
        config
    }

    pub fn effective_usb_path(&self) -> PathBuf {
        if cfg!(target_os = "linux") {
            let run_media = PathBuf::from("/run/media");
            if run_media.exists() {
                return run_media;
            }
        }
        self.usb_import_path.clone()
    }

    pub fn printer_for_size(&self, size: &str) -> Option<&str> {
        self.printer_for_size.get(size).map(|s| s.as_str())
    }

    pub fn all_printers(&self) -> Vec<&str> {
        let mut printers: Vec<&str> = self.printer_for_size.values()
            .map(|s| s.as_str())
            .collect();
        printers.sort();
        printers.dedup();
        printers
    }
}

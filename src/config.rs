use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterOptions {
    pub color_correction: String,   // None, Accurate, Bright, Hue, Uncorrected, Raw, etc.
    pub brightness: String,         // None, 1100, 1200, ... or Custom.REAL
    pub contrast: String,           // None, 1100, 1200, ...
    pub saturation: String,         // None, 1100, 1200, ...
    pub gamma: String,              // None, 1100, 1200, ...
    pub cyan_gamma: String,         // None, 1100, ...
    pub magenta_gamma: String,      // None, 1100, ...
    pub yellow_gamma: String,       // None, 1100, ...
    pub cyan_balance: String,       // None, 1100, ...
    pub magenta_balance: String,    // None, 1100, ...
    pub yellow_balance: String,     // None, 1100, ...
}

impl Default for PrinterOptions {
    fn default() -> Self {
        Self {
            color_correction: "Accurate".to_string(),
            brightness: "None".to_string(),
            contrast: "None".to_string(),
            saturation: "None".to_string(),
            gamma: "None".to_string(),
            cyan_gamma: "None".to_string(),
            magenta_gamma: "None".to_string(),
            yellow_gamma: "None".to_string(),
            cyan_balance: "None".to_string(),
            magenta_balance: "None".to_string(),
            yellow_balance: "None".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub photo_directory: PathBuf,
    pub paper_sizes: Vec<String>,
    pub default_paper_size: usize,
    pub copies_default: u32,
    /// Which CUPS media/PageSize to use for each paper size
    pub media_for_size: HashMap<String, String>,
    /// Which CUPS printer to use for each paper size
    pub printer_for_size: HashMap<String, String>,
    /// Per-printer color/quality options (key = CUPS printer name)
    pub printer_options: HashMap<String, PrinterOptions>,
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
    /// Price per print format (currency units, e.g. SEK)
    pub price_per_format: HashMap<String, f64>,
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let mut printer_for_size = HashMap::new();
        printer_for_size.insert("10x15".to_string(), "Printer1".to_string());
        printer_for_size.insert("15x23".to_string(), "Printer2".to_string());

        let mut media_for_size = HashMap::new();
        media_for_size.insert("10x15".to_string(), "w288h432".to_string());
        media_for_size.insert("15x23".to_string(), "w432h648".to_string());

        let mut printer_options = HashMap::new();
        printer_options.insert("Printer1".to_string(), PrinterOptions::default());
        printer_options.insert("Printer2".to_string(), PrinterOptions::default());

        Self {
            photo_directory: PathBuf::from("./photos"),
            paper_sizes: vec!["10x15".to_string(), "15x23".to_string()],
            default_paper_size: 0,
            copies_default: 1,
            media_for_size,
            printer_for_size,
            printer_options,
            usb_import_path: PathBuf::from("/media"),
            google_client_id: String::new(),
            google_client_secret: String::new(),
            temp_directory: PathBuf::from("./temp_imports"),
            wifi_ssid: String::from("ZalStudio"),
            wifi_password: String::from(""),
            server_port: 8080,
            price_per_format: {
                let mut m = HashMap::new();
                m.insert("10x15".to_string(), 15.0);
                m.insert("15x23".to_string(), 25.0);
                m
            },
            source_path: None,
        }
    }
}

impl Config {
    pub fn media_for_size(&self, size: &str) -> Option<&str> {
        self.media_for_size.get(size).map(|s| s.as_str())
    }

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
                        Ok(mut config) => {
                            eprintln!("[ZalStudio] Loaded config from: {}", candidate.display());
                            eprintln!(
                                "[ZalStudio] google_client_id len = {}",
                                config.google_client_id.len()
                            );
                            eprintln!(
                                "[ZalStudio] google_client_secret len = {}",
                                config.google_client_secret.len()
                            );
                            config.source_path = Some(candidate.clone());
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
        let mut config = Config::default();
        let _ = std::fs::create_dir_all(os_config_path.parent().unwrap());
        let _ = std::fs::write(
            &os_config_path,
            toml::to_string_pretty(&config).unwrap_or_default(),
        );
        eprintln!(
            "[ZalStudio] No config found. Created default at: {}",
            os_config_path.display()
        );
        config.source_path = Some(os_config_path);
        config
    }

    pub fn save(&self) -> Result<(), String> {
        let path = self
            .source_path
            .as_ref()
            .cloned()
            .or_else(|| Some(PathBuf::from("config.toml")))
            .ok_or("No config path")?;
        let contents =
            toml::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(&path, contents).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn effective_usb_path(&self) -> PathBuf {
        let run_media = PathBuf::from("/run/media");
        if run_media.exists() {
            return run_media;
        }
        self.usb_import_path.clone()
    }

    pub fn printer_for_size(&self, size: &str) -> Option<&str> {
        self.printer_for_size.get(size).map(|s| s.as_str())
    }

    pub fn all_printers(&self) -> Vec<&str> {
        let mut printers: Vec<&str> = self.printer_for_size.values().map(|s| s.as_str()).collect();
        printers.sort();
        printers.dedup();
        printers
    }

    pub fn price_for_size(&self, size: &str) -> f64 {
        self.price_per_format.get(size).copied().unwrap_or(0.0)
    }

    pub fn options_for_printer(&self, printer_name: &str) -> &PrinterOptions {
        self.printer_options
            .get(printer_name)
            .unwrap_or_else(|| {
                // Return a static default if not configured
                static DEFAULT: std::sync::OnceLock<PrinterOptions> = std::sync::OnceLock::new();
                DEFAULT.get_or_init(PrinterOptions::default)
            })
    }
}

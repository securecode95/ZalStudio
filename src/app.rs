use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use egui::{Context, TextureHandle};

use crate::config::Config;
use crate::gallery::{discover_photos, Photo};
use crate::lang::Language;
use crate::printer::{Printer, PrintJob};

#[cfg(windows)]
#[cfg(windows)]
use crate::powershell_mtp_backend::{PsCommand, PsFolder, PsPhoto, PsResult, spawn_powershell_worker};

// ============================================================================
// App screens
// ============================================================================
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppScreen {
    SourceSelect,
    Gallery,
    Preview,
    Queue,
    Importing,
    MobileUpload,
    MobileMenu,
    PhoneConnecting,
    PhoneFolderSelect,
    WiredPhonePicker,
    GoogleDriveAuth,
    GoogleDrivePicker,
    GooglePhotosPicker,
    PrintDone,
}

// ============================================================================
// Phone type
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoneType {
    Android,
    IPhone,
}

// ============================================================================
// Queue item
// ============================================================================
#[derive(Debug, Clone)]
pub struct QueueItem {
    pub photo_idx: usize,
    pub copies: u32,
    pub paper_size: String,
}

// ============================================================================
// ZalStudio app state
// ============================================================================
pub struct ZalStudio {
    pub screen: AppScreen,
    pub lang: Language,

    // Photos
    pub photos: Vec<Photo>,
    pub selected_photo: usize,

    // Queue / printing
    pub queue: Vec<QueueItem>,
    pub queue_selected: usize,
    pub queue_clear_confirm: bool,
    pub queue_clear_timer: f32,
    pub copies: u32,
    pub paper_size_idx: usize,
    pub printers: Vec<Printer>,
    pub print_jobs: Vec<PrintJob>,
    pub print_done_timer: f32,

    // Config
    pub config: Config,

    // Import
    pub import_status: String,

    // Mobile upload / server
    pub server_url: Option<String>,
    pub server_handle: Option<std::thread::JoinHandle<()>>,
    pub qr_tx: Sender<String>,
    pub qr_rx: Receiver<String>,
    pub qr_texture: Option<TextureHandle>,
    pub wifi_qr_texture: Option<TextureHandle>,

    // Windows hotspot background update
    #[cfg(windows)]
    pub hotspot_rx: Option<Receiver<(String, String, Result<(), String>)>>,

    // Phone connection
    pub phone_type: PhoneType,
    pub phone_connect_start: Option<Instant>,
    pub phone_poll_last: Instant,

    // Windows WPD (PowerShell MTP backend)
    #[cfg(windows)]
    pub wpd_cmd_tx: Option<Sender<PsCommand>>,
    #[cfg(windows)]
    pub wpd_res_rx: Option<Receiver<PsResult>>,
    #[cfg(windows)]
    pub wpd_folders: Vec<PsFolder>,
    #[cfg(windows)]
    pub wpd_photos: Vec<PsPhoto>,
    #[cfg(windows)]
    pub wpd_selected: Vec<bool>,

    // Linux MTP / wired phone
    pub mtp_photos: Vec<crate::wired_import::PhonePhoto>,
    pub mtp_selected: Vec<bool>,
    pub phone_photos: Vec<crate::wired_import::PhonePhoto>,
    pub phone_selected: Vec<bool>,

    // Google Drive
    pub pkce_state: Option<crate::google_drive::PkceState>,
    pub auth_qr_texture: Option<TextureHandle>,
    pub drive_access_token: Option<String>,
    pub drive_files: Vec<crate::google_drive::DriveFile>,
    pub drive_selected: Vec<bool>,
    pub drive_thumbnails: HashMap<String, TextureHandle>,
    pub drive_status: String,

    // Google Photos
    pub photo_items: Vec<crate::google_drive::GooglePhoto>,
    pub photo_selected: Vec<bool>,

    // Toast notification
    pub toast: Option<(String, f32)>,

    // Cached textures
    textures: HashMap<String, TextureHandle>,
}

impl ZalStudio {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load();
        let lang = Language::default();
        let photo_dir = config.photo_directory.clone();
        let photos = discover_photos(&photo_dir);

        let (qr_tx, qr_rx) = channel();

        // Start HTTP server for mobile upload
        let server_port = config.server_port;
        let temp_dir = config.temp_directory.clone();
        let server_handle = std::thread::spawn(move || {
            let _ = crate::server::start_server(temp_dir, server_port);
        });

        let server_url = Some(format!("http://{}:{}", local_ip(), server_port));

        // Generate QR code for server URL
        if let Some(url) = &server_url {
            let _ = qr_tx.send(url.clone());
        }

        // Pre-load textures for all photos
        let mut textures = HashMap::new();
        for photo in &photos {
            if let Some(tex) = load_texture(&cc.egui_ctx, &photo.path) {
                textures.insert(photo.path.to_string_lossy().to_string(), tex);
            }
        }

        // Initialize printers from config
        let printers: Vec<Printer> = config
            .all_printers()
            .into_iter()
            .map(|name| Printer::new(name))
            .collect();

        Self {
            screen: AppScreen::SourceSelect,
            lang,
            photos,
            selected_photo: 0,
            queue: Vec::new(),
            queue_selected: 0,
            queue_clear_confirm: false,
            queue_clear_timer: 0.0,
            copies: config.copies_default,
            paper_size_idx: config.default_paper_size,
            printers,
            print_jobs: Vec::new(),
            print_done_timer: 5.0,
            config,
            import_status: String::new(),
            server_url,
            server_handle: Some(server_handle),
            qr_tx,
            qr_rx,
            qr_texture: None,
            wifi_qr_texture: None,
            #[cfg(windows)]
            hotspot_rx: None,
            phone_type: PhoneType::Android,
            phone_connect_start: None,
            phone_poll_last: Instant::now(),
            #[cfg(windows)]
            wpd_cmd_tx: None,
            #[cfg(windows)]
            wpd_res_rx: None,
            #[cfg(windows)]
            wpd_folders: Vec::new(),
            #[cfg(windows)]
            wpd_photos: Vec::new(),
            #[cfg(windows)]
            wpd_selected: Vec::new(),
            mtp_photos: Vec::new(),
            mtp_selected: Vec::new(),
            phone_photos: Vec::new(),
            phone_selected: Vec::new(),
            pkce_state: None,
            auth_qr_texture: None,
            drive_access_token: None,
            drive_files: Vec::new(),
            drive_selected: Vec::new(),
            drive_thumbnails: HashMap::new(),
            drive_status: String::new(),
            photo_items: Vec::new(),
            photo_selected: Vec::new(),
            toast: None,
            textures,
        }
    }

    // ========================================================================
    // Photo / texture helpers
    // ========================================================================
    pub fn texture_for(&mut self, ctx: &Context, path: &Path) -> Option<&TextureHandle> {
        let key = path.to_string_lossy().to_string();
        if !self.textures.contains_key(&key) {
            if let Some(tex) = load_texture(ctx, path) {
                self.textures.insert(key.clone(), tex);
            }
        }
        self.textures.get(&key)
    }

    pub fn selected_texture(&mut self, ctx: &Context) -> Option<&TextureHandle> {
        let path = self.photos.get(self.selected_photo)?.path.clone();
        self.texture_for(ctx, &path)
    }

    // ========================================================================
    // QR textures
    // ========================================================================
    pub fn qr_texture(&mut self, ctx: &Context) -> Option<&TextureHandle> {
        if self.qr_texture.is_none() {
            while let Ok(url) = self.qr_rx.try_recv() {
                if let Some(tex) = generate_qr_texture(ctx, &url) {
                    self.qr_texture = Some(tex);
                }
            }
        }
        self.qr_texture.as_ref()
    }

    pub fn wifi_qr_texture(&mut self, ctx: &Context) -> Option<&TextureHandle> {
        if self.wifi_qr_texture.is_none() {
            let ssid = &self.config.wifi_ssid;
            let pass = &self.config.wifi_password;
            let wifi_string = if pass.is_empty() {
                format!("WIFI:S:{};T:nopass;;", ssid)
            } else {
                format!("WIFI:S:{};T:WPA;P:{};;", ssid, pass)
            };
            if let Some(tex) = generate_qr_texture(ctx, &wifi_string) {
                self.wifi_qr_texture = Some(tex);
            }
        }
        self.wifi_qr_texture.as_ref()
    }

    pub fn auth_qr_texture(&mut self, _ctx: &Context) -> Option<&TextureHandle> {
        self.auth_qr_texture.as_ref()
    }

    // ========================================================================
    // Gallery
    // ========================================================================
    pub fn rescan(&mut self) {
        self.photos = discover_photos(&self.config.photo_directory);
        self.textures.clear();
    }

    // ========================================================================
    // Import
    // ========================================================================
    pub fn import_usb(&mut self) {
        self.screen = AppScreen::Importing;
        self.import_status = "Söker efter bilder...".to_string();
        let source = self.config.effective_usb_path();
        let dest = self.config.temp_directory.clone();
        match crate::import::import_from_storage(&source, &dest) {
            Ok(count) => {
                self.import_status = format!("{} bilder importerade", count);
                self.rescan();
                self.screen = AppScreen::Gallery;
                self.show_toast(format!("{} bilder importerade", count));
            }
            Err(e) => {
                self.import_status = e.clone();
                self.show_toast_long(e);
            }
        }
    }

    // ========================================================================
    // Printer helper
    // ========================================================================
    pub fn printer_for_current_size(&self) -> Option<&str> {
        let size = self.config.paper_sizes.get(self.paper_size_idx)?;
        self.config.printer_for_size(size)
    }

    // ========================================================================
    // Queue
    // ========================================================================
    pub fn add_to_queue(&mut self) {
        let _ = self.photos.get(self.selected_photo);
        let paper_size = self.config.paper_sizes.get(self.paper_size_idx)
            .cloned()
            .unwrap_or_else(|| "10x15".to_string());
        self.queue.push(QueueItem {
            photo_idx: self.selected_photo,
            copies: self.copies,
            paper_size,
        });
        self.show_toast("Tillagd i kön".to_string());
    }

    pub fn remove_from_queue(&mut self, idx: usize) {
        if idx < self.queue.len() {
            self.queue.remove(idx);
            if self.queue_selected >= self.queue.len() && self.queue_selected > 0 {
                self.queue_selected -= 1;
            }
        }
    }

    // ========================================================================
    // Print
    // ========================================================================
    pub fn print_queue(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        for item in &self.queue {
            let _photo = self.photos.get(item.photo_idx);
            let _printer_name = self.config.printer_for_size(&item.paper_size);
        }

        self.queue.clear();
        self.queue_selected = 0;
        self.screen = AppScreen::PrintDone;
        self.print_done_timer = 5.0;
    }

    // ========================================================================
    // Toast
    // ========================================================================
    pub fn show_toast(&mut self, msg: String) {
        self.toast = Some((msg, 2.0));
    }

    pub fn show_toast_long(&mut self, msg: String) {
        self.toast = Some((msg, 5.0));
    }

    // ========================================================================
    // Mobile upload
    // ========================================================================
    pub fn open_mobile_upload(&mut self) {
        self.screen = AppScreen::MobileUpload;

        #[cfg(windows)]
        {
            // Check if hotspot is already active — if so, skip background thread entirely
            if is_hotspot_active() {
                let ssid = read_windows_hotspot_ssid();
                let pass = read_windows_hotspot_password();
                eprintln!("[Hotspot] Already active: SSID={}, PASS={}", ssid, pass);
                self.hotspot_rx = None;
            } else {
                // Hotspot not active — start it in background but ALWAYS show QR immediately
                let (tx, rx) = channel();
                let ssid = read_windows_hotspot_ssid();
                let pass = read_windows_hotspot_password();
                // Use config values as fallback, and pass them to the hotspot starter
                let ssid = if ssid.is_empty() { self.config.wifi_ssid.clone() } else { ssid };
                let pass = if pass.is_empty() { self.config.wifi_password.clone() } else { pass };
                let hotspot_result = ensure_hotspot_enabled(&ssid, &pass);
                let _ = tx.send((ssid, pass, hotspot_result));
                self.hotspot_rx = Some(rx);
            }
        }
    }

    #[cfg(windows)]
    pub fn check_hotspot_result(&mut self) {
        if let Some(rx) = &self.hotspot_rx {
            if let Ok((_ssid, _pass, result)) = rx.try_recv() {
                self.hotspot_rx = None;
                match result {
                    Ok(()) => {
                        self.show_toast_long("✅ WiFi-hotspot startad!".into());
                    }
                    Err(e) => {
                        eprintln!("[Hotspot] {}", e);
                        if e.to_lowercase().contains("admin") {
                            self.show_toast_long(
                                "⚠️ WiFi-hotspot kräver admin-rättigheter. \
                                 QR-koden visas ändå — aktivera hotspot i Windows-inställningarna om den inte redan är på.".into()
                            );
                        } else {
                            self.show_toast_long(format!("WiFi-hotspot: {}. Använd USB-kabel eller aktivera hotspot i Windows-inställningarna.", e));
                        }
                    }
                }
            }
        }
    }

    // ========================================================================
    // Phone flow
    // ========================================================================
    pub fn start_phone_flow(&mut self, phone_type: PhoneType) {
        self.phone_type = phone_type;
        self.screen = AppScreen::PhoneConnecting;
        self.phone_connect_start = Some(Instant::now());
        self.phone_poll_last = Instant::now();

        #[cfg(windows)]
        if phone_type == PhoneType::Android {
            let (cmd_tx, res_rx) = spawn_powershell_worker();
            self.wpd_cmd_tx = Some(cmd_tx);
            self.wpd_res_rx = Some(res_rx);
            self.wpd_folders.clear();
            self.wpd_photos.clear();
            self.wpd_selected.clear();
        }
    }

    pub fn poll_phone_connection(&mut self) {
        #[cfg(windows)]
        {
            let mut wpd_results = Vec::new();
            if let Some(ref rx) = self.wpd_res_rx {
                while let Ok(result) = rx.try_recv() {
                    wpd_results.push(result);
                }
            }
            for result in wpd_results {
                match result {
                    PsResult::Folders(folders) => {
                        self.wpd_folders = folders;
                        self.screen = AppScreen::PhoneFolderSelect;
                    }
                    PsResult::Photos(photos) => {
                        self.wpd_photos = photos;
                        self.wpd_selected = vec![false; self.wpd_photos.len()];
                        self.screen = AppScreen::WiredPhonePicker;
                    }
                    PsResult::Downloaded { count } => {
                        self.import_status = format!("{} bilder importerade", count);
                        self.rescan();
                        self.screen = AppScreen::Gallery;
                        self.show_toast(format!("{} bilder importerade", count));
                        // Clean up WPD channels
                        self.wpd_cmd_tx = None;
                        self.wpd_res_rx = None;
                    }
                    PsResult::Error(e) => {
                        self.show_toast_long(format!("Fel: {}", e));
                        self.screen = AppScreen::MobileMenu;
                        self.wpd_cmd_tx = None;
                        self.wpd_res_rx = None;
                    }
                }
            }

            // If we're in PhoneConnecting and it's been a while, try listing folders
            if self.screen == AppScreen::PhoneConnecting {
                if let Some(ref tx) = self.wpd_cmd_tx {
                    let _ = tx.send(PsCommand::ListFolders);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: check for GVFS mounts
            let mounts = crate::wired_import::find_phone_mounts();
            if !mounts.is_empty() {
                self.phone_photos = crate::wired_import::list_phone_photos(&mounts[0]);
                self.phone_selected = vec![false; self.phone_photos.len()];
                self.screen = AppScreen::WiredPhonePicker;
            }
        }
    }

    pub fn open_phone_folder(&mut self, folder_path: String) {
        #[cfg(windows)]
        {
            if let Some(ref tx) = self.wpd_cmd_tx {
                let _ = tx.send(PsCommand::ListPhotos { folder_path });
            }
        }
    }

    pub fn import_selected_phone_photos(&mut self) {
        #[cfg(windows)]
        {
            let selected: Vec<PsPhoto> = self.wpd_photos.iter()
                .enumerate()
                .filter(|(i, _)| self.wpd_selected.get(*i).copied().unwrap_or(false))
                .map(|(_, p)| p.clone())
                .collect();

            if !selected.is_empty() {
                if let Some(ref tx) = self.wpd_cmd_tx {
                    let _ = tx.send(PsCommand::Download {
                        photos: selected,
                        dest_dir: self.config.temp_directory.clone(),
                    });
                    self.screen = AppScreen::Importing;
                    self.import_status = "Laddar ner bilder...".to_string();
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let selected: Vec<&crate::wired_import::PhonePhoto> = self.phone_photos.iter()
                .enumerate()
                .filter(|(i, _)| self.phone_selected.get(*i).copied().unwrap_or(false))
                .map(|(_, p)| p)
                .collect();

            let mut count = 0;
            for photo in selected {
                let dest = self.config.temp_directory.join(&photo.file_name);
                if std::fs::copy(&photo.path, dest).is_ok() {
                    count += 1;
                }
            }

            self.import_status = format!("{} bilder importerade", count);
            self.rescan();
            self.screen = AppScreen::Gallery;
            self.show_toast(format!("{} bilder importerade", count));
        }
    }

    // ========================================================================
    // Google Drive
    // ========================================================================
    pub fn start_google_photos_auth(&mut self) {
        let pkce = crate::google_drive::generate_pkce(format!("http://localhost:{}/oauth", self.config.server_port));
        self.pkce_state = Some(pkce.clone());
        let auth_url = crate::google_drive::build_auth_url(
            &self.config.google_client_id,
            &pkce.redirect_uri,
            &pkce.state,
            &pkce.code_challenge,
        );
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("rundll32")
                .args(["url.dll,FileProtocolHandler", &auth_url])
                .spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&auth_url)
                .spawn();
        }
    }

    pub fn drive_thumbnail(&mut self, ctx: &Context, file_id: &str, url: &str) -> Option<&TextureHandle> {
        if !self.drive_thumbnails.contains_key(file_id) {
            if let Ok(response) = ureq::get(url).call() {
                if let Ok(bytes) = response.into_body().read_to_vec() {
                    if let Ok(image) = image::load_from_memory(&bytes) {
                        let size = 128;
                        let image = image.resize_to_fill(size, size, image::imageops::FilterType::Triangle);
                        let rgba = image.to_rgba8();
                        let tex = ctx.load_texture(
                            file_id,
                            egui::ColorImage::from_rgba_unmultiplied([size as usize, size as usize], &rgba.into_raw()),
                            egui::TextureOptions::default(),
                        );
                        self.drive_thumbnails.insert(file_id.to_string(), tex);
                    }
                }
            }
        }
        self.drive_thumbnails.get(file_id)
    }

    pub fn download_selected_drive_files(&mut self) {
        if let Some(token) = &self.drive_access_token {
            let selected: Vec<String> = self.drive_files.iter()
                .enumerate()
                .filter(|(i, _)| self.drive_selected.get(*i).copied().unwrap_or(false))
                .map(|(_, f)| f.id.clone())
                .collect();

            self.screen = AppScreen::Importing;
            self.import_status = "Laddar ner från Drive...".to_string();
            let token = token.clone();
            let dest = self.config.temp_directory.clone();
            let tx = self.qr_tx.clone();

            std::thread::spawn(move || {
                let mut count = 0;
                for file_id in selected {
                    match crate::google_drive::download_drive_file(&file_id, &token, &dest) {
                        Ok(_) => count += 1,
                        Err(e) => eprintln!("[Drive download] {}", e),
                    }
                }
                let _ = tx.send(format!("DOWNLOADED:{}", count));
            });
        }
    }

    pub fn download_selected_google_photos(&mut self) {
        if let Some(token) = &self.drive_access_token {
            let selected: Vec<String> = self.photo_items.iter()
                .enumerate()
                .filter(|(i, _)| self.photo_selected.get(*i).copied().unwrap_or(false))
                .map(|(_, p)| p.id.clone())
                .collect();

            self.screen = AppScreen::Importing;
            self.import_status = "Laddar ner från Google Foto...".to_string();
            let token = token.clone();
            let dest = self.config.temp_directory.clone();
            let tx = self.qr_tx.clone();

            std::thread::spawn(move || {
                let mut count = 0;
                for photo_id in selected {
                    match crate::google_drive::download_google_photo(&token, &photo_id, &dest) {
                        Ok(_) => count += 1,
                        Err(e) => eprintln!("[Photos download] {}", e),
                    }
                }
                let _ = tx.send(format!("DOWNLOADED:{}", count));
            });
        }
    }
}

// ============================================================================
// eframe App trait
// ============================================================================
impl eframe::App for ZalStudio {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Update toast timer
        if let Some((_, ref mut t)) = self.toast {
            *t -= ctx.input(|i| i.unstable_dt);
            if *t <= 0.0 {
                self.toast = None;
            }
        }

        // Update print done timer
        if self.screen == AppScreen::PrintDone {
            self.print_done_timer -= ctx.input(|i| i.unstable_dt);
            if self.print_done_timer <= 0.0 {
                self.screen = AppScreen::Gallery;
            }
        }

        // Update queue clear timer
        if self.queue_clear_confirm {
            self.queue_clear_timer -= ctx.input(|i| i.unstable_dt);
            if self.queue_clear_timer <= 0.0 {
                self.queue_clear_confirm = false;
            }
        }

        // Check for QR code updates
        if self.qr_texture.is_none() {
            while let Ok(url) = self.qr_rx.try_recv() {
                if url.starts_with("DOWNLOADED:") {
                    let count: usize = url.trim_start_matches("DOWNLOADED:").parse().unwrap_or(0);
                    self.import_status = format!("{} bilder importerade", count);
                    self.rescan();
                    self.screen = AppScreen::Gallery;
                    self.show_toast(format!("{} bilder importerade", count));
                } else if let Some(tex) = generate_qr_texture(ctx, &url) {
                    self.qr_texture = Some(tex);
                }
            }
        }

        // Check hotspot result on Windows
        #[cfg(windows)]
        self.check_hotspot_result();

        // Poll phone connection
        if self.screen == AppScreen::PhoneConnecting {
            if self.phone_poll_last.elapsed() > Duration::from_secs(1) {
                self.phone_poll_last = Instant::now();
                self.poll_phone_connection();
            }
        }

        crate::ui::draw(ctx, self);
    }
}

// ============================================================================
// Helpers
// ============================================================================
fn load_texture(ctx: &Context, path: &Path) -> Option<TextureHandle> {
    let image = image::open(path).ok()?;
    let size = 256;
    let image = image.resize_to_fill(size, size, image::imageops::FilterType::Triangle);
    let rgba = image.to_rgba8();
    Some(ctx.load_texture(
        path.to_string_lossy(),
        egui::ColorImage::from_rgba_unmultiplied([size as usize, size as usize], &rgba.into_raw()),
        egui::TextureOptions::default(),
    ))
}

fn generate_qr_texture(ctx: &Context, data: &str) -> Option<TextureHandle> {
    let code = qrcode::QrCode::new(data.as_bytes()).ok()?;
    let size = code.width();
    let scale = 4;
    let img_size = size * scale;
    let mut pixels = vec![255u8; img_size * img_size * 4];

    for y in 0..size {
        for x in 0..size {
            let dark = code[(x, y)] == qrcode::Color::Dark;
            let color = if dark { 0 } else { 255 };
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x * scale + dx;
                    let py = y * scale + dy;
                    let idx = (py * img_size + px) * 4;
                    pixels[idx] = color;
                    pixels[idx + 1] = color;
                    pixels[idx + 2] = color;
                    pixels[idx + 3] = 255;
                }
            }
        }
    }

    Some(ctx.load_texture(
        "qr",
        egui::ColorImage::from_rgba_unmultiplied([img_size, img_size], &pixels),
        egui::TextureOptions::default(),
    ))
}

fn local_ip() -> String {
    // Try to get a reasonable local IP
    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", "(Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -like '192.168.*' -or $_.IPAddress -like '10.*' -or $_.IPAddress -like '172.*' } | Select-Object -First 1).IPAddress"])
            .output()
        {
            let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !ip.is_empty() {
                return ip;
            }
        }
    }
    "localhost".to_string()
}

// ============================================================================
// Windows Hotspot info
// ============================================================================
#[cfg(windows)]
fn read_hotspot_from_netsh(field: &str) -> Option<String> {
    let cmd = if field == "SSID name" {
        format!("netsh wlan show hostednetwork 2>$null | Select-String '{}'", field)
    } else {
        format!("netsh wlan show hostednetwork setting=security 2>$null | Select-String '{}'", field)
    };
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .ok()?;
    let out = String::from_utf8_lossy(&output.stdout);
    out.split(':').nth(1).map(|s| s.trim().trim_matches('"').to_string())
}

#[cfg(windows)]
fn read_windows_hotspot_value(name: &str) -> Option<String> {
    let cmd = format!(
        "Get-ItemProperty -Path 'HKLM:\\SYSTEM\\CurrentControlSet\\Services\\icssvc\\Settings' -Name '{}' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty '{}'",
        name, name
    );
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &cmd])
        .output()
        .ok()?;
    let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if out.is_empty() { None } else { Some(out) }
}

#[cfg(windows)]
pub fn read_windows_hotspot_ssid() -> String {
    read_hotspot_from_netsh("SSID name")
        .or_else(|| read_windows_hotspot_value("SSID"))
        .unwrap_or_default()
}

#[cfg(windows)]
pub fn read_windows_hotspot_password() -> String {
    read_hotspot_from_netsh("User security key")
        .or_else(|| read_windows_hotspot_value("Passphrase"))
        .unwrap_or_default()
}

/// Check if any Windows hotspot (old hostednetwork or modern Mobile Hotspot) is active.
#[cfg(windows)]
fn is_hotspot_active() -> bool {
    // Method 1: Old netsh hostednetwork
    if let Ok(o) = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", "netsh wlan show hostednetwork 2>$null | Select-String 'Status'"])
        .output()
    {
        let out = String::from_utf8_lossy(&o.stdout);
        if out.to_lowercase().contains("started") {
            return true;
        }
    }
    // Method 2: Modern Mobile Hotspot — check for active WiFi Direct virtual adapter
    if let Ok(o) = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command",
            "Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object { $_.InterfaceDescription -match 'Wi-Fi Direct|Virtual Hosted Network' -and $_.Status -eq 'Up' } | Select-Object -First 1"
        ])
        .output()
    {
        let out = String::from_utf8_lossy(&o.stdout);
        if out.trim().lines().count() >= 2 { // header + data line
            return true;
        }
    }
    // Method 3: Check if a 192.168.137.x gateway exists (Mobile Hotspot default subnet)
    if let Ok(o) = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command",
            "Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue | Where-Object { $_.NextHop -like '192.168.137.*' } | Select-Object -First 1"
        ])
        .output()
    {
        let out = String::from_utf8_lossy(&o.stdout);
        if !out.trim().is_empty() {
            return true;
        }
    }
    false
}

#[cfg(windows)]
pub fn ensure_hotspot_enabled(ssid: &str, pass: &str) -> Result<(), String> {
    // If already active, nothing to do.
    if is_hotspot_active() {
        return Ok(());
    }

    // Check if hosted network is supported by the driver
    let check = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", "netsh wlan show drivers | findstr 'Hosted network supported'"])
        .output();
    let hosted_supported = match check {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_lowercase().contains("yes"),
        Err(_) => false,
    };

    if !hosted_supported {
        // Try modern Mobile Hotspot API (Windows 10/11)
        return ensure_modern_mobile_hotspot(ssid, pass);
    }

    // Build a PowerShell script that configures + starts the hosted network.
    // This requires admin privileges. We do NOT check admin via .NET API —
    // instead we just run netsh and let Windows tell us if admin is missing.
    let script = format!(r#"
# Disconnect from any existing WiFi first (hosted network needs the adapter free)
netsh wlan disconnect 2>$null | Out-Null
Start-Sleep -Milliseconds 500

# Configure the hosted network — capture ALL output (stdout + stderr)
$output1 = (netsh wlan set hostednetwork mode=allow ssid="{ssid}" key="{pass}") 2>&1 | Out-String
Write-Output ("SET_OUTPUT:" + $output1.Trim())

# Start the hosted network
$output2 = (netsh wlan start hostednetwork) 2>&1 | Out-String
Write-Output ("START_OUTPUT:" + $output2.Trim())

# Verify status
$status = (netsh wlan show hostednetwork | Select-String 'Status') | Out-String
Write-Output ("STATUS:" + $status.Trim())
"#, ssid = ssid, pass = pass);

    let temp_path = std::env::temp_dir().join("zalstudio_start_hotspot.ps1");
    let _ = std::fs::write(&temp_path, &script);
    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            temp_path.to_str().unwrap_or(""),
        ])
        .output();

    match output {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            eprintln!("[Hotspot script] FULL OUTPUT:\n{}", text);
            if !stderr.is_empty() {
                eprintln!("[Hotspot script] STDERR:\n{}", stderr);
            }

            let text_lower = text.to_lowercase();

            // Check for admin privilege message anywhere in output
            if text_lower.contains("administrator privilege") || text_lower.contains("administratörsbehörighet") {
                return Err(
                    "WiFi-hotspot kräver admin-rättigheter. \
                     Starta appen som administratör, eller aktivera Mobile Hotspot manuellt i Windows-inställningarna.".into()
                );
            }

            // Check for success indicators
            if text_lower.contains("the hosted network started") || text_lower.contains("started") {
                return Ok(());
            }

            // Check for explicit failure messages
            if text_lower.contains("could not be started") || text_lower.contains("couldn't be started") {
                return Err(format!("Hotspot kunde inte startas. Kontrollera att WiFi-adaptern är på och inte upptagen.\n\nDetaljer:\n{}", text));
            }

            // Unknown result — return everything for debugging
            Err(format!("Hotspot kunde inte startas:\n{}", text))
        }
        Err(e) => Err(format!("PowerShell-fel: {}", e)),
    }
}

/// Try to enable the modern Windows Mobile Hotspot (Windows 10 1607+ / Windows 11).
/// Uses the Windows.Networking.NetworkOperators.TetheringManager WinRT API via PowerShell.
#[cfg(windows)]
fn ensure_modern_mobile_hotspot(ssid: &str, pass: &str) -> Result<(), String> {
    eprintln!("[Hotspot] Hosted network not supported. Trying modern Mobile Hotspot API...");

    // PowerShell script using Windows Runtime TetheringManager API
    let script = format!(r#"
# Check for active WiFi adapter first
$adapter = Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object {{ $_.InterfaceDescription -match 'Wi-Fi|Wireless' -and $_.Status -eq 'Up' }} | Select-Object -First 1
if (-not $adapter) {{
    # Also check for any wireless adapter even if not "Up" (might be disconnected)
    $adapter = Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object {{ $_.InterfaceDescription -match 'Wi-Fi|Wireless' }} | Select-Object -First 1
    if (-not $adapter) {{
        Write-Output "ERROR:NO_WIFI_ADAPTER"
        exit 1
    }}
    # Adapter exists but might be disabled/disconnected — still worth trying
}}

# Method 1: Try Windows Runtime TetheringManager API (most reliable)
try {{
    # Load Windows Runtime
    [Windows.System.Profile.SystemManufacturers.SmbiosInformation, Windows.System.Profile.SystemManufacturers, ContentType=WindowsRuntime] | Out-Null

    # Get the TetheringManager for the WiFi adapter
    $connectionProfile = [Windows.Networking.Connectivity.NetworkInformation, Windows.Networking.Connectivity, ContentType=WindowsRuntime]::GetInternetConnectionProfile()
    $tetheringManager = [Windows.Networking.NetworkOperators.NetworkOperatorTetheringManager, Windows.Networking.NetworkOperators, ContentType=WindowsRuntime]::CreateFromConnectionProfile($connectionProfile)

    # Check current state
    $tetheringState = $tetheringManager.TetheringOperationalState
    Write-Output ("INFO:TetheringState=" + $tetheringState)

    if ($tetheringState -eq 'On') {{
        Write-Output "SUCCESS:ALREADY_ON"
        exit 0
    }}

    # Configure hotspot settings
    $config = $tetheringManager.GetCurrentAccessPointConfiguration()
    $config.Ssid = "{ssid}"
    $config.Passphrase = "{pass}"

    # Apply configuration
    $configureResult = $tetheringManager.ConfigureAccessPointAsync($config)
    # Wait for async operation
    while ($configureResult.Status -eq 'Started') {{ Start-Sleep -Milliseconds 100 }}

    if ($configureResult.Status -eq 'Error') {{
        Write-Output ("ERROR:CONFIGURE_FAILED:" + $configureResult.ErrorCode)
        exit 1
    }}

    # Start tethering
    $startResult = $tetheringManager.StartTetheringAsync()
    while ($startResult.Status -eq 'Started') {{ Start-Sleep -Milliseconds 100 }}

    if ($startResult.Status -eq 'Completed') {{
        $result = $startResult.GetResults()
        if ($result.Status -eq 'Success' -or $result.Status -eq 'AlreadyOn') {{
            Write-Output "SUCCESS:TETHERING_API"
            exit 0
        }} else {{
            Write-Output ("ERROR:TETHERING_START_FAILED:" + $result.Status)
            exit 1
        }}
    }} else {{
        Write-Output ("ERROR:TETHERING_ASYNC_FAILED:" + $startResult.ErrorCode)
        exit 1
    }}
}} catch {{
    $err = $_.Exception.Message
    Write-Output ("INFO:TetheringAPI_failed:" + $err)
}}

# Method 2: Try to enable via registry + service restart (requires admin)
try {{
    $regPath = 'HKLM:\SYSTEM\CurrentControlSet\Services\icssvc\Settings'

    # Windows stores hotspot config as binary blobs. The format is complex,
    # but we can try a simpler approach: just ensure the service is running
    # and hope the user has already configured hotspot settings in Windows.

    $svc = Get-Service icssvc -ErrorAction SilentlyContinue
    if ($svc -and $svc.Status -ne 'Running') {{
        Start-Service icssvc -ErrorAction Stop
        Write-Output "SUCCESS:SERVICE_STARTED"
        exit 0
    }}
}} catch {{
    $err = $_.Exception.Message
    if ($err -match 'denied' -or $err -match 'behörighet' -or $err -match 'Access is denied') {{
        Write-Output "ERROR:ADMIN_REQUIRED"
        exit 1
    }}
    Write-Output ("INFO:Service_method_failed:" + $err)
}}

# Method 3: Check if Mobile Hotspot is already configured and just needs to be toggled
# Try using the Settings app's URI scheme to open hotspot settings
Write-Output "INFO:MANUAL_REQUIRED"
"#, ssid = ssid, pass = pass);

    let temp_path = std::env::temp_dir().join("zalstudio_modern_hotspot.ps1");
    let _ = std::fs::write(&temp_path, &script);
    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            temp_path.to_str().unwrap_or(""),
        ])
        .output();

    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            eprintln!("[Modern Hotspot] stdout: {}", text);
            if !stderr.is_empty() {
                eprintln!("[Modern Hotspot] stderr: {}", stderr);
            }

            if text.contains("SUCCESS") {
                // Give Windows a moment to bring up the hotspot
                std::thread::sleep(std::time::Duration::from_secs(2));
                if is_hotspot_active() {
                    return Ok(());
                }
                std::thread::sleep(std::time::Duration::from_secs(3));
                if is_hotspot_active() {
                    return Ok(());
                }
                return Err(
                    "Mobile Hotspot aktiverades men är inte redo än. \
                     Vänta några sekunder och försök igen, eller aktivera det manuellt i Windows-inställningarna.".into()
                );
            }

            if text.contains("ADMIN_REQUIRED") || text.contains("denied") || text.contains("Access is denied") {
                return Err(
                    "WiFi-hotspot kräver admin-rättigheter för att konfigurera Mobile Hotspot. \
                     Starta appen som administratör, eller aktivera Mobile Hotspot manuellt i Windows-inställningarna.".into()
                );
            }

            if text.contains("NO_WIFI_ADAPTER") {
                return Err(
                    "Ingen WiFi-adapter hittades. Kontrollera att WiFi är påslaget.".into()
                );
            }

            // All automatic methods failed — guide user to manual setup
            Err(
                "Automatisk hotspot kunde inte startas. \
                 Din WiFi-adaptern (Qualcomm Atheros) stöder inte äldre hosted network.\n\n\
                 Lösningar:\n\
                 1. Aktivera Mobile Hotspot manuellt: Inställningar → Nätverk → Mobile Hotspot\n\
                 2. Använd USB-kabel istället för WiFi\n\
                 3. Uppdatera WiFi-drivrutinen från tillverkaren".into()
            )
        }
        Err(e) => Err(format!("Kunde inte köra Mobile Hotspot-skript: {}", e)),
    }
}

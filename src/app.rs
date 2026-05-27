use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use egui::{Context, TextureHandle};

use crate::config::Config;
use crate::gallery::{Photo, discover_photos};
use crate::lang::Language;
use crate::printer::{JobStatus, PrintJob, Printer};

use crate::mtp_backend::{MtpCommand, MtpPhoto, MtpResult, spawn_mtp_worker};

// ============================================================================
// App screens
// ============================================================================
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppScreen {
    ProductSelect,
    SourceSelect,
    Gallery,
    Preview,
    Queue,
    Importing,
    MobileUpload,
    MobileMenu,
    UsbPlugWait,
    PhoneConnecting,
    PhoneFolderSelect,
    WiredPhonePicker,
    GoogleDriveAuth,
    GoogleDrivePicker,
    GooglePhotosPicker,
    PrintDone,
    PrintProgress,
    ThankYou,
    Payment,
    LayoutSelect,
    CollageEditor,
    SettingsAuth,
    Settings,
}

// ============================================================================
// Phone type
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoneType {
    Android,
    IPhone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GallerySort {
    Date,
    Name,
}

// ============================================================================
// Picker source (wired phone picker can be used for phone or USB)
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerSource {
    Phone,
    Usb,
}

// ============================================================================
// Photo edit state (crop, zoom, rotate, filter)
// ============================================================================
#[derive(Debug, Clone)]
pub struct PhotoEdit {
    pub zoom: f32,     // 1.0 = cover-fit, >1.0 = zoomed in
    pub pan_x: f32,    // -1.0 .. 1.0 horizontal pan
    pub pan_y: f32,    // -1.0 .. 1.0 vertical pan
    pub rotation: u32, // 0, 90, 180, 270 degrees
    pub grayscale: bool,
    pub text_overlay: String, // text to draw on image
    pub text_x: f32,          // 0.0..1.0 relative position
    pub text_y: f32,          // 0.0..1.0 relative position
    pub text_size: u32,       // font size in pixels on output
}

impl Default for PhotoEdit {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            rotation: 0,
            grayscale: false,
            text_overlay: String::new(),
            text_x: 0.5,
            text_y: 0.5,
            text_size: 48,
        }
    }
}

// ============================================================================
// Queue item
// ============================================================================
#[derive(Debug, Clone)]
pub struct QueueItem {
    pub photo_idx: usize,
    pub copies: u32,
    pub paper_size: String,
    pub edit: PhotoEdit,
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
    /// Multi-selected photo indices for batch ordering (new gallery flow)
    pub selected_photos: Vec<usize>,
    /// Copies per photo per size: photo_idx -> size -> count
    pub photo_copies: HashMap<usize, HashMap<String, u32>>,

    // Current edit settings for the photo being previewed
    pub current_edit: PhotoEdit,

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
    pub print_progress_total: usize,
    pub print_progress_done: usize,
    pub print_progress_failed: usize,
    pub print_progress_start: Option<Instant>,
    pub thank_you_timer: f32,
    /// Maps print job id -> history folder path for status updates
    pub history_job_map: std::collections::HashMap<usize, PathBuf>,

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

    // Per-session WiFi credentials (randomized for each mobile upload)
    pub session_ssid: Option<String>,
    pub session_password: Option<String>,

    // Hotspot background result
    pub hotspot_rx: Option<Receiver<(String, String, Result<(), String>)>>,

    // Previous WiFi connection to restore when hotspot stops
    pub previous_wifi_conn: Option<String>,

    // USB plug-and-wait background worker
    pub usb_rx: Option<Receiver<crate::usb_detect::UsbResult>>,

    // Thumbnail background loader
    pub thumb_rx: Option<Receiver<(String, egui::ColorImage)>>,

    // Save-edit background worker
    pub save_rx: Option<Receiver<Result<(), String>>>,
    pub save_in_progress: bool,

    // Phone connection
    pub phone_type: PhoneType,
    pub phone_connect_start: Option<Instant>,
    pub phone_poll_last: Instant,

    // Native MTP backend
    pub mtp_cmd_tx: Option<Sender<MtpCommand>>,
    pub mtp_res_rx: Option<Receiver<MtpResult>>,
    /// Raw MtpPhoto list kept for downloading (populated when native MTP is used)
    pub mtp_raw_photos: Vec<MtpPhoto>,

    // Linux MTP / wired phone
    pub mtp_photos: Vec<crate::wired_import::PhonePhoto>,
    pub mtp_selected: Vec<bool>,
    pub phone_photos: Vec<crate::wired_import::PhonePhoto>,
    pub phone_selected: Vec<bool>,

    // Product selection (Foto / Album / Collage)
    pub selected_product: String,
    pub selected_product_size: String,

    // Gallery sorting
    pub gallery_sort: GallerySort,
    pub gallery_sort_ascending: bool,

    // Collage / Album
    pub selected_layout: Option<crate::collage::CollageLayout>,
    pub collage_photo_indices: Vec<usize>,
    pub collage_preview_path: Option<PathBuf>,

    // Picker context
    pub picker_source: PickerSource,

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

    // Texture loading throttle (prevents UI freeze with many photos)
    pub textures_loaded_this_frame: usize,
    pub last_texture_pass: u64,
    pub max_textures_per_frame: usize,

    // Print history
    pub history_folders: Vec<std::path::PathBuf>,

    // Settings / admin
    pub settings_pin_input: String,
    pub settings_auth_failed: bool,
    pub settings_tab: SettingsTab,
    pub settings_price_edit: HashMap<String, String>,
    pub settings_product_active: HashMap<String, bool>,
    pub settings_save_confirm: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Products,
    Prices,
    General,
    Dispatcher,
}

impl ZalStudio {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Request window focus immediately on X11 kiosk setups
        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Focus);

        // Load system symbol/emoji fonts for broad Unicode coverage (icons)
        let mut fonts = egui::FontDefinitions::default();

        // DejaVu Sans has excellent symbol/geometric-shape coverage
        if let Ok(dejavu_data) = std::fs::read("/usr/share/fonts/TTF/DejaVuSans.ttf") {
            fonts.font_data.insert(
                "dejavu_sans".to_owned(),
                egui::FontData::from_owned(dejavu_data),
            );
        }

        // Noto Color Emoji for full-colour emojis
        if let Ok(emoji_data) = std::fs::read("/usr/share/fonts/noto/NotoColorEmoji.ttf") {
            fonts.font_data.insert(
                "noto_color_emoji".to_owned(),
                egui::FontData::from_owned(emoji_data).tweak(egui::FontTweak {
                    scale: 0.90,
                    ..Default::default()
                }),
            );
        }

        // Insert as fallbacks AFTER the built-in fonts so built-ins are tried first
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            let list = fonts.families.entry(family).or_default();
            if fonts.font_data.contains_key("dejavu_sans")
                && !list.contains(&"dejavu_sans".to_owned())
            {
                list.push("dejavu_sans".to_owned());
            }
            if fonts.font_data.contains_key("noto_color_emoji")
                && !list.contains(&"noto_color_emoji".to_owned())
            {
                list.push("noto_color_emoji".to_owned());
            }
        }

        cc.egui_ctx.set_fonts(fonts);

        let config = Config::load();
        let lang = Language::default();
        let photo_dir = config.photo_directory.clone();
        let mut photos = discover_photos(&photo_dir);
        let mut temp_photos = discover_photos(&config.temp_directory);
        photos.append(&mut temp_photos);

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

        // Initialize printers from config
        let printers: Vec<Printer> = config
            .all_printers()
            .into_iter()
            .map(|name| Printer::new(name))
            .collect();

        Self {
            screen: AppScreen::ProductSelect,
            lang,
            photos,
            selected_photo: 0,
            selected_photos: Vec::new(),
            photo_copies: HashMap::new(),
            current_edit: PhotoEdit::default(),
            queue: Vec::new(),
            queue_selected: 0,
            queue_clear_confirm: false,
            queue_clear_timer: 0.0,
            copies: config.copies_default,
            paper_size_idx: config.default_paper_size,
            printers,
            print_jobs: Vec::new(),
            print_done_timer: 5.0,
            print_progress_total: 0,
            print_progress_done: 0,
            print_progress_failed: 0,
            print_progress_start: None,
            thank_you_timer: 5.0,
            history_folders: Vec::new(),
            history_job_map: std::collections::HashMap::new(),
            config,
            import_status: String::new(),
            server_url,
            server_handle: Some(server_handle),
            qr_tx,
            qr_rx,
            qr_texture: None,
            wifi_qr_texture: None,
            session_ssid: None,
            session_password: None,
            hotspot_rx: None,
            previous_wifi_conn: None,
            usb_rx: None,
            thumb_rx: None,
            save_rx: None,
            save_in_progress: false,
            phone_type: PhoneType::Android,
            phone_connect_start: None,
            phone_poll_last: Instant::now(),
            mtp_cmd_tx: None,
            mtp_res_rx: None,
            mtp_raw_photos: Vec::new(),
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
            picker_source: PickerSource::Phone,
            selected_product: String::new(),
            selected_product_size: String::new(),
            gallery_sort: GallerySort::Date,
            gallery_sort_ascending: false,
            selected_layout: None,
            collage_photo_indices: Vec::new(),
            collage_preview_path: None,
            toast: None,
            textures: HashMap::new(),
            textures_loaded_this_frame: 0,
            last_texture_pass: 0,
            max_textures_per_frame: 4,
            settings_pin_input: String::new(),
            settings_auth_failed: false,
            settings_tab: SettingsTab::Products,
            settings_price_edit: HashMap::new(),
            settings_product_active: HashMap::new(),
            settings_save_confirm: 0.0,
        }
    }

    // ========================================================================
    // Photo / texture helpers
    // ========================================================================
    /// Look up a cached texture without triggering a load.
    pub fn cached_texture(&self, path: &Path) -> Option<&TextureHandle> {
        let key = path.to_string_lossy().to_string();
        self.textures.get(&key)
    }

    pub fn texture_for(&mut self, ctx: &Context, path: &Path) -> Option<&TextureHandle> {
        let key = path.to_string_lossy().to_string();
        if !self.textures.contains_key(&key) {
            let current_pass = ctx.cumulative_pass_nr();
            if self.last_texture_pass != current_pass {
                self.last_texture_pass = current_pass;
                self.textures_loaded_this_frame = 0;
            }
            if self.textures_loaded_this_frame >= self.max_textures_per_frame {
                ctx.request_repaint();
                return None;
            }
            if let Some(tex) = load_texture(ctx, path) {
                self.textures.insert(key.clone(), tex);
                self.textures_loaded_this_frame += 1;
                ctx.request_repaint();
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
            let ssid = self
                .session_ssid
                .as_ref()
                .unwrap_or(&self.config.wifi_ssid)
                .clone();
            let pass = self
                .session_password
                .as_ref()
                .unwrap_or(&self.config.wifi_password)
                .clone();
            let ssid = wifi_qr_escape(&ssid);
            let pass = wifi_qr_escape(&pass);
            let wifi_string = if pass.is_empty() {
                format!("WIFI:T:nopass;S:{};H:false;;", ssid)
            } else {
                format!("WIFI:T:WPA;S:{};P:{};H:false;;", ssid, pass)
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
        let mut photos = discover_photos(&self.config.photo_directory);
        let mut temp_photos = discover_photos(&self.config.temp_directory);
        photos.append(&mut temp_photos);
        self.photos = photos;
        self.apply_gallery_sort();
        self.textures.clear();
    }

    pub fn apply_gallery_sort(&mut self) {
        self.photos.sort_by(|a, b| {
            let ord = match self.gallery_sort {
                GallerySort::Date => match (a.date_taken, b.date_taken) {
                    (Some(d1), Some(d2)) => d1.cmp(&d2),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.file_name.cmp(&b.file_name),
                },
                GallerySort::Name => a.file_name.cmp(&b.file_name),
            };
            if self.gallery_sort_ascending {
                ord
            } else {
                ord.reverse()
            }
        });
    }

    /// After an import, jump directly to Preview with the first newly-imported
    /// photo selected so the user can immediately set size, copies, etc.
    pub fn enter_preview_for_imported(&mut self) {
        if self.photos.is_empty() {
            self.screen = AppScreen::Gallery;
            return;
        }
        // For collage / album products, go to layout selection first
        if self.selected_product == "collage" || self.selected_product == "album" {
            self.collage_photo_indices.clear();
            self.selected_layout = None;
            self.screen = AppScreen::LayoutSelect;
            return;
        }
        // For normal foto prints, go to the gallery so user can select multiple
        // photos and set copies per size
        self.selected_photos.clear();
        self.photo_copies.clear();
        self.screen = AppScreen::Gallery;
    }

    // ========================================================================
    // Import
    // ========================================================================
    pub fn import_usb(&mut self) {
        self.start_usb_wait();
    }

    fn start_usb_wait(&mut self) {
        self.screen = AppScreen::UsbPlugWait;
        let dest = self.config.temp_directory.clone();
        self.usb_rx = Some(crate::usb_detect::spawn_usb_worker(dest));
    }

    fn handle_usb_result(&mut self, result: crate::usb_detect::UsbResult) {
        match result {
            crate::usb_detect::UsbResult::Mounted(path) => {
                self.screen = AppScreen::Importing;
                self.import_status = format!("Importerar från {}...", path.display());
            }
            crate::usb_detect::UsbResult::FoundPhotos(photos) => {
                self.usb_rx = None;
                self.phone_photos = photos.clone();
                self.phone_selected = vec![false; self.phone_photos.len()];
                self.picker_source = PickerSource::Usb;
                self.phone_type = PhoneType::IPhone; // force GVFS fallback path in picker
                self.spawn_thumbnail_worker(photos);
                self.screen = AppScreen::WiredPhonePicker;
            }
            crate::usb_detect::UsbResult::Imported(count) => {
                self.usb_rx = None;
                self.import_status = format!("{} bilder importerade", count);
                eprintln!("[IMPORT] Starting rescan after import...");
                self.rescan();
                eprintln!("[IMPORT] rescan done, {} photos total", self.photos.len());
                self.enter_preview_for_imported();
                self.show_toast(format!("{} bilder importerade", count));
                eprintln!("[IMPORT] Switched to Gallery");
            }
            crate::usb_detect::UsbResult::Error(e) => {
                self.usb_rx = None;
                self.import_status = e.clone();
                self.show_toast_long(e);
                self.screen = AppScreen::SourceSelect;
            }
        }
    }

    fn spawn_thumbnail_worker(&mut self, photos: Vec<crate::wired_import::PhonePhoto>) {
        let (tx, rx) = channel::<(String, egui::ColorImage)>();
        self.thumb_rx = Some(rx);

        // Use multiple threads for faster thumbnail generation
        let num_workers = std::thread::available_parallelism()
            .map(|n| n.get().min(8))
            .unwrap_or(2);
        let chunk_size = (photos.len() + num_workers - 1) / num_workers;
        let chunks: Vec<Vec<crate::wired_import::PhonePhoto>> =
            photos.chunks(chunk_size).map(|c| c.to_vec()).collect();

        for chunk in chunks {
            let tx = tx.clone();
            std::thread::spawn(move || {
                for photo in chunk {
                    let path = photo.path;
                    let key = path.to_string_lossy().to_string();
                    if let Some(color_image) = thumbnail_color_image(&path, 140) {
                        if tx.send((key, color_image)).is_err() {
                            break;
                        }
                    }
                }
            });
        }
    }

    // ========================================================================
    // Printer helper
    // ========================================================================
    pub fn printer_for_current_size(&self) -> Option<&str> {
        self.config.printer_for_size(&self.selected_product_size)
    }

    // ========================================================================
    // Queue
    // ========================================================================
    pub fn add_to_queue(&mut self) {
        let _ = self.photos.get(self.selected_photo);
        let paper_size = self.selected_product_size.clone();
        self.queue.push(QueueItem {
            photo_idx: self.selected_photo,
            copies: self.copies,
            paper_size,
            edit: PhotoEdit::default(),
        });
        self.show_toast("Tillagd i kön".to_string());
    }

    /// Render the current edit and save it back over the original photo file,
    /// then refresh the gallery entry so the updated image is shown.
    pub fn save_current_edit(&mut self) -> Result<(), String> {
        let photo = self
            .photos
            .get(self.selected_photo)
            .ok_or("Ingen bild vald")?;
        let src_path = photo.path.clone();
        let paper_size = self.selected_product_size.clone();

        // Ensure photo directory exists
        let _ = std::fs::create_dir_all(&self.config.photo_directory);

        // Render edited image
        let out_path = render_edited_photo(
            &src_path,
            &self.current_edit,
            &paper_size,
            &self.config.photo_directory,
        )?;

        // Atomically replace original file
        let tmp_path = src_path.with_extension("tmp");
        std::fs::copy(&out_path, &tmp_path)
            .map_err(|e| format!("Kunde inte kopiera redigerad bild: {}", e))?;
        std::fs::rename(&tmp_path, &src_path).map_err(|e| format!("Kunde inte spara: {}", e))?;
        let _ = std::fs::remove_file(&out_path);

        // Reload dimensions from the saved file
        if let Ok(img) = image::open(&src_path) {
            if let Some(p) = self.photos.get_mut(self.selected_photo) {
                p.dimensions = Some((img.width(), img.height()));
            }
        }

        // Remove old texture from cache so it reloads
        let key = src_path.to_string_lossy().to_string();
        self.textures.remove(&key);

        // Reset edit state
        self.current_edit = PhotoEdit::default();

        self.show_toast("Bilden sparad".to_string());
        self.screen = AppScreen::Gallery;
        Ok(())
    }

    /// Spawn a background thread to render and save the edited image.
    /// Sets `save_in_progress = true` and stores the receiver in `save_rx`.
    pub fn start_save_edit_thread(&mut self) {
        let photo = match self.photos.get(self.selected_photo) {
            Some(p) => p.clone(),
            None => {
                self.show_toast_long("Ingen bild vald".to_string());
                return;
            }
        };
        let edit = self.current_edit.clone();
        let paper_size = self.selected_product_size.clone();
        let photo_dir = self.config.photo_directory.clone();

        self.save_in_progress = true;
        let (tx, rx) = channel();
        self.save_rx = Some(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<(), String> {
                let out_path = render_edited_photo(&photo.path, &edit, &paper_size, &photo_dir)?;
                let tmp_path = photo.path.with_extension("tmp");
                std::fs::copy(&out_path, &tmp_path)
                    .map_err(|e| format!("Kunde inte kopiera redigerad bild: {}", e))?;
                std::fs::rename(&tmp_path, &photo.path)
                    .map_err(|e| format!("Kunde inte spara: {}", e))?;
                let _ = std::fs::remove_file(&out_path);
                Ok(())
            })();
            let _ = tx.send(result);
        });
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
    pub fn queue_total_price(&self) -> f64 {
        self.queue
            .iter()
            .map(|item| {
                let price = self.config.price_for_size(&item.paper_size);
                price * item.copies as f64
            })
            .sum()
    }

    // ── Photo copy helpers for new gallery order builder ─────────────────────
    pub fn photo_copy_count(&self, photo_idx: usize, size: &str) -> u32 {
        self.photo_copies
            .get(&photo_idx)
            .and_then(|m| m.get(size))
            .copied()
            .unwrap_or(0)
    }

    pub fn set_photo_copy_count(&mut self, photo_idx: usize, size: &str, count: u32) {
        self.photo_copies
            .entry(photo_idx)
            .or_default()
            .insert(size.to_string(), count);
        if count == 0 {
            if let Some(m) = self.photo_copies.get_mut(&photo_idx) {
                m.remove(size);
                if m.is_empty() {
                    self.photo_copies.remove(&photo_idx);
                }
            }
        }
    }

    pub fn total_copies_for_photo(&self, photo_idx: usize) -> u32 {
        self.photo_copies
            .get(&photo_idx)
            .map(|m| m.values().sum())
            .unwrap_or(0)
    }

    pub fn total_order_copies(&self) -> u32 {
        self.photo_copies
            .values()
            .map(|m| m.values().sum::<u32>())
            .sum()
    }

    pub fn total_order_price(&self) -> f64 {
        self.photo_copies
            .iter()
            .map(|(photo_idx, sizes)| {
                sizes
                    .iter()
                    .map(|(size, count)| {
                        let price = self.config.price_for_size(size);
                        price * *count as f64
                    })
                    .sum::<f64>()
            })
            .sum()
    }

    pub fn add_selected_to_queue(&mut self) {
        for (&photo_idx, sizes) in &self.photo_copies {
            for (size, &count) in sizes {
                if count > 0 {
                    self.queue.push(QueueItem {
                        photo_idx,
                        paper_size: size.clone(),
                        copies: count,
                        edit: PhotoEdit::default(),
                    });
                }
            }
        }
    }

    pub fn print_queue(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        let mut dispatched = 0;
        let mut failed = 0;

        for item in &self.queue {
            let photo = match self.photos.get(item.photo_idx) {
                Some(p) => p,
                None => {
                    failed += 1;
                    continue;
                }
            };
            let printer_name = match self.config.printer_for_size(&item.paper_size) {
                Some(name) => name,
                None => {
                    failed += 1;
                    continue;
                }
            };
            let printer = match self.printers.iter_mut().find(|p| p.name() == printer_name) {
                Some(p) => p,
                None => {
                    eprintln!("[PRINT] No Printer instance found for '{}'", printer_name);
                    failed += 1;
                    continue;
                }
            };

            // Render edited version (crop, rotate, B&W) to a temp file
            // Always print original (edits are saved destructively to the source file)
            let edit_path = match render_edited_photo(
                &photo.path,
                &PhotoEdit::default(),
                &item.paper_size,
                &self.config.temp_directory,
            ) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("[PRINT] Failed to render edited photo: {}", e);
                    failed += 1;
                    continue;
                }
            };

            // Save to print history before dispatching
            let hist_result = crate::print_history::save_print(
                &self.config.temp_directory,
                &edit_path,
                &photo.file_name,
                &item.paper_size,
                item.copies,
                printer_name,
                "pending",
                None,
            );
            let hist_folder = hist_result.ok();

            let job_id = printer.queue_job(
                &edit_path,
                &item.paper_size,
                item.copies,
                self.config.fit_to_page,
            );

            if let Some(folder) = hist_folder {
                self.history_folders.push(folder.clone());
                self.history_job_map.insert(job_id, folder);
            }
            eprintln!(
                "[PRINT] Queued job {} on {} for {} ({} copies, {})",
                job_id, printer_name, photo.file_name, item.copies, item.paper_size
            );
            dispatched += 1;
        }

        self.queue.clear();
        self.queue_selected = 0;

        if dispatched > 0 {
            self.print_progress_total = dispatched;
            self.print_progress_done = 0;
            self.print_progress_failed = failed;
            self.print_progress_start = Some(Instant::now());
            self.screen = AppScreen::PrintProgress;
            if failed > 0 {
                self.show_toast(format!("{} skrivna, {} misslyckades", dispatched, failed));
            } else {
                self.show_toast(format!("{} jobb skickade till skrivaren", dispatched));
            }
        } else {
            self.show_toast_long("Inga jobb kunde skrivas ut".into());
            self.screen = AppScreen::Gallery;
        }
    }

    pub fn reset_for_new_customer(&mut self) {
        self.photos.clear();
        self.selected_photo = 0;
        self.selected_photos.clear();
        self.photo_copies.clear();
        self.current_edit = PhotoEdit::default();
        self.queue.clear();
        self.queue_selected = 0;
        self.queue_clear_confirm = false;
        self.queue_clear_timer = 0.0;
        self.print_jobs.clear();
        self.print_progress_total = 0;
        self.print_progress_done = 0;
        self.print_progress_failed = 0;
        self.print_progress_start = None;
        // NOTE: history_folders is kept across customers
        self.history_job_map.clear();
        self.selected_product.clear();
        self.selected_product_size.clear();
        self.selected_layout = None;
        self.collage_photo_indices.clear();
        self.collage_preview_path = None;
        self.mtp_photos.clear();
        self.mtp_selected.clear();
        self.phone_photos.clear();
        self.phone_selected.clear();
        self.mtp_raw_photos.clear();
        self.textures.clear();
        self.screen = AppScreen::ProductSelect;
    }

    // ========================================================================
    // Toast
    // ========================================================================
    pub fn show_toast(&mut self, msg: String) {
        self.toast = Some((msg, 2.0));
    }

    pub fn reprint_from_history(&mut self, folder: &std::path::Path) -> Result<(), String> {
        let record_path = folder.join("record.json");
        let contents = std::fs::read_to_string(&record_path).map_err(|e| e.to_string())?;
        let record: crate::print_history::PrintRecord =
            serde_json::from_str(&contents).map_err(|e| e.to_string())?;

        let ext = std::path::Path::new(&record.photo_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        let photo_path = folder.join(format!("print.{}", ext));
        if !photo_path.exists() {
            return Err("Utskriftsfilen hittades inte".to_string());
        }

        let printer_name = match self.config.printer_for_size(&record.paper_size) {
            Some(name) => name,
            None => return Err(format!("Ingen skrivare konfigurerad för {}", record.paper_size)),
        };

        let printer = match self.printers.iter_mut().find(|p| p.name() == printer_name) {
            Some(p) => p,
            None => return Err(format!("Skrivaren '{}' hittades inte", printer_name)),
        };

        let _job_id = printer.queue_job(
            &photo_path,
            &record.paper_size,
            record.copies,
            self.config.fit_to_page,
        );
        Ok(())
    }

    pub fn show_toast_long(&mut self, msg: String) {
        self.toast = Some((msg, 5.0));
    }

    // ========================================================================
    // Mobile upload
    // ========================================================================
    pub fn open_mobile_upload(&mut self) {
        self.screen = AppScreen::MobileUpload;

        // Save current WiFi connection so we can restore it later
        if self.previous_wifi_conn.is_none() {
            if let Some(conn) = get_active_wifi_connection() {
                eprintln!("[WiFi] Saving previous connection: {}", conn);
                self.previous_wifi_conn = Some(conn);
            }
        }

        // Generate fresh random credentials for this session
        let ssid = format!("Zalstudio_{:04}", fastrand::u32(0..10000));
        let pass = format!("{:08x}", fastrand::u32(0..u32::MAX));
        self.session_ssid = Some(ssid.clone());
        self.session_password = Some(pass.clone());
        // Invalidate old WiFi QR so it regenerates with new credentials
        self.wifi_qr_texture = None;

        // Stop any existing hotspot so we can reconfigure with new credentials
        if linux_is_hotspot_active() {
            let _ = linux_stop_hotspot();
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        let (tx, rx) = channel();
        let ssid = ssid.clone();
        let pass = pass.clone();
        self.hotspot_rx = Some(rx);
        std::thread::spawn(move || {
            let result = linux_start_hotspot(&ssid, &pass);
            let _ = tx.send((ssid, pass, result));
        });
    }

    pub fn refresh_server_url(&mut self) {
        let ip = get_hotspot_ip().unwrap_or_else(local_ip);
        let new_url = format!("http://{}:{}", ip, self.config.server_port);
        eprintln!("[Server URL] {}", new_url);
        self.server_url = Some(new_url.clone());
        // Force QR code to regenerate with the new URL
        self.qr_texture = None;
        let _ = self.qr_tx.send(new_url);
    }

    pub fn close_mobile_upload(&mut self) {
        self.session_ssid = None;
        self.session_password = None;
        self.wifi_qr_texture = None;
    }

    pub fn stop_hotspot(&mut self) {
        let _ = linux_stop_hotspot();

        // Restore previous WiFi connection if we saved one
        if let Some(conn) = self.previous_wifi_conn.take() {
            eprintln!("[WiFi] Restoring previous connection: {}", conn);
            std::thread::spawn(move || {
                // Give NetworkManager a moment to clean up the hotspot interface
                std::thread::sleep(std::time::Duration::from_secs(2));
                let output = std::process::Command::new("nmcli")
                    .args(["connection", "up", &conn])
                    .output();
                match output {
                    Ok(o) if o.status.success() => {
                        eprintln!("[WiFi] Reconnected to {}", conn);
                    }
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        eprintln!("[WiFi] Failed to reconnect to {}: {}", conn, stderr.trim());
                    }
                    Err(e) => {
                        eprintln!("[WiFi] nmcli error: {}", e);
                    }
                }
            });
        }
    }

    pub fn check_hotspot_result(&mut self) {
        if let Some(rx) = &self.hotspot_rx {
            if let Ok((_ssid, _pass, result)) = rx.try_recv() {
                self.hotspot_rx = None;
                match result {
                    Ok(()) => {
                        // Hotspot is up — now detect the correct interface IP and refresh QR
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        self.refresh_server_url();
                        self.show_toast_long("✅ WiFi-hotspot startad!".into());
                    }
                    Err(e) => {
                        eprintln!("[Hotspot] {}", e);
                        self.show_toast_long(format!("WiFi-hotspot: {}", e));
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

        if phone_type == PhoneType::Android {
            let (cmd_tx, res_rx) = spawn_mtp_worker();
            self.mtp_cmd_tx = Some(cmd_tx);
            self.mtp_res_rx = Some(res_rx);
            self.mtp_raw_photos.clear();
            self.mtp_photos.clear();
            self.mtp_selected.clear();
        }
    }

    pub fn poll_phone_connection(&mut self) {
        // Collect results from native MTP worker
        let mut mtp_results = Vec::new();
        if let Some(ref rx) = self.mtp_res_rx {
            while let Ok(result) = rx.try_recv() {
                mtp_results.push(result);
            }
        }
        for result in mtp_results {
            match result {
                MtpResult::Photos(photos) => {
                    self.mtp_raw_photos = photos.clone();
                    self.mtp_photos = photos
                        .iter()
                        .map(|p| crate::wired_import::PhonePhoto {
                            path: std::path::PathBuf::from(&p.filename),
                            file_name: p.filename.clone(),
                            file_size: p.size,
                        })
                        .collect();
                    self.mtp_selected = vec![false; self.mtp_photos.len()];
                    self.screen = AppScreen::WiredPhonePicker;
                }
                MtpResult::Downloaded { count } => {
                    self.import_status = format!("{} bilder importerade", count);
                    self.rescan();
                    self.enter_preview_for_imported();
                    self.show_toast(format!("{} bilder importerade", count));
                    self.mtp_cmd_tx = None;
                    self.mtp_res_rx = None;
                }
                MtpResult::Error(e) => {
                    self.show_toast_long(format!("Fel: {}", e));
                    self.screen = AppScreen::MobileMenu;
                    self.mtp_cmd_tx = None;
                    self.mtp_res_rx = None;
                }
            }
        }

        // If still connecting, try both native MTP and GVFS
        if self.screen == AppScreen::PhoneConnecting {
            // Request native MTP listing (if worker is running)
            if let Some(ref tx) = self.mtp_cmd_tx {
                let _ = tx.send(MtpCommand::ListPhotos);
            }

            // Also check GVFS mounts as parallel fallback
            let mounts = crate::wired_import::find_phone_mounts();
            if !mounts.is_empty() {
                let photos = crate::wired_import::list_phone_photos(&mounts[0]);
                if !photos.is_empty() {
                    self.phone_photos = photos;
                    self.phone_selected = vec![false; self.phone_photos.len()];
                    // Use phone_photos for the unified UI via mtp_photos
                    self.mtp_photos = self.phone_photos.clone();
                    self.mtp_selected = vec![false; self.mtp_photos.len()];
                    // GVFS found photos — stop native MTP worker
                    self.mtp_cmd_tx = None;
                    self.mtp_res_rx = None;
                    self.mtp_raw_photos.clear();
                    self.screen = AppScreen::WiredPhonePicker;
                }
            }
        }
    }

    pub fn open_phone_folder(&mut self, _folder_path: String) {
        // No-op on Linux — native MTP does not use folder selection
    }

    pub fn import_selected_phone_photos(&mut self) {
        if self.mtp_cmd_tx.is_some() && !self.mtp_raw_photos.is_empty() {
            // Native MTP download
            let selected: Vec<MtpPhoto> = self
                .mtp_raw_photos
                .iter()
                .enumerate()
                .filter(|(i, _)| self.mtp_selected.get(*i).copied().unwrap_or(false))
                .map(|(_, p)| p.clone())
                .collect();

            if !selected.is_empty() {
                if let Some(ref tx) = self.mtp_cmd_tx {
                    let _ = tx.send(MtpCommand::Download {
                        photos: selected,
                        dest_dir: self.config.temp_directory.clone(),
                    });
                    self.screen = AppScreen::Importing;
                    self.import_status = "Laddar ner bilder...".to_string();
                }
            }
        } else {
            // GVFS / USB filesystem copy
            let selected: Vec<&crate::wired_import::PhonePhoto> = self
                .phone_photos
                .iter()
                .enumerate()
                .filter(|(i, _)| self.phone_selected.get(*i).copied().unwrap_or(false))
                .map(|(_, p)| p)
                .collect();

            if !selected.is_empty() {
                self.screen = AppScreen::Importing;
                self.import_status = "Importerar valda bilder...".to_string();
                let dest = self.config.temp_directory.clone();
                let (tx, rx) = channel::<crate::usb_detect::UsbResult>();
                let selected_owned: Vec<crate::wired_import::PhonePhoto> =
                    selected.iter().map(|p| (*p).clone()).collect();
                std::thread::spawn(move || {
                    let _ = std::fs::create_dir_all(&dest);
                    // Clean out previous temp imports so the gallery stays manageable
                    eprintln!("[IMPORT-THREAD] Clearing old temp imports...");
                    let _ = crate::wired_import::clear_temp_imports(&dest);
                    eprintln!(
                        "[IMPORT-THREAD] Copying {} selected photos...",
                        selected_owned.len()
                    );
                    let mut count = 0;
                    for photo in &selected_owned {
                        let dest_path = dest.join(&photo.file_name);
                        if std::fs::copy(&photo.path, &dest_path).is_ok() {
                            count += 1;
                        }
                    }
                    eprintln!(
                        "[IMPORT-THREAD] Done, copied {}/{} files",
                        count,
                        selected_owned.len()
                    );
                    let _ = tx.send(crate::usb_detect::UsbResult::Imported(count));
                });
                self.usb_rx = Some(rx);
            }
        }
    }

    // ========================================================================
    // Google Drive
    // ========================================================================
    pub fn start_google_photos_auth(&mut self) {
        let pkce = crate::google_drive::generate_pkce(format!(
            "http://localhost:{}/oauth",
            self.config.server_port
        ));
        self.pkce_state = Some(pkce.clone());
        let auth_url = crate::google_drive::build_auth_url(
            &self.config.google_client_id,
            &pkce.redirect_uri,
            &pkce.state,
            &pkce.code_challenge,
        );
        let _ = std::process::Command::new("xdg-open")
            .arg(&auth_url)
            .spawn();
    }

    pub fn drive_thumbnail(
        &mut self,
        ctx: &Context,
        file_id: &str,
        url: &str,
    ) -> Option<&TextureHandle> {
        if !self.drive_thumbnails.contains_key(file_id) {
            if let Ok(response) = ureq::get(url).call() {
                if let Ok(bytes) = response.into_body().read_to_vec() {
                    if let Ok(image) = image::load_from_memory(&bytes) {
                        let size = 128;
                        let image =
                            image.resize_to_fill(size, size, image::imageops::FilterType::Triangle);
                        let rgba = image.to_rgba8();
                        let tex = ctx.load_texture(
                            file_id,
                            egui::ColorImage::from_rgba_unmultiplied(
                                [size as usize, size as usize],
                                &rgba.into_raw(),
                            ),
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
            let selected: Vec<String> = self
                .drive_files
                .iter()
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
            let selected: Vec<String> = self
                .photo_items
                .iter()
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

        // Update print progress — poll printer jobs
        if self.screen == AppScreen::PrintProgress {
            let mut done = 0;
            let mut failed = 0;
            let mut queued = 0;
            let mut printing = 0;
            for printer in &self.printers {
                for job in printer.jobs() {
                    match &job.status {
                        JobStatus::Done => {
                            done += 1;
                            // Update history status to done
                            if let Some(folder) = self.history_job_map.get(&job.id) {
                                let _ = crate::print_history::update_print_status(folder, "done", None);
                            }
                        }
                        JobStatus::Failed(e) => {
                            failed += 1;
                            eprintln!("[PRINT] Job {} failed: {}", job.id, e);
                            if let Some(folder) = self.history_job_map.get(&job.id) {
                                let _ = crate::print_history::update_print_status(folder, "failed", Some(e));
                            }
                        }
                        JobStatus::Queued => queued += 1,
                        JobStatus::Printing => printing += 1,
                    }
                }
            }
            self.print_progress_done = done;
            self.print_progress_failed = failed;

            let total = self.print_progress_total;
            let elapsed = self.print_progress_start.map(|t| t.elapsed().as_secs_f32()).unwrap_or(0.0);
            eprintln!("[PRINT-PROGRESS] total={}, done={}, failed={}, elapsed={:.1}s", total, done, failed, elapsed);

            // Must show progress for at least 2 seconds so user can see it
            let all_done = done + failed >= total && total > 0;
            let min_time_met = elapsed >= 2.0;
            if all_done && min_time_met {
                eprintln!("[PRINT-PROGRESS] All jobs finished! Switching to ThankYou.");
                self.screen = AppScreen::ThankYou;
                self.thank_you_timer = 8.0;
            } else {
                ctx.request_repaint_after(Duration::from_millis(200));
            }
        }

        // Update thank-you timer — reset for new customer when done
        if self.screen == AppScreen::ThankYou {
            self.thank_you_timer -= ctx.input(|i| i.unstable_dt);
            if self.thank_you_timer <= 0.0 {
                self.reset_for_new_customer();
            }
        }

        // Update queue clear timer
        if self.queue_clear_confirm {
            self.queue_clear_timer -= ctx.input(|i| i.unstable_dt);
            if self.queue_clear_timer <= 0.0 {
                self.queue_clear_confirm = false;
            }
        }

        // Update settings save confirm timer
        if self.settings_save_confirm > 0.0 {
            self.settings_save_confirm -= ctx.input(|i| i.unstable_dt);
        }

        // Check for QR code updates
        if self.qr_texture.is_none() {
            while let Ok(url) = self.qr_rx.try_recv() {
                if url.starts_with("DOWNLOADED:") {
                    let count: usize = url.trim_start_matches("DOWNLOADED:").parse().unwrap_or(0);
                    self.import_status = format!("{} bilder importerade", count);
                    self.rescan();
                    self.enter_preview_for_imported();
                    self.show_toast(format!("{} bilder importerade", count));
                } else if let Some(tex) = generate_qr_texture(ctx, &url) {
                    self.qr_texture = Some(tex);
                }
            }
        }

        // Check hotspot result
        self.check_hotspot_result();

        // Poll save-edit background worker
        if self.save_in_progress {
            if let Some(ref rx) = self.save_rx {
                match rx.try_recv() {
                    Ok(Ok(())) => {
                        self.save_in_progress = false;
                        self.save_rx = None;
                        // Reload dimensions from the saved file
                        let photo_path =
                            self.photos.get(self.selected_photo).map(|p| p.path.clone());
                        if let Some(path) = photo_path {
                            if let Ok(img) = image::open(&path) {
                                if let Some(p2) = self.photos.get_mut(self.selected_photo) {
                                    p2.dimensions = Some((img.width(), img.height()));
                                }
                            }
                            // Remove old texture from cache so it reloads
                            let key = path.to_string_lossy().to_string();
                            self.textures.remove(&key);
                        }
                        self.current_edit = PhotoEdit::default();
                        self.show_toast("Bilden sparad".to_string());
                        self.screen = AppScreen::Gallery;
                    }
                    Ok(Err(e)) => {
                        self.save_in_progress = false;
                        self.save_rx = None;
                        self.show_toast_long(format!("Sparfel: {}", e));
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.save_in_progress = false;
                        self.save_rx = None;
                        self.show_toast_long("Sparning avbröts oväntat".to_string());
                    }
                }
            }
        }

        // Poll USB background worker (used both during plug-wait and importing)
        if self.screen == AppScreen::UsbPlugWait || self.screen == AppScreen::Importing {
            if let Some(ref rx) = self.usb_rx {
                match rx.try_recv() {
                    Ok(result) => {
                        eprintln!("[USB-MAIN] Received result: {:?}", result);
                        self.handle_usb_result(result);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        eprintln!("[USB-MAIN] Worker disconnected unexpectedly");
                        self.usb_rx = None;
                        self.screen = AppScreen::SourceSelect;
                        self.show_toast_long("USB-import misslyckades".into());
                    }
                }
            }
        }

        // Poll phone connection
        if self.screen == AppScreen::PhoneConnecting {
            if self.phone_poll_last.elapsed() > Duration::from_secs(1) {
                self.phone_poll_last = Instant::now();
                self.poll_phone_connection();
            }
        }

        // Poll MTP download results during import
        if self.screen == AppScreen::Importing && self.mtp_res_rx.is_some() {
            let mut mtp_results = Vec::new();
            if let Some(ref rx) = self.mtp_res_rx {
                while let Ok(result) = rx.try_recv() {
                    mtp_results.push(result);
                }
            }
            for result in mtp_results {
                match result {
                    MtpResult::Downloaded { count } => {
                        self.import_status = format!("{} bilder importerade", count);
                        self.rescan();
                        self.enter_preview_for_imported();
                        self.show_toast(format!("{} bilder importerade", count));
                        self.mtp_cmd_tx = None;
                        self.mtp_res_rx = None;
                    }
                    MtpResult::Error(e) => {
                        self.show_toast_long(format!("Fel: {}", e));
                        self.screen = AppScreen::Gallery;
                        self.mtp_cmd_tx = None;
                        self.mtp_res_rx = None;
                    }
                    MtpResult::Photos(_) => {}
                }
            }
        }

        // Poll thumbnail background loader
        let mut thumbs_received = 0;
        if let Some(ref rx) = self.thumb_rx {
            while let Ok((key, color_image)) = rx.try_recv() {
                let tex = ctx.load_texture(&key, color_image, egui::TextureOptions::default());
                self.textures.insert(key, tex);
                thumbs_received += 1;
            }
        }
        if thumbs_received > 0 {
            ctx.request_repaint();
        }

        // Stop thumbnail worker when leaving picker to free CPU/GPU resources
        if self.screen != AppScreen::WiredPhonePicker && self.thumb_rx.is_some() {
            eprintln!("[THUMB] Stopping thumbnail worker because screen changed");
            self.thumb_rx = None;
        }

        // Keep repainting picker while thumbnails are still loading
        if self.screen == AppScreen::WiredPhonePicker {
            let photos = if self.phone_type == PhoneType::Android && cfg!(target_os = "linux") {
                &self.mtp_photos
            } else {
                &self.phone_photos
            };
            let pending = photos.iter().any(|p| {
                let key = p.path.to_string_lossy().to_string();
                !self.textures.contains_key(&key)
            });
            if pending {
                ctx.request_repaint();
            }
        }

        crate::ui::draw(ctx, self);
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Parse a paper size like "10x15" into width/height aspect ratio.
fn paper_size_aspect(size: &str) -> f32 {
    let parts: Vec<&str> = size.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse::<f32>().unwrap_or(1.0);
        let h = parts[1].parse::<f32>().unwrap_or(1.0);
        if h > 0.0 {
            return w / h;
        }
    }
    1.0
}

/// Apply rotation, grayscale, zoom-crop to an image and save to temp file.
/// Returns the path to the rendered file.
pub fn render_edited_photo(
    src_path: &Path,
    edit: &PhotoEdit,
    paper_size: &str,
    temp_dir: &Path,
) -> Result<PathBuf, String> {
    let mut img = image::open(src_path).map_err(|e| e.to_string())?;

    // Apply rotation
    img = match edit.rotation {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => img,
    };

    // Apply grayscale
    if edit.grayscale {
        img = img.grayscale();
    }

    // Calculate crop in pixel coords (after rotation)
    let (img_w, img_h) = (img.width() as f32, img.height() as f32);
    let frame_aspect = paper_size_aspect(paper_size);
    let eff_aspect = img_w / img_h.max(1.0);

    let (mut crop_w, mut crop_h) = if eff_aspect >= frame_aspect {
        (frame_aspect / eff_aspect * img_w, img_h)
    } else {
        (img_w, eff_aspect / frame_aspect * img_h)
    };

    crop_w = (crop_w / edit.zoom).max(1.0);
    crop_h = (crop_h / edit.zoom).max(1.0);

    let max_px = (img_w - crop_w).max(0.0);
    let max_py = (img_h - crop_h).max(0.0);
    let mut cx = (0.5 + edit.pan_x * 0.5 * max_px / img_w.max(1.0) - crop_w / img_w / 2.0) * img_w;
    let mut cy = (0.5 + edit.pan_y * 0.5 * max_py / img_h.max(1.0) - crop_h / img_h / 2.0) * img_h;
    cx = cx.max(0.0).min(img_w - crop_w);
    cy = cy.max(0.0).min(img_h - crop_h);

    let cropped = img.crop_imm(cx as u32, cy as u32, crop_w as u32, crop_h as u32);

    // Convert to RGBA for text drawing
    let mut final_img = cropped.to_rgba8();

    // Draw text overlay if present
    if !edit.text_overlay.is_empty() {
        if let Ok(font_data) = std::fs::read("/usr/share/fonts/TTF/DejaVuSans.ttf") {
            if let Ok(font) = ab_glyph::FontArc::try_from_vec(font_data) {
                let scale = ab_glyph::PxScale::from(edit.text_size as f32);
                let (tw, th) = imageproc::drawing::text_size(scale, &font, &edit.text_overlay);
                let x = ((final_img.width().saturating_sub(tw)) as f32 * edit.text_x) as i32;
                let y = ((final_img.height().saturating_sub(th)) as f32 * edit.text_y) as i32;
                let white = image::Rgba([255u8, 255u8, 255u8, 255u8]);
                imageproc::drawing::draw_text_mut(
                    &mut final_img,
                    white,
                    x,
                    y,
                    scale,
                    &font,
                    &edit.text_overlay,
                );
            }
        }
    }

    // JPEG does not support RGBA8 — convert back to RGB8 before saving
    let rgb_img = image::DynamicImage::ImageRgba8(final_img).to_rgb8();

    // Save to temp
    let file_name = format!(
        "edit_{}_{}.jpg",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        fastrand::u32(..)
    );
    let out_path = temp_dir.join(&file_name);
    rgb_img.save(&out_path).map_err(|e| e.to_string())?;

    Ok(out_path)
}

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

// ============================================================================
// Fast EXIF thumbnail extraction — most cameras embed a tiny JPEG thumbnail
// ============================================================================
fn find_exif_app1_offset(buf: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 10 < buf.len() {
        if buf[i] == 0xFF && buf[i + 1] == 0xE1 && &buf[i + 4..i + 10] == b"Exif\0\0" {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn extract_exif_thumbnail(path: &Path) -> Option<image::DynamicImage> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; 65536];
    let n = std::io::Read::read(&mut file, &mut buf).ok()?;
    buf.truncate(n);

    let exif = exif::Reader::new()
        .read_from_container(&mut std::io::BufReader::new(std::io::Cursor::new(&buf)))
        .ok()?;

    let offset = exif
        .get_field(exif::Tag::JPEGInterchangeFormat, exif::In::THUMBNAIL)?
        .value
        .get_uint(0)? as usize;
    let len = exif
        .get_field(exif::Tag::JPEGInterchangeFormatLength, exif::In::THUMBNAIL)?
        .value
        .get_uint(0)? as usize;

    let app1_offset = find_exif_app1_offset(&buf)?;
    let tiff_header_offset = app1_offset + 4 + 6; // marker(2) + length(2) + "Exif\0\0"(6)
    let abs_offset = tiff_header_offset + offset;

    if abs_offset + len > buf.len() {
        // Thumbnail extends past the 64KB we read — fall back to full decode
        return None;
    }

    let thumb_bytes = &buf[abs_offset..abs_offset + len];
    image::load_from_memory(thumb_bytes).ok()
}

fn thumbnail_color_image(path: &Path, target_size: u32) -> Option<egui::ColorImage> {
    // Try EXIF thumbnail first — cameras embed a tiny JPEG (~5KB) that's almost instant to read
    let img = extract_exif_thumbnail(path).or_else(|| image::open(path).ok())?;
    let thumb = img.thumbnail_exact(target_size, target_size);
    let rgba = thumb.to_rgba8();
    let pixels: Vec<egui::Color32> = rgba
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    Some(egui::ColorImage {
        size: [target_size as usize, target_size as usize],
        pixels,
    })
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
    // Trick: connect a UDP socket to a non-routable address.
    // This forces the OS to pick an interface without sending any packets.
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("10.255.255.255:1").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "0.0.0.0" && !ip.starts_with("127.") {
                    return ip;
                }
            }
        }
    }

    // Fallback: try connecting to external host to discover our IP
    if let Ok(addrs) = std::net::TcpStream::connect("8.8.8.8:80") {
        if let Ok(addr) = addrs.local_addr() {
            return addr.ip().to_string();
        }
    }

    // Last resort: hostname lookup
    if let Ok(output) = std::process::Command::new("hostname").args(["-I"]).output() {
        let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !ip.is_empty() {
            return ip.split_whitespace().next().unwrap_or(&ip).to_string();
        }
    }

    "127.0.0.1".to_string()
}

/// Escape special characters in Wi-Fi QR code SSID / password fields.
fn wifi_qr_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace(':', "\\:")
        .replace('"', "\\\"")
}

/// Try to discover the IP address of the active hotspot interface.
fn get_hotspot_ip() -> Option<String> {
    // Query NetworkManager for the IP of our hotspot connection
    if let Ok(output) = std::process::Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "IP4.ADDRESS",
            "connection",
            "show",
            "--active",
            HOTSPOT_CONN_NAME,
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // nmcli -t output: "IP4.ADDRESS[1]:10.42.0.1/24"
            // Strip the field name prefix, then the CIDR suffix
            let value = line.split(':').nth(1).unwrap_or(line);
            let ip = value.split('/').next().unwrap_or(value);
            if !ip.is_empty() {
                eprintln!("[Hotspot IP] nmcli returned: {}", ip);
                return Some(ip.to_string());
            }
        }
    }

    // Fallback: look at all active connections for any AP/hotspot mode
    if let Ok(output) = std::process::Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "NAME,DEVICE,TYPE",
            "connection",
            "show",
            "--active",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && parts[2].to_lowercase().contains("wireless") {
                let device = parts[1];
                // Get IP of this device
                if let Ok(ip_output) = std::process::Command::new("nmcli")
                    .args(["-t", "-f", "IP4.ADDRESS", "device", "show", device])
                    .output()
                {
                    let ip_stdout = String::from_utf8_lossy(&ip_output.stdout);
                    for ip_line in ip_stdout.lines() {
                        let ip_line = ip_line.trim();
                        if let Some(ip) = ip_line.split('/').next() {
                            if !ip.is_empty() && !ip.starts_with("127.") {
                                eprintln!("[Hotspot IP] device {} returned: {}", device, ip);
                                return Some(ip.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

// ============================================================================
// Linux Hotspot via NetworkManager (nmcli)
// ============================================================================
const HOTSPOT_CONN_NAME: &str = "ZalStudio-Hotspot";

fn linux_start_hotspot(ssid: &str, pass: &str) -> Result<(), String> {
    if linux_is_hotspot_active() {
        eprintln!("[Hotspot] Already active");
        return Ok(());
    }

    // Clean up leftover connections from previous runs
    for name in &[HOTSPOT_CONN_NAME, &format!("Hotspot-{}", ssid)[..]] {
        let _ = std::process::Command::new("nmcli")
            .args(["connection", "delete", name])
            .output();
    }

    // Find a WiFi device that supports AP mode
    let device = linux_find_ap_device();
    eprintln!("[Hotspot] AP-capable device: {:?}", device);

    let device = device.ok_or(
        "Ingen WiFi-adapter med AP-stöd hittades. Kontrollera att en WiFi-adapter stöder Access Point-läge."
    )?;

    // If the device is currently connected to WiFi, disconnect it first
    let was_connected = linux_device_connected(&device);
    if was_connected {
        eprintln!("[Hotspot] Disconnecting {} from current network...", device);
        let _ = std::process::Command::new("nmcli")
            .args(["device", "disconnect", &device])
            .output();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Strategy 1: nmcli device wifi hotspot (one-shot)
    let mut cmd = std::process::Command::new("nmcli");
    cmd.args([
        "device",
        "wifi",
        "hotspot",
        "ifname",
        &device,
        "con-name",
        HOTSPOT_CONN_NAME,
        "ssid",
        ssid,
        "band",
        "bg",
        "channel",
        "6",
    ]);
    if !pass.is_empty() {
        cmd.args(["password", pass]);
    }

    let output = cmd.output().map_err(|e| format!("nmcli: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("[Hotspot] nmcli hotspot stdout: {}", stdout.trim());
    if !stderr.trim().is_empty() {
        eprintln!("[Hotspot] nmcli hotspot stderr: {}", stderr.trim());
    }

    if output.status.success() {
        std::thread::sleep(std::time::Duration::from_secs(3));
        if linux_is_hotspot_active() {
            return Ok(());
        }
    }

    // Clean up if strategy 1 failed
    let _ = std::process::Command::new("nmcli")
        .args(["connection", "delete", HOTSPOT_CONN_NAME])
        .output();

    // Strategy 2: explicit connection add + up
    let mut add_cmd = std::process::Command::new("nmcli");
    add_cmd.args([
        "connection",
        "add",
        "save",
        "no",
        "type",
        "wifi",
        "ifname",
        &device,
        "con-name",
        HOTSPOT_CONN_NAME,
        "autoconnect",
        "no",
        "ssid",
        ssid,
        "mode",
        "ap",
        "802-11-wireless.band",
        "bg",
        "802-11-wireless.channel",
        "6",
        "ipv4.method",
        "shared",
    ]);
    if !pass.is_empty() {
        add_cmd.args(["wifi-sec.key-mgmt", "wpa-psk", "wifi-sec.psk", pass]);
    }

    let add_output = add_cmd.output().map_err(|e| format!("nmcli add: {}", e))?;
    eprintln!(
        "[Hotspot] add stdout: {}",
        String::from_utf8_lossy(&add_output.stdout).trim()
    );
    let add_stderr = String::from_utf8_lossy(&add_output.stderr);
    if !add_stderr.trim().is_empty() {
        eprintln!("[Hotspot] add stderr: {}", add_stderr.trim());
    }

    if !add_output.status.success() {
        return Err(format!("Kunde inte skapa hotspot: {}", add_stderr.trim()));
    }

    let up_output = std::process::Command::new("nmcli")
        .args(["connection", "up", HOTSPOT_CONN_NAME, "ifname", &device])
        .output()
        .map_err(|e| format!("nmcli up: {}", e))?;

    let up_stderr = String::from_utf8_lossy(&up_output.stderr);
    eprintln!(
        "[Hotspot] up stdout: {}",
        String::from_utf8_lossy(&up_output.stdout).trim()
    );
    if !up_stderr.trim().is_empty() {
        eprintln!("[Hotspot] up stderr: {}", up_stderr.trim());
    }

    if up_output.status.success() {
        std::thread::sleep(std::time::Duration::from_secs(2));
        return Ok(());
    }

    Err(format!("Kunde inte starta hotspot: {}", up_stderr.trim()))
}

fn linux_is_hotspot_active() -> bool {
    if let Ok(output) = std::process::Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "NAME,DEVICE,TYPE",
            "connection",
            "show",
            "--active",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            if lower.contains("hotspot") || lower.contains("zalstudio-hotspot") {
                return true;
            }
        }
    }
    false
}

/// Find a WiFi device that supports AP (Access Point) mode.
/// Checks /sys/class/net/<dev>/phy80211/name to map device -> phy,
/// then runs `iw phy<N> info` to check for AP support.
fn linux_find_ap_device() -> Option<String> {
    let devs = linux_wifi_devices();
    for dev in devs {
        if linux_device_supports_ap(&dev) {
            return Some(dev);
        }
    }
    None
}

fn linux_wifi_devices() -> Vec<String> {
    let output = std::process::Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE", "device", "status"])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 && parts[1] == "wifi" {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .collect()
}

fn linux_device_supports_ap(device: &str) -> bool {
    // Read phy name from /sys/class/net/<device>/phy80211/name
    let phy_path = format!("/sys/class/net/{}/phy80211/name", device);
    let phy_name = std::fs::read_to_string(&phy_path)
        .unwrap_or_default()
        .trim()
        .to_string();

    if phy_name.is_empty() {
        return false;
    }

    // Run `iw phyN info` and check for "AP" in supported interface modes
    let output = std::process::Command::new("iw")
        .args([&format!("{}.info", phy_name)])
        .output();

    // That's not the right iw syntax. Use: iw phy phyN info
    let output = std::process::Command::new("iw")
        .args(["phy", &phy_name, "info"])
        .output();

    let Ok(output) = output else { return false };
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Look for " * AP" in the Supported interface modes section
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed == "* AP" {
            return true;
        }
    }
    false
}

fn linux_device_connected(device: &str) -> bool {
    let output = std::process::Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,STATE", "device", "status"])
        .output();
    let Ok(output) = output else { return false };
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[0] == device {
            return parts[1] == "connected";
        }
    }
    false
}

fn linux_stop_hotspot() -> Result<(), String> {
    eprintln!("[Hotspot] Stopping Linux hotspot...");
    let _ = std::process::Command::new("nmcli")
        .args(["connection", "down", HOTSPOT_CONN_NAME])
        .output();
    let _ = std::process::Command::new("nmcli")
        .args(["connection", "delete", HOTSPOT_CONN_NAME])
        .output();
    Ok(())
}

/// Get the name of the currently active 802.11 WiFi connection (if any).
fn get_active_wifi_connection() -> Option<String> {
    let output = std::process::Command::new("nmcli")
        .args(["-t", "-f", "NAME,TYPE", "connection", "show", "--active"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[1] == "802-11-wireless" {
            return Some(parts[0].to_string());
        }
    }
    None
}

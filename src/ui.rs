use egui::{Color32, Frame, Rounding, Stroke, Vec2};

use crate::app::{AppScreen, ZalStudio};
use crate::gallery::{format_dimensions, format_file_size};
use crate::lang::l;

/// Scale factor for touch-friendly sizing on large screens.
/// Baseline is 1080 px height; clamps between 1.0 and 1.5.
pub fn touch_scale(ctx: &egui::Context) -> f32 {
    let h = ctx.screen_rect().height();
    let w = ctx.screen_rect().width();
    let min_dim = h.min(w);
    // Scale relative to 1080p reference; allow < 1.0 for small VGA screens
    (min_dim / 1080.0).clamp(0.4, 1.5)
}

pub fn apply_style(ctx: &egui::Context) {
    let s = touch_scale(ctx);
    let mut style = egui::Style::default();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = Color32::from_rgb(22, 22, 28);
    style.visuals.window_fill = Color32::from_rgb(28, 28, 36);
    style.visuals.extreme_bg_color = Color32::from_rgb(16, 16, 22);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(40, 40, 55);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(50, 50, 70);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(70, 70, 95);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(90, 90, 120);
    style.visuals.selection.bg_fill = Color32::from_rgb(0, 150, 200);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 42);
    style.spacing.item_spacing = Vec2::new(8.0 * s, 8.0 * s);
    style.spacing.button_padding = Vec2::new(16.0 * s, 12.0 * s);
    ctx.set_style(style);
}

pub fn draw(ctx: &egui::Context, app: &mut ZalStudio) {
    apply_style(ctx);

    match app.screen {
        AppScreen::ProductSelect => draw_product_select(ctx, app),
        AppScreen::SourceSelect => draw_welcome(ctx, app),
        AppScreen::UsbPlugWait => draw_usb_plug_wait(ctx, app),
        AppScreen::Gallery => draw_gallery(ctx, app),
        AppScreen::Preview => draw_preview(ctx, app),
        AppScreen::Queue => draw_queue(ctx, app),
        AppScreen::Importing => draw_importing(ctx, app),
        AppScreen::MobileUpload => draw_mobile_upload(ctx, app),
        AppScreen::MobileMenu => draw_mobile_menu(ctx, app),
        AppScreen::PhoneConnecting => draw_phone_connecting(ctx, app),
        AppScreen::PhoneFolderSelect => draw_phone_folder_select(ctx, app),
        AppScreen::WiredPhonePicker => draw_wired_phone_picker(ctx, app),
        AppScreen::GoogleDriveAuth => draw_google_drive_auth(ctx, app),
        AppScreen::GoogleDrivePicker => draw_google_drive_picker(ctx, app),
        AppScreen::GooglePhotosPicker => draw_google_photos_picker(ctx, app),
        AppScreen::PrintDone => draw_print_done(ctx, app),
        AppScreen::Payment => draw_payment(ctx, app),
    }

    draw_toast(ctx, app);
}

// =============================================================================
// PRODUCT SELECT SCREEN (Foto / Album / Collage)
// =============================================================================
fn draw_product_select(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(24.0 * s),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0 * s);

                ui.label(
                    egui::RichText::new("📷  Zalstudio Kiosk")
                        .size(32.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new(pack.product_select_title)
                        .size(20.0 * s)
                        .color(Color32::from_gray(140)),
                );

                ui.add_space(50.0 * s);

                let btn_width = ui.available_width().min(400.0 * s);

                // FOTO
                let foto_btn = egui::Button::new(
                    egui::RichText::new(format!(
                        "🖼  {}\n{}",
                        pack.product_foto, pack.product_foto_desc
                    ))
                    .size(16.0 * s)
                    .strong()
                    .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 120, 90))
                .rounding(Rounding::same(12.0 * s))
                .min_size(Vec2::new(btn_width, 90.0 * s));
                if ui.add(foto_btn).clicked() {
                    app.selected_product = "foto".to_string();
                    app.screen = AppScreen::SourceSelect;
                }
                ui.add_space(14.0 * s);

                // ALBUM 15x23
                let album_btn = egui::Button::new(
                    egui::RichText::new(format!(
                        "📔  {}\n{}",
                        pack.product_album, pack.product_album_desc
                    ))
                    .size(16.0 * s)
                    .strong()
                    .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(45, 85, 140))
                .rounding(Rounding::same(12.0 * s))
                .min_size(Vec2::new(btn_width, 90.0 * s));
                if ui.add(album_btn).clicked() {
                    app.selected_product = "album".to_string();
                    // Lock paper size to 15x23 for album product
                    if let Some(idx) = app.config.paper_sizes.iter().position(|s| s == "15x23") {
                        app.paper_size_idx = idx;
                    }
                    app.screen = AppScreen::SourceSelect;
                }
                ui.add_space(14.0 * s);

                // COLLAGE
                let collage_btn = egui::Button::new(
                    egui::RichText::new(format!(
                        "🎨  {}\n{}",
                        pack.product_collage, pack.product_collage_desc
                    ))
                    .size(16.0 * s)
                    .strong()
                    .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(140, 55, 115))
                .rounding(Rounding::same(12.0 * s))
                .min_size(Vec2::new(btn_width, 90.0 * s));
                if ui.add(collage_btn).clicked() {
                    app.selected_product = "collage".to_string();
                    app.screen = AppScreen::SourceSelect;
                }
            });
        });
}

// =============================================================================
// WELCOME SCREEN
// =============================================================================
fn draw_welcome(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(18, 18, 26))
                .inner_margin(24.0 * s),
        )
        .show(ctx, |ui| {
            let rect = ui.max_rect();

            // ── Subtle vertical gradient background ─────────────────────────
            {
                let p = ui.painter();
                let top_color = Color32::from_rgb(14, 14, 22);
                let bottom_color = Color32::from_rgb(28, 30, 44);
                let bands = 80;
                let h = rect.height();
                let band_h = h / bands as f32;
                for i in 0..bands {
                    let t = i as f32 / (bands - 1) as f32;
                    let r = (top_color.r() as f32 * (1.0 - t) + bottom_color.r() as f32 * t) as u8;
                    let g = (top_color.g() as f32 * (1.0 - t) + bottom_color.g() as f32 * t) as u8;
                    let b = (top_color.b() as f32 * (1.0 - t) + bottom_color.b() as f32 * t) as u8;
                    let band_rect = egui::Rect::from_min_max(
                        egui::pos2(rect.left(), rect.top() + band_h * i as f32),
                        egui::pos2(rect.right(), rect.top() + band_h * (i + 1) as f32),
                    );
                    p.rect_filled(band_rect, 0.0, Color32::from_rgb(r, g, b));
                }
            }

            // Top bar: back to product, product label + language toggle
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::ProductSelect;
                }
                ui.add_space(8.0 * s);
                if !app.selected_product.is_empty() {
                    let product_label = match app.selected_product.as_str() {
                        "album" => pack.product_album,
                        "collage" => pack.product_collage,
                        _ => pack.product_foto,
                    };
                    ui.label(
                        egui::RichText::new(format!("🛍 {}", product_label))
                            .size(13.0 * s)
                            .strong()
                            .color(Color32::from_rgb(80, 220, 120)),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    let lang_btn = egui::Button::new(
                        egui::RichText::new(app.lang.name())
                            .size(13.0 * s)
                            .color(Color32::from_gray(180)),
                    )
                    .fill(Color32::from_rgb(35, 35, 48))
                    .rounding(Rounding::same(6.0 * s))
                    .min_size(Vec2::new(48.0 * s, 32.0 * s));
                    if ui.add(lang_btn).clicked() {
                        app.lang = app.lang.toggle();
                    }
                });
            });

            ui.vertical_centered(|ui| {
                ui.add_space(60.0 * s);

                // Title
                ui.label(
                    egui::RichText::new("📷  Zalstudio Kiosk")
                        .size(38.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(10.0 * s);

                // Subtitle
                ui.label(
                    egui::RichText::new(pack.source_select_subtitle)
                        .size(17.0 * s)
                        .color(Color32::from_gray(140)),
                );

                ui.add_space(60.0 * s);

                let btn_width = ui.available_width().min(480.0 * s);

                // USB button
                let usb_btn = egui::Button::new(
                    egui::RichText::new(format!("💾   {}", pack.source_usb_camera))
                        .size(18.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(45, 85, 140))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 68.0 * s));
                if ui.add(usb_btn).clicked() {
                    app.import_usb();
                }
                ui.add_space(14.0 * s);

                // Mobile button
                let mobile_btn = egui::Button::new(
                    egui::RichText::new(format!("📱   {}", pack.source_mobile))
                        .size(18.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(140, 55, 115))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 68.0 * s));
                if ui.add(mobile_btn).clicked() {
                    app.open_mobile_upload();
                }
            });
        });
}

// =============================================================================
// USB PLUG WAIT SCREEN
// =============================================================================
fn draw_usb_plug_wait(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(24.0 * s),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::SourceSelect;
                    app.usb_rx = None;
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(80.0 * s);

                ui.label(
                    egui::RichText::new("💾")
                        .size(64.0 * s)
                        .color(Color32::from_rgb(0, 200, 255)),
                );
                ui.add_space(16.0 * s);

                ui.label(
                    egui::RichText::new(pack.usb_plug_title)
                        .size(28.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(12.0 * s);

                ui.label(
                    egui::RichText::new(pack.usb_plug_subtitle)
                        .size(16.0 * s)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new(pack.usb_plug_hint)
                        .size(13.0 * s)
                        .color(Color32::from_gray(120)),
                );

                ui.add_space(40.0 * s);

                // Animated spinner
                let time = ui.ctx().input(|i| i.time);
                let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                ui.label(
                    egui::RichText::new(spinner)
                        .size(40.0 * s)
                        .color(Color32::from_rgb(0, 200, 255)),
                );

                ui.add_space(16.0 * s);
                ui.label(
                    egui::RichText::new(pack.usb_plug_searching)
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::from_rgb(0, 200, 255)),
                );
            });
        });
}

fn big_button(
    ui: &mut egui::Ui,
    label: &str,
    color: Color32,
    width: f32,
    s: f32,
    mut on_click: impl FnMut(),
) {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .size(16.0 * s)
            .strong()
            .color(Color32::WHITE),
    )
    .fill(color)
    .rounding(Rounding::same(10.0 * s))
    .min_size(Vec2::new(width, 52.0 * s));
    if ui.add(btn).clicked() {
        on_click();
    }
    ui.add_space(8.0 * s);
}

// =============================================================================
// GALLERY SCREEN
// =============================================================================
fn draw_gallery(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(12.0 * s),
        )
        .show(ctx, |ui| {
            // Top bar
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.gallery)
                            .size(18.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let queue_count = app.queue.len();
                    let cart_label = if queue_count > 0 {
                        format!("🛒 {}", queue_count)
                    } else {
                        "🛒".to_string()
                    };
                    if nav_button(ui, &cart_label, s).clicked() {
                        app.screen = AppScreen::Queue;
                    }
                    ui.add_space(4.0 * s);
                    if nav_button(ui, "🔄", s).clicked() {
                        app.rescan();
                    }
                });
            });

            ui.add_space(8.0 * s);

            if app.photos.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(pack.no_photos)
                            .size(14.0 * s)
                            .color(Color32::from_gray(120)),
                    );
                });
                return;
            }

            let photos: Vec<crate::gallery::Photo> = app.photos.clone();
            let selected = app.selected_photo;
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (i, photo) in photos.iter().enumerate() {
                        let in_queue = app.queue.iter().any(|q| q.photo_idx == i);
                        let is_selected = i == selected;

                        let bg = if is_selected {
                            Color32::from_rgb(0, 110, 160)
                        } else {
                            Color32::from_rgb(30, 30, 45)
                        };

                        let frame = Frame::none()
                            .fill(bg)
                            .rounding(Rounding::same(8.0 * s))
                            .inner_margin(10.0 * s)
                            .stroke(Stroke::new(
                                1.5 * s,
                                if is_selected {
                                    Color32::from_rgb(0, 200, 255)
                                } else {
                                    Color32::TRANSPARENT
                                },
                            ));

                        let response = frame
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Thumbnail
                                    let thumb_size = 48.0 * s;
                                    if let Some(texture) = app.texture_for(ctx, &photo.path) {
                                        let tex_size = texture.size_vec2();
                                        let aspect = tex_size.x / tex_size.y;
                                        let (tw, th) = if aspect > 1.0 {
                                            (thumb_size, thumb_size / aspect)
                                        } else {
                                            (thumb_size * aspect, thumb_size)
                                        };
                                        ui.add(
                                            egui::Image::new((texture.id(), Vec2::new(tw, th)))
                                                .rounding(Rounding::same(4.0 * s)),
                                        );
                                    } else {
                                        ui.add_space(thumb_size);
                                    }
                                    ui.add_space(8.0 * s);

                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(&photo.file_name)
                                                .size(14.0 * s)
                                                .strong()
                                                .color(Color32::WHITE),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} · {}",
                                                format_dimensions(photo.dimensions),
                                                format_file_size(photo.file_size)
                                            ))
                                            .size(11.0 * s)
                                            .color(Color32::from_gray(150)),
                                        );
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if in_queue {
                                                ui.label(
                                                    egui::RichText::new("✓")
                                                        .size(18.0 * s)
                                                        .color(Color32::from_rgb(0, 255, 150)),
                                                );
                                            }
                                        },
                                    );
                                });
                            })
                            .response;

                        if response.clicked() {
                            app.selected_photo = i;
                            app.current_edit = crate::app::PhotoEdit::default();
                            app.screen = AppScreen::Preview;
                        }
                    }
                });
        });
}

fn nav_button(ui: &mut egui::Ui, label: &str, s: f32) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).size(16.0 * s).color(Color32::WHITE))
            .fill(Color32::from_rgb(40, 40, 58))
            .rounding(Rounding::same(8.0 * s))
            .min_size(Vec2::new(44.0 * s, 44.0 * s)),
    )
}

// =============================================================================
// EDIT / PREVIEW HELPERS
// =============================================================================

/// Parse a paper size like "10x15" or "15x23" into width/height aspect ratio.
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

/// Calculate the four UV corners for a rotated, zoomed, panned crop.
fn calculate_crop_uvs(
    img_w: f32,
    img_h: f32,
    frame_aspect: f32,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    rotation: u32,
) -> [egui::Pos2; 4] {
    // Effective aspect after rotation
    let eff_aspect = match rotation {
        90 | 270 => img_h / img_w.max(1.0),
        _ => img_w / img_h.max(1.0),
    };

    // Default crop: cover the frame (crop sides or top/bottom to match aspect)
    let (mut crop_w, mut crop_h) = if eff_aspect >= frame_aspect {
        (frame_aspect / eff_aspect, 1.0)
    } else {
        (1.0, eff_aspect / frame_aspect)
    };

    // Apply zoom
    crop_w = (crop_w / zoom).max(0.01).min(1.0);
    crop_h = (crop_h / zoom).max(0.01).min(1.0);

    // Centered pan range
    let max_px = (1.0 - crop_w).max(0.0);
    let max_py = (1.0 - crop_h).max(0.0);
    let mut cx = 0.5 + pan_x * 0.5 * max_px - crop_w * 0.5;
    let mut cy = 0.5 + pan_y * 0.5 * max_py - crop_h * 0.5;
    cx = cx.max(0.0).min(1.0 - crop_w);
    cy = cy.max(0.0).min(1.0 - crop_h);

    // Corners in rotated normalized space (top-left origin)
    let tl = (cx, cy);
    let tr = (cx + crop_w, cy);
    let br = (cx + crop_w, cy + crop_h);
    let bl = (cx, cy + crop_h);

    // Map rotated-space corners to original texture UVs
    let map = |(rx, ry): (f32, f32)| -> egui::Pos2 {
        let (ox, oy) = match rotation {
            90 => (1.0 - ry, rx),
            180 => (1.0 - rx, 1.0 - ry),
            270 => (ry, 1.0 - rx),
            _ => (rx, ry),
        };
        egui::pos2(ox.clamp(0.0, 1.0), oy.clamp(0.0, 1.0))
    };

    // Return in order: bottom-left, bottom-right, top-right, top-left
    // (matches the mesh vertex order used by egui quads)
    [map(bl), map(br), map(tr), map(tl)]
}

/// Draw an image with rotation / zoom / pan applied via custom mesh.
fn draw_editable_image(
    painter: &egui::Painter,
    texture_id: egui::TextureId,
    dest_rect: egui::Rect,
    uvs: [egui::Pos2; 4],
    tint: Color32,
) {
    let mut mesh = egui::Mesh::with_texture(texture_id);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);

    let corners = [
        dest_rect.left_bottom(),
        dest_rect.right_bottom(),
        dest_rect.right_top(),
        dest_rect.left_top(),
    ];

    for i in 0..4 {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: corners[i],
            uv: uvs[i],
            color: tint,
        });
    }

    painter.add(mesh);
}

// =============================================================================
// PREVIEW SCREEN
// =============================================================================
fn draw_preview(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(8.0 * s),
        )
        .show(ctx, |ui| {
            // ── Top bar ─────────────────────────────────────────────────
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::Gallery;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.preview)
                            .size(16.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let queue_count = app.queue.len();
                    let cart_label = if queue_count > 0 {
                        format!("🛒 {}", queue_count)
                    } else {
                        "🛒".to_string()
                    };
                    if nav_button(ui, &cart_label, s).clicked() {
                        app.screen = AppScreen::Queue;
                    }
                });
            });

            ui.add_space(4.0 * s);

            // ── Image area ──────────────────────────────────────────────
            let paper_size = app
                .config
                .paper_sizes
                .get(app.paper_size_idx)
                .cloned()
                .unwrap_or_else(|| "10x15".to_string());
            let frame_aspect = paper_size_aspect(&paper_size);

            // Extract texture info in a limited scope so mutable borrow ends
            let tex_info = app.selected_texture(ctx).map(|t| (t.id(), t.size_vec2()));

            if let Some((tex_id, tex_size)) = tex_info {
                // Allocate image area: fit within available width, use paper aspect ratio
                let available = ui.available_size();
                let max_img_w = available.x;
                let max_img_h = available.y * 0.66; // reserve ~34% for controls
                let img_w = max_img_w.min(max_img_h * frame_aspect);
                let img_h = img_w / frame_aspect;

                let (img_rect, img_response) =
                    ui.allocate_exact_size(Vec2::new(img_w, img_h), egui::Sense::drag());

                // Handle drag-to-pan
                if img_response.dragged() {
                    let delta = img_response.drag_delta();
                    let sensitivity = 2.0 / app.current_edit.zoom.max(1.0);
                    app.current_edit.pan_x -= delta.x / (img_w.max(1.0) * 0.5) * sensitivity;
                    app.current_edit.pan_y -= delta.y / (img_h.max(1.0) * 0.5) * sensitivity;
                    app.current_edit.pan_x = app.current_edit.pan_x.clamp(-1.0, 1.0);
                    app.current_edit.pan_y = app.current_edit.pan_y.clamp(-1.0, 1.0);
                }

                let uvs = calculate_crop_uvs(
                    tex_size.x,
                    tex_size.y,
                    frame_aspect,
                    app.current_edit.zoom,
                    app.current_edit.pan_x,
                    app.current_edit.pan_y,
                    app.current_edit.rotation,
                );

                let painter = ui.painter();
                // Dark background behind the image
                painter.rect_filled(img_rect, 4.0 * s, Color32::from_rgb(8, 8, 14));
                // The image
                draw_editable_image(painter, tex_id, img_rect, uvs, Color32::WHITE);
                // Print frame border
                painter.rect_stroke(
                    img_rect,
                    4.0 * s,
                    Stroke::new(2.5 * s, Color32::from_rgb(0, 200, 255)),
                );
                // Corner marks for "crop" feel
                let corner_len = 12.0 * s;
                let corners = [
                    (
                        img_rect.left_top(),
                        Vec2::new(1.0, 0.0),
                        Vec2::new(0.0, 1.0),
                    ),
                    (
                        img_rect.right_top(),
                        Vec2::new(-1.0, 0.0),
                        Vec2::new(0.0, 1.0),
                    ),
                    (
                        img_rect.right_bottom(),
                        Vec2::new(-1.0, 0.0),
                        Vec2::new(0.0, -1.0),
                    ),
                    (
                        img_rect.left_bottom(),
                        Vec2::new(1.0, 0.0),
                        Vec2::new(0.0, -1.0),
                    ),
                ];
                for (pos, dx, dy) in corners {
                    let a = pos + dx * corner_len;
                    let b = pos + dy * corner_len;
                    painter.line_segment(
                        [pos, a],
                        Stroke::new(2.0 * s, Color32::from_rgb(0, 200, 255)),
                    );
                    painter.line_segment(
                        [pos, b],
                        Stroke::new(2.0 * s, Color32::from_rgb(0, 200, 255)),
                    );
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("—")
                            .size(20.0 * s)
                            .color(Color32::from_gray(100)),
                    );
                });
            }

            ui.add_space(6.0 * s);

            // ── Controls ────────────────────────────────────────────────
            ui.vertical_centered(|ui| {
                // Row 1: Edit tools
                ui.horizontal(|ui| {
                    // Zoom out
                    let zoom_out = egui::Button::new(
                        egui::RichText::new("−")
                            .size(16.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(45, 45, 65))
                    .rounding(Rounding::same(6.0 * s))
                    .min_size(Vec2::new(44.0 * s, 36.0 * s));
                    if ui.add(zoom_out).clicked() && app.current_edit.zoom > 1.0 {
                        app.current_edit.zoom = (app.current_edit.zoom / 1.2).max(1.0);
                    }
                    ui.label(
                        egui::RichText::new(format!("{:.1}x", app.current_edit.zoom))
                            .size(13.0 * s)
                            .color(Color32::from_gray(180)),
                    );
                    // Zoom in
                    let zoom_in = egui::Button::new(
                        egui::RichText::new("+")
                            .size(16.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(45, 45, 65))
                    .rounding(Rounding::same(6.0 * s))
                    .min_size(Vec2::new(44.0 * s, 36.0 * s));
                    if ui.add(zoom_in).clicked() && app.current_edit.zoom < 5.0 {
                        app.current_edit.zoom = (app.current_edit.zoom * 1.2).min(5.0);
                    }

                    ui.add_space(10.0 * s);

                    // Rotate left
                    let rot_left = egui::Button::new(
                        egui::RichText::new("↺")
                            .size(14.0 * s)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(45, 45, 65))
                    .rounding(Rounding::same(6.0 * s))
                    .min_size(Vec2::new(44.0 * s, 36.0 * s));
                    if ui.add(rot_left).clicked() {
                        app.current_edit.rotation = (app.current_edit.rotation + 270) % 360;
                    }
                    // Rotate right
                    let rot_right = egui::Button::new(
                        egui::RichText::new("↻")
                            .size(14.0 * s)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(45, 45, 65))
                    .rounding(Rounding::same(6.0 * s))
                    .min_size(Vec2::new(44.0 * s, 36.0 * s));
                    if ui.add(rot_right).clicked() {
                        app.current_edit.rotation = (app.current_edit.rotation + 90) % 360;
                    }

                    ui.add_space(10.0 * s);

                    // B&W toggle
                    let bw_text = if app.current_edit.grayscale {
                        "B&W ✓"
                    } else {
                        "B&W"
                    };
                    let bw_btn = egui::Button::new(
                        egui::RichText::new(bw_text)
                            .size(12.0 * s)
                            .color(Color32::WHITE),
                    )
                    .fill(if app.current_edit.grayscale {
                        Color32::from_rgb(80, 80, 80)
                    } else {
                        Color32::from_rgb(45, 45, 65)
                    })
                    .rounding(Rounding::same(6.0 * s))
                    .min_size(Vec2::new(56.0 * s, 36.0 * s));
                    if ui.add(bw_btn).clicked() {
                        app.current_edit.grayscale = !app.current_edit.grayscale;
                    }
                });

                ui.add_space(4.0 * s);

                // Row 2: Paper size
                let printer_label = app
                    .printer_for_current_size()
                    .unwrap_or(pack.no_printer_for_size);
                ui.label(
                    egui::RichText::new(format!(
                        "{}: {}  ·  {}: {}",
                        pack.paper_size,
                        app.config.paper_sizes[app.paper_size_idx],
                        pack.printer_for_size,
                        printer_label
                    ))
                    .size(11.0 * s)
                    .color(Color32::from_gray(160)),
                );
                ui.add_space(2.0 * s);
                ui.horizontal(|ui| {
                    for (i, size) in app.config.paper_sizes.iter().enumerate() {
                        let is_selected = i == app.paper_size_idx;
                        let btn = egui::Button::new(
                            egui::RichText::new(size).size(13.0 * s).strong().color(
                                if is_selected {
                                    Color32::WHITE
                                } else {
                                    Color32::from_gray(200)
                                },
                            ),
                        )
                        .fill(if is_selected {
                            Color32::from_rgb(0, 130, 190)
                        } else {
                            Color32::from_rgb(40, 40, 55)
                        })
                        .rounding(Rounding::same(6.0 * s))
                        .min_size(Vec2::new(72.0 * s, 34.0 * s));
                        if ui.add(btn).clicked() {
                            app.paper_size_idx = i;
                        }
                        ui.add_space(4.0 * s);
                    }
                });

                ui.add_space(4.0 * s);

                // Row 3: Copies
                ui.horizontal(|ui| {
                    let minus = egui::Button::new(egui::RichText::new("−").size(16.0 * s).strong())
                        .fill(Color32::from_rgb(45, 45, 65))
                        .rounding(Rounding::same(6.0 * s))
                        .min_size(Vec2::new(40.0 * s, 34.0 * s));
                    if ui.add(minus).clicked() && app.copies > 1 {
                        app.copies -= 1;
                    }
                    ui.add_space(10.0 * s);
                    ui.label(
                        egui::RichText::new(format!("{} {}", app.copies, pack.copies))
                            .size(14.0 * s)
                            .strong()
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    ui.add_space(10.0 * s);
                    let plus = egui::Button::new(egui::RichText::new("+").size(16.0 * s).strong())
                        .fill(Color32::from_rgb(45, 45, 65))
                        .rounding(Rounding::same(6.0 * s))
                        .min_size(Vec2::new(40.0 * s, 34.0 * s));
                    if ui.add(plus).clicked() && app.copies < 99 {
                        app.copies += 1;
                    }
                });

                ui.add_space(6.0 * s);

                // Row 4: Actions
                let btn_width = ui.available_width().min(300.0 * s);
                let add_btn = egui::Button::new(
                    egui::RichText::new(format!("➕ {}", pack.add_to_queue))
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 140, 100))
                .rounding(Rounding::same(8.0 * s))
                .min_size(Vec2::new(btn_width, 42.0 * s));
                if ui.add(add_btn).clicked() {
                    app.add_to_queue();
                }

                ui.add_space(3.0 * s);

                let queue_count = app.queue.len();
                let cart_btn = egui::Button::new(
                    egui::RichText::new(format!(
                        "🛒 {} ({})",
                        pack.print_queue_bottom, queue_count
                    ))
                    .size(14.0 * s)
                    .strong()
                    .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 120, 180))
                .rounding(Rounding::same(8.0 * s))
                .min_size(Vec2::new(btn_width, 42.0 * s));
                if ui.add(cart_btn).clicked() {
                    app.screen = AppScreen::Queue;
                }
            });
        });
}

// =============================================================================
// QUEUE / CART SCREEN
// =============================================================================
fn draw_queue(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(12.0 * s),
        )
        .show(ctx, |ui| {
            // Top bar
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::Gallery;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.print_queue_bottom)
                            .size(18.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if app.queue_clear_confirm {
                        format!("⚠️ {}", pack.confirm_clear_queue)
                    } else {
                        "🗑".to_string()
                    };
                    if nav_button(ui, &label, s).clicked() {
                        if app.queue_clear_confirm {
                            app.queue.clear();
                            app.queue_selected = 0;
                            app.queue_clear_confirm = false;
                        } else {
                            app.queue_clear_confirm = true;
                            app.queue_clear_timer = 3.0;
                        }
                    }
                });
            });

            ui.add_space(8.0 * s);

            if app.queue.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(pack.queue_empty)
                            .size(14.0 * s)
                            .color(Color32::from_gray(120)),
                    );
                });
            } else {
                let queue_items: Vec<(usize, crate::app::QueueItem)> =
                    app.queue.iter().cloned().enumerate().collect();
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for (i, item) in queue_items.iter().cloned() {
                            let photo_name = app
                                .photos
                                .get(item.photo_idx)
                                .map(|p| p.file_name.clone())
                                .unwrap_or_default();
                            let copy_word = crate::lang::copies_word(app.lang, item.copies);

                            let frame = Frame::none()
                                .fill(Color32::from_rgb(30, 30, 45))
                                .rounding(Rounding::same(8.0 * s))
                                .inner_margin(10.0 * s);

                            let response = frame
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Thumbnail
                                        let thumb_size = 40.0;
                                        if let Some(path) =
                                            app.photos.get(item.photo_idx).map(|p| p.path.clone())
                                        {
                                            if let Some(texture) = app.texture_for(ctx, &path) {
                                                let tex_size = texture.size_vec2();
                                                let aspect = tex_size.x / tex_size.y;
                                                let (tw, th) = if aspect > 1.0 {
                                                    (thumb_size, thumb_size / aspect)
                                                } else {
                                                    (thumb_size * aspect, thumb_size)
                                                };
                                                ui.add(
                                                    egui::Image::new((
                                                        texture.id(),
                                                        Vec2::new(tw, th),
                                                    ))
                                                    .rounding(Rounding::same(4.0 * s)),
                                                );
                                            } else {
                                                ui.add_space(thumb_size);
                                            }
                                        }
                                        ui.add_space(6.0 * s);

                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(&photo_name)
                                                    .size(13.0 * s)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            );
                                            let price = app.config.price_for_size(&item.paper_size);
                                            let line = if price > 0.0 {
                                                format!(
                                                    "{} · {} {} · {:.0} kr",
                                                    item.paper_size,
                                                    item.copies,
                                                    copy_word,
                                                    price * item.copies as f64
                                                )
                                            } else {
                                                format!(
                                                    "{} · {} {}",
                                                    item.paper_size, item.copies, copy_word
                                                )
                                            };
                                            ui.label(
                                                egui::RichText::new(line)
                                                    .size(11.0 * s)
                                                    .color(Color32::from_gray(150)),
                                            );
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let del_btn = ui.add(
                                                    egui::Button::new(
                                                        egui::RichText::new("×")
                                                            .size(18.0 * s)
                                                            .color(Color32::from_rgb(255, 80, 80)),
                                                    )
                                                    .frame(false),
                                                );
                                                if del_btn.clicked() {
                                                    app.remove_from_queue(i);
                                                }
                                            },
                                        );
                                    });
                                })
                                .response;

                            if response.clicked() {
                                app.queue_selected = i;
                            }
                        }
                    });

                ui.add_space(10.0 * s);

                // Total price + action buttons
                let count = app.queue.len();
                let total = app.queue_total_price();
                let btn_width = ui.available_width().min(320.0);

                ui.vertical_centered(|ui| {
                    if total > 0.0 {
                        ui.label(
                            egui::RichText::new(format!("{}: {:.0} kr", pack.total_label, total))
                                .size(18.0 * s)
                                .strong()
                                .color(Color32::from_rgb(80, 220, 120)),
                        );
                        ui.add_space(8.0 * s);
                    }

                    let pay_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "💳 {} ({}) — {:.0} kr",
                            pack.payment_title, count, total
                        ))
                        .size(16.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 140, 100))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 52.0 * s));
                    if ui.add(pay_btn).clicked() {
                        app.screen = AppScreen::Payment;
                    }

                    ui.add_space(8.0 * s);

                    let print_btn = egui::Button::new(
                        egui::RichText::new(format!("🖨 {} ({})", pack.print_queue, count))
                            .size(14.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 120, 180))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 44.0 * s));
                    if ui.add(print_btn).clicked() {
                        app.print_queue(); // handles screen transition internally
                    }
                });
            }
        });
}

// =============================================================================
// IMPORTING SCREEN
// =============================================================================
fn draw_importing(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    // Keep spinner animating
    ctx.request_repaint();
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(20.0 * s),
        )
        .show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    let time = ui.ctx().input(|i| i.time);
                    let spinner = ["◆", "▲", "◆", "▼"][(time * 3.0) as usize % 4];
                    ui.label(
                        egui::RichText::new(spinner)
                            .size(56.0 * s)
                            .color(Color32::from_rgb(0, 150, 220)),
                    );
                    ui.add_space(20.0 * s);
                    ui.label(
                        egui::RichText::new(pack.importing)
                            .size(24.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(10.0 * s);
                    ui.label(
                        egui::RichText::new(&app.import_status)
                            .size(14.0 * s)
                            .color(Color32::from_gray(160)),
                    );
                });
            });
        });
}

// =============================================================================
// MOBILE MENU SCREEN — Cable fallback (Android / iPhone)
// =============================================================================
fn draw_mobile_menu(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(16.0 * s),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(20.0 * s);
                ui.label(
                    egui::RichText::new("🔌")
                        .size(40.0 * s)
                        .color(Color32::from_rgb(0, 200, 255)),
                );
                ui.label(
                    egui::RichText::new("Anslut med kabel")
                        .size(20.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    egui::RichText::new("Om WiFi inte fungerar, koppla in telefonen med USB")
                        .size(12.0 * s)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(24.0 * s);

                let btn_width = ui.available_width().min(320.0);

                big_button(
                    ui,
                    pack.mobile_android,
                    Color32::from_rgb(50, 180, 80),
                    btn_width,
                    s,
                    || {
                        app.start_phone_flow(crate::app::PhoneType::Android);
                    },
                );
                big_button(
                    ui,
                    pack.mobile_iphone,
                    Color32::from_rgb(200, 60, 60),
                    btn_width,
                    s,
                    || {
                        app.start_phone_flow(crate::app::PhoneType::IPhone);
                    },
                );

                ui.add_space(24.0 * s);

                // Back to QR/WiFi
                let back_btn = egui::Button::new(
                    egui::RichText::new(pack.wifi_qr_upload)
                        .size(13.0 * s)
                        .color(Color32::from_gray(200)),
                )
                .fill(Color32::from_rgb(35, 35, 50))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 44.0 * s));
                if ui.add(back_btn).clicked() {
                    app.open_mobile_upload();
                }
            });
        });
}

// =============================================================================
// PHONE CONNECTING SCREEN
// =============================================================================
fn draw_phone_connecting(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(16.0 * s))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = if app.picker_source == crate::app::PickerSource::Usb {
                        AppScreen::SourceSelect
                    } else {
                        AppScreen::MobileMenu
                    };
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(20.0 * s);
                ui.label(
                    egui::RichText::new("📱")
                        .size(48.0 * s)
                        .color(Color32::from_rgb(0, 200, 255)),
                );
                ui.label(
                    egui::RichText::new(pack.connect_phone_title)
                        .size(22.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(16.0 * s);

                let hint = match app.phone_type {
                    crate::app::PhoneType::Android => pack.connect_phone_android,
                    crate::app::PhoneType::IPhone => pack.connect_phone_iphone,
                };
                ui.label(
                    egui::RichText::new(hint)
                        .size(14.0 * s)
                        .color(Color32::from_gray(180)),
                );
                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new(pack.connect_phone_hint)
                        .size(12.0 * s)
                        .color(Color32::from_gray(140)),
                );

                ui.add_space(24.0 * s);

                // Animated spinner
                let time = ui.ctx().input(|i| i.time);
                let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                ui.label(
                    egui::RichText::new(spinner)
                        .size(32.0 * s)
                        .color(Color32::from_rgb(0, 200, 255)),
                );
                ui.add_space(8.0 * s);

                let elapsed = app.phone_connect_start.map(|t| t.elapsed().as_secs()).unwrap_or(0);

                // Show searching status with elapsed time — we switch screens immediately
                // when results arrive, so this is the only message the user sees while waiting.
                if elapsed > 10 {
                    ui.label(
                        egui::RichText::new(format!("{} ({}s)", pack.connect_phone_scanning, elapsed))
                            .size(14.0 * s)
                            .strong()
                            .color(Color32::from_rgb(255, 200, 0)),
                    );
                    ui.add_space(4.0 * s);
                    ui.label(
                        egui::RichText::new("Detta kan ta upp till en minut...")
                            .size(11.0 * s)
                            .color(Color32::from_gray(140)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!("{} ({}s)", pack.connect_phone_searching, elapsed))
                            .size(14.0 * s)
                            .strong()
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                }

                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new("Anslut telefonen och tryck Tillåt på skärmen. Se till att telefonen är i filöverföringsläge.")
                        .size(11.0 * s)
                        .color(Color32::from_gray(120)),
                );

                ui.add_space(16.0 * s);

                let btn_width = ui.available_width().min(280.0 * s);
                let search_btn = egui::Button::new(
                    egui::RichText::new(pack.connect_phone_search)
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 130, 190))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 44.0 * s));
                if ui.add(search_btn).clicked() {
                    app.phone_poll_last = std::time::Instant::now();
                    app.phone_connect_start = Some(std::time::Instant::now());
                    app.poll_phone_connection();
                }
            });
        });
}

// =============================================================================
// MOBILE UPLOAD SCREEN — WiFi QR + Upload QR side by side
// =============================================================================
fn draw_mobile_upload(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(16.0 * s),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if nav_button(ui, app.lang.name(), s).clicked() {
                        app.lang = app.lang.toggle();
                    }
                });
            });

            ui.vertical_centered(|ui| {
                ui.add_space(6.0 * s);
                ui.label(
                    egui::RichText::new(pack.mobile_upload_title)
                        .size(22.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    egui::RichText::new(pack.mobile_upload_hint)
                        .size(12.0 * s)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(8.0 * s);

                // ── WiFi info box ─────────────────────────────────────────────
                egui::Frame::none()
                    .fill(Color32::from_rgb(25, 25, 40))
                    .rounding(Rounding::same(12.0 * s))
                    .inner_margin(10.0 * s)
                    .show(ui, |ui| {
                        ui.set_max_width(340.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "📶  {}  ·  🔑  {}",
                                    app.config.wifi_ssid, app.config.wifi_password
                                ))
                                .size(14.0 * s)
                                .strong()
                                .color(Color32::from_rgb(255, 220, 100)),
                            );
                        });
                    });

                ui.add_space(6.0 * s);

                // ── Helper text for kiosk operators ───────────────────────────
                ui.label(
                    egui::RichText::new(if app.lang == crate::lang::Language::Swedish {
                        "💡  Om WiFi inte syns: Starta Mobile Hotspot i Windows-inställningarna"
                    } else {
                        "💡  If WiFi is not visible: Turn on Mobile Hotspot in Windows Settings"
                    })
                    .size(11.0 * s)
                    .color(Color32::from_gray(140)),
                );

                ui.add_space(10.0 * s);

                // ── Two QR codes side by side (centered) ──────────────────────
                let qr_size = 140.0 * s;
                let gap = 32.0 * s;
                let row_width = qr_size * 2.0 + gap;

                ui.horizontal(|ui| {
                    let left_pad = (ui.available_width() - row_width).max(0.0) / 2.0;
                    ui.add_space(left_pad);

                    // Left: WiFi connect QR
                    ui.vertical(|ui| {
                        ui.set_max_width(qr_size);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(pack.wifi_step_connect)
                                    .size(12.0 * s)
                                    .strong()
                                    .color(Color32::from_rgb(0, 200, 255)),
                            );
                            ui.add_space(4.0 * s);
                            if let Some(texture) = app.wifi_qr_texture(ctx) {
                                ui.image((texture.id(), Vec2::new(qr_size, qr_size)));
                            } else {
                                ui.label(
                                    egui::RichText::new("…")
                                        .size(12.0 * s)
                                        .color(Color32::from_gray(120)),
                                );
                            }
                        });
                    });

                    ui.add_space(gap);

                    // Right: Upload QR
                    ui.vertical(|ui| {
                        ui.set_max_width(qr_size);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(pack.wifi_step_upload)
                                    .size(12.0 * s)
                                    .strong()
                                    .color(Color32::from_rgb(0, 200, 255)),
                            );
                            ui.add_space(4.0 * s);
                            if let Some(texture) = app.qr_texture(ctx) {
                                ui.image((texture.id(), Vec2::new(qr_size, qr_size)));
                            } else {
                                ui.label(
                                    egui::RichText::new("…")
                                        .size(12.0 * s)
                                        .color(Color32::from_gray(120)),
                                );
                            }
                        });
                    });
                });

                ui.add_space(4.0 * s);
                if let Some(url) = &app.server_url {
                    ui.label(
                        egui::RichText::new(format!("{}: {}", pack.mobile_upload_url, url))
                            .size(10.0 * s)
                            .color(Color32::from_gray(140)),
                    );
                }

                ui.add_space(8.0 * s);

                let btn_width = ui.available_width().min(280.0);
                let refresh_btn = egui::Button::new(
                    egui::RichText::new(pack.mobile_upload_refresh)
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 130, 190))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 44.0 * s));
                if ui.add(refresh_btn).clicked() {
                    app.rescan();
                }

                ui.add_space(6.0 * s);

                let done_btn = egui::Button::new(
                    egui::RichText::new(pack.mobile_upload_done)
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 140, 100))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 44.0 * s));
                if ui.add(done_btn).clicked() {
                    app.screen = AppScreen::Gallery;
                }

                ui.add_space(6.0 * s);

                // Small secondary link for cable connection
                let cable_btn = egui::Button::new(
                    egui::RichText::new(pack.mobile_cable_connect)
                        .size(11.0 * s)
                        .color(Color32::from_gray(140)),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
                .frame(false);
                if ui.add(cable_btn).clicked() {
                    app.screen = AppScreen::MobileMenu;
                }
            });
        });
}

// =============================================================================
// PHONE FOLDER SELECT SCREEN (Windows — pick a folder before loading photos)
// =============================================================================
#[cfg(windows)]
fn draw_phone_folder_select(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(16.0 * s),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::MobileMenu;
                    #[cfg(windows)]
                    {
                        app.mtp_cmd_tx = None;
                        app.mtp_res_rx = None;
                    }
                }
            });

            ui.add_space(8.0 * s);
            ui.label(
                egui::RichText::new(pack.phone_folders_title)
                    .size(20.0 * s)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(4.0 * s);
            ui.label(
                egui::RichText::new(pack.phone_folders_hint)
                    .size(12.0 * s)
                    .color(Color32::from_gray(140)),
            );
            ui.add_space(12.0 * s);

            let non_empty: Vec<_> = app
                .mtp_folders
                .iter()
                .filter(|f| f.item_count > 0)
                .cloned()
                .collect();
            if non_empty.is_empty() {
                ui.label(
                    egui::RichText::new("Inga bildmappar hittades i DCIM")
                        .size(13.0 * s)
                        .color(Color32::from_gray(120)),
                );
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for folder in &non_empty {
                            let btn = egui::Button::new(
                                egui::RichText::new(format!(
                                    "📁 {}  ({} bilder)",
                                    folder.name, folder.item_count
                                ))
                                .size(14.0 * s)
                                .strong()
                                .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(35, 35, 55))
                            .rounding(Rounding::same(10.0 * s))
                            .min_size(Vec2::new(ui.available_width(), 52.0 * s));

                            if ui.add(btn).clicked() {
                                let path = folder.full_path.clone();
                                app.open_phone_folder(path);
                            }
                            ui.add_space(6.0 * s);
                        }
                    });
            }
        });
}

#[cfg(not(windows))]
fn draw_phone_folder_select(_ctx: &egui::Context, _app: &mut ZalStudio) {}

// =============================================================================
// GOOGLE PHOTOS AUTH SCREEN — QR code login
// =============================================================================
fn draw_google_drive_auth(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(16.0 * s),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::SourceSelect;
                    app.pkce_state = None;
                    app.auth_qr_texture = None;
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(10.0 * s);
                ui.label(
                    egui::RichText::new("📸")
                        .size(36.0 * s)
                        .color(Color32::from_rgb(234, 67, 53)),
                );
                ui.label(
                    egui::RichText::new(pack.google_drive_title)
                        .size(22.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(6.0 * s);
                ui.label(
                    egui::RichText::new(pack.google_drive_auth_hint)
                        .size(12.0 * s)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(20.0 * s);

                if app.pkce_state.is_some() {
                    ui.label(
                        egui::RichText::new("🌐 En webbläsare har öppnats")
                            .size(16.0 * s)
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    ui.add_space(8.0 * s);
                    ui.label(
                        egui::RichText::new("Logga in på Google i webbläsaren och godkänn åtkomst")
                            .size(14.0 * s)
                            .color(Color32::from_gray(180)),
                    );
                    ui.add_space(8.0 * s);
                    ui.label(
                        egui::RichText::new("(Stäng webbläsaren när du är klar)")
                            .size(12.0 * s)
                            .color(Color32::from_gray(120)),
                    );
                    ui.add_space(16.0 * s);

                    // Retry button in case browser didn't open
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Öppna webbläsaren igen")
                                    .size(14.0 * s)
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(40, 40, 60))
                            .rounding(Rounding::same(8.0 * s))
                            .min_size(Vec2::new(220.0 * s, 44.0 * s)),
                        )
                        .clicked()
                    {
                        if let Some(pkce) = &app.pkce_state {
                            let auth_url = crate::google_drive::build_auth_url(
                                &app.config.google_client_id,
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
                    }

                    ui.add_space(10.0 * s);
                    ui.label(
                        egui::RichText::new(&app.drive_status)
                            .size(12.0 * s)
                            .color(Color32::from_rgb(255, 200, 0)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(&app.drive_status)
                            .size(13.0 * s)
                            .color(Color32::from_rgb(255, 60, 60)),
                    );
                }
            });
        });
}

// =============================================================================
// GOOGLE DRIVE / PHOTOS PICKER
// =============================================================================
fn draw_google_drive_picker(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(12.0 * s),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
            });

            ui.add_space(4.0 * s);
            ui.heading(
                egui::RichText::new(pack.tab_drive)
                    .size(18.0 * s)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(8.0 * s);

            if app.drive_files.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Inga bilder i Drive")
                            .size(13.0 * s)
                            .color(Color32::from_gray(140)),
                    );
                });
            } else {
                // Responsive thumbnail size based on screen width
                let avail_w = ui.available_width();
                let thumb_size = (avail_w / 5.0).clamp(80.0, 160.0);
                let spacing = 10.0_f32;
                let item_w = thumb_size + spacing;
                let cols = ((avail_w - spacing) / item_w).floor() as usize;
                let cols = cols.max(1).min(8);

                // Calculate total grid width so we can center it
                let grid_width = cols as f32 * item_w - spacing;
                let left_pad = (avail_w - grid_width).max(0.0) / 2.0;

                // Pre-load thumbnails
                let files: Vec<crate::google_drive::DriveFile> = app.drive_files.clone();
                for file in &files {
                    if let Some(url) = &file.thumbnail_link {
                        let _ = app.drive_thumbnail(ctx, &file.id, url);
                    }
                }

                let mut clicked_idx: Option<usize> = None;

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let rows = (files.len() + cols - 1) / cols;

                        for row in 0..rows {
                            ui.add_space(left_pad);
                            ui.horizontal(|ui| {
                                for col in 0..cols {
                                    let idx = row * cols + col;
                                    if idx >= files.len() {
                                        break;
                                    }
                                    let file = &files[idx];
                                    let is_selected =
                                        app.drive_selected.get(idx).copied().unwrap_or(false);

                                    let bg = if is_selected {
                                        Color32::from_rgb(0, 150, 220)
                                    } else {
                                        Color32::from_rgb(30, 30, 45)
                                    };

                                    let frame = Frame::none()
                                        .fill(bg)
                                        .rounding(Rounding::same(8.0 * s))
                                        .inner_margin(4.0 * s);

                                    let response = frame
                                        .show(ui, |ui| {
                                            ui.set_max_width(thumb_size);
                                            ui.set_min_width(thumb_size);
                                            ui.vertical_centered(|ui| {
                                                // Thumbnail
                                                let img_size = thumb_size - 8.0;
                                                if let Some(tex) =
                                                    app.drive_thumbnails.get(&file.id)
                                                {
                                                    ui.image((
                                                        tex.id(),
                                                        Vec2::new(img_size, img_size),
                                                    ));
                                                } else {
                                                    ui.add_sized(
                                                        Vec2::new(img_size, img_size),
                                                        egui::Label::new(
                                                            egui::RichText::new("📷")
                                                                .size(thumb_size * 0.25),
                                                        ),
                                                    );
                                                }

                                                // Filename (truncated)
                                                let max_chars = (thumb_size / 8.0) as usize;
                                                let name = if file.name.len() > max_chars {
                                                    format!(
                                                        "{}...",
                                                        &file.name[..max_chars.saturating_sub(3)]
                                                    )
                                                } else {
                                                    file.name.clone()
                                                };
                                                ui.label(
                                                    egui::RichText::new(name)
                                                        .size(10.0 * s)
                                                        .color(Color32::from_gray(200)),
                                                );

                                                // Checkmark if selected
                                                if is_selected {
                                                    ui.label(egui::RichText::new("✅").size(14.0 * s));
                                                }
                                            });
                                        })
                                        .response;

                                    if response.clicked() {
                                        clicked_idx = Some(idx);
                                    }

                                    if col + 1 < cols && idx + 1 < files.len() {
                                        ui.add_space(spacing);
                                    }
                                }
                            });
                        }
                    });

                if let Some(i) = clicked_idx {
                    if let Some(sel) = app.drive_selected.get_mut(i) {
                        *sel = !*sel;
                    }
                }

                ui.add_space(12.0 * s);

                let selected_count = app.drive_selected.iter().filter(|&&s| s).count();
                let btn_width = ui.available_width().min(320.0);
                ui.vertical_centered(|ui| {
                    let download_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} ({})",
                            pack.google_drive_download, selected_count
                        ))
                        .size(15.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(66, 133, 244))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 48.0 * s));

                    if ui.add(download_btn).clicked() && selected_count > 0 {
                        app.download_selected_drive_files();
                    }
                });
            }
        });
}

// =============================================================================
// GOOGLE PHOTOS PICKER
// =============================================================================
fn draw_google_photos_picker(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(12.0 * s),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
            });

            ui.add_space(4.0 * s);

            // Tabs
            ui.horizontal(|ui| {
                let drive_btn = egui::Button::new(
                    egui::RichText::new(pack.tab_drive)
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::from_gray(200)),
                )
                .fill(Color32::from_rgb(30, 30, 45))
                .rounding(Rounding::same(8.0 * s))
                .min_size(Vec2::new(ui.available_width() * 0.45, 36.0 * s));
                if ui.add(drive_btn).clicked() {
                    app.screen = AppScreen::GoogleDrivePicker;
                }

                ui.add_space(8.0 * s);

                let photos_btn = egui::Button::new(
                    egui::RichText::new(pack.tab_photos)
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(200, 60, 120))
                .rounding(Rounding::same(8.0 * s))
                .min_size(Vec2::new(ui.available_width(), 36.0 * s));
                ui.add(photos_btn);
            });

            ui.add_space(8.0 * s);

            if app.photo_items.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Inga bilder i Google Foto")
                            .size(13.0 * s)
                            .color(Color32::from_gray(140)),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for (i, photo) in app.photo_items.iter().enumerate() {
                            let mut is_selected =
                                app.photo_selected.get(i).copied().unwrap_or(false);

                            let bg = if is_selected {
                                Color32::from_rgb(200, 60, 120)
                            } else {
                                Color32::from_rgb(30, 30, 45)
                            };

                            let frame = Frame::none()
                                .fill(bg)
                                .rounding(Rounding::same(8.0 * s))
                                .inner_margin(10.0 * s);

                            let response = frame
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let checkbox = ui.add(egui::Checkbox::new(
                                            &mut is_selected,
                                            egui::RichText::new(&photo.filename)
                                                .size(13.0 * s)
                                                .strong()
                                                .color(Color32::WHITE),
                                        ));
                                        if checkbox.clicked() {
                                            if let Some(sel) = app.photo_selected.get_mut(i) {
                                                *sel = !*sel;
                                            }
                                        }
                                    });
                                })
                                .response;

                            if response.clicked() {
                                if let Some(sel) = app.photo_selected.get_mut(i) {
                                    *sel = !*sel;
                                }
                            }
                        }
                    });

                ui.add_space(8.0 * s);

                let selected_count = app.photo_selected.iter().filter(|&&s| s).count();
                let btn_width = ui.available_width().min(320.0);
                ui.vertical_centered(|ui| {
                    let download_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} ({})",
                            pack.google_drive_download, selected_count
                        ))
                        .size(15.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(200, 60, 120))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 48.0 * s));

                    if ui.add(download_btn).clicked() && selected_count > 0 {
                        app.download_selected_google_photos();
                    }
                });
            }
        });
}

// =============================================================================
// TOAST
// =============================================================================
fn draw_toast(ctx: &egui::Context, app: &mut ZalStudio) {
    let s = touch_scale(ctx);
    if let Some((ref msg, t)) = app.toast {
        let alpha = (t * 255.0 / 0.5).min(255.0) as u8;
        let screen = ctx.screen_rect();
        let toast_width = 340.0 * s;
        let toast_height = 50.0 * s;
        let pos = egui::pos2(
            screen.center().x - toast_width / 2.0,
            screen.max.y - toast_height - 16.0,
        );

        egui::Area::new("toast".into())
            .fixed_pos(pos)
            .show(ctx, |ui| {
                let frame = Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(30, 30, 45, alpha))
                    .rounding(Rounding::same(10.0 * s))
                    .stroke(Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(60, 60, 90, alpha),
                    ))
                    .inner_margin(14.0 * s);

                frame.show(ui, |ui| {
                    ui.set_min_width(toast_width);
                    ui.set_min_height(toast_height);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(msg)
                                .size(14.0 * s)
                                .strong()
                                .color(Color32::from_rgb(0, 220, 255)),
                        );
                    });
                });
            });
    }
}

// =============================================================================
// PRINT DONE SCREEN
// =============================================================================
fn draw_print_done(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(20.0 * s),
        )
        .show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("🖨")
                            .size(64.0 * s)
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    ui.add_space(20.0 * s);
                    ui.label(
                        egui::RichText::new(pack.print_done_title)
                            .size(26.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(12.0 * s);
                    ui.label(
                        egui::RichText::new(pack.print_done_subtitle)
                            .size(14.0 * s)
                            .color(Color32::from_gray(160)),
                    );
                    ui.add_space(40.0 * s);

                    let time = ui.ctx().input(|i| i.time);
                    let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                    ui.label(
                        egui::RichText::new(spinner)
                            .size(32.0 * s)
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    ui.add_space(8.0 * s);
                    ui.label(
                        egui::RichText::new(format!("{:.0}s", app.print_done_timer))
                            .size(12.0 * s)
                            .color(Color32::from_gray(120)),
                    );
                });
            });
        });
}

// =============================================================================
// PAYMENT SCREEN
// =============================================================================
fn draw_payment(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    let total = app.queue_total_price();
    let count = app.queue.len();

    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(20.0 * s),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(24.0 * s);
                ui.label(egui::RichText::new("💳").size(56.0 * s));
                ui.add_space(16.0 * s);
                ui.label(
                    egui::RichText::new(pack.payment_title)
                        .size(26.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new(format!("{}: {:.0} kr", pack.total_label, total))
                        .size(22.0 * s)
                        .strong()
                        .color(Color32::from_rgb(80, 220, 120)),
                );
                ui.add_space(4.0 * s);
                ui.label(
                    egui::RichText::new(format!("{}: {}", pack.print_queue, count))
                        .size(14.0 * s)
                        .color(Color32::from_gray(160)),
                );

                ui.add_space(32.0 * s);

                let btn_width = ui.available_width().min(320.0);

                if ui
                    .add_sized(
                        Vec2::new(btn_width, 56.0 * s),
                        egui::Button::new(
                            egui::RichText::new(pack.payment_card)
                                .size(18.0 * s)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(0, 140, 100))
                        .rounding(Rounding::same(10.0 * s)),
                    )
                    .clicked()
                {
                    // TODO: integrate card terminal
                    app.print_queue();
                }

                ui.add_space(12.0 * s);

                if ui
                    .add_sized(
                        Vec2::new(btn_width, 52.0 * s),
                        egui::Button::new(
                            egui::RichText::new(pack.payment_swish)
                                .size(16.0 * s)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(70, 50, 170))
                        .rounding(Rounding::same(10.0 * s)),
                    )
                    .clicked()
                {
                    // TODO: integrate Swish
                    app.print_queue();
                }

                ui.add_space(24.0 * s);

                if nav_button(ui, pack.back, s).clicked() {
                    app.screen = AppScreen::Queue;
                }
            });
        });
}

// =============================================================================
// WIRED PHONE PICKER SCREEN — thumbnail grid, touch-friendly
// =============================================================================
fn draw_wired_phone_picker(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    let is_linux_mtp =
        app.phone_type == crate::app::PhoneType::Android && cfg!(target_os = "linux");
    let is_windows_wpd = app.phone_type == crate::app::PhoneType::Android && cfg!(windows);
    let is_android_direct = is_linux_mtp || is_windows_wpd;

    let photo_count = if is_android_direct {
        app.mtp_photos.len()
    } else {
        app.phone_photos.len()
    };
    let selected_count = if is_android_direct {
        app.mtp_selected.iter().filter(|&&s| s).count()
    } else {
        app.phone_selected.iter().filter(|&&s| s).count()
    };
    let all_selected = photo_count > 0 && selected_count == photo_count;

    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(14, 14, 22))
                .inner_margin(16.0 * s),
        )
        .show(ctx, |ui| {
            // Top navigation bar — large touch-friendly buttons
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 56.0 * s),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        // Back button (large, with text)
                        let back_btn = egui::Button::new(
                            egui::RichText::new(format!("← {}", pack.back))
                                .size(18.0 * s)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(40, 40, 58))
                        .rounding(Rounding::same(12.0 * s))
                        .min_size(Vec2::new(140.0 * s, 52.0 * s));
                        if ui.add(back_btn).clicked() {
                            app.screen = if app.picker_source == crate::app::PickerSource::Usb {
                                AppScreen::SourceSelect
                            } else {
                                AppScreen::MobileMenu
                            };
                        }

                        ui.with_layout(
                            egui::Layout::top_down(egui::Align::Center)
                                .with_cross_align(egui::Align::Center),
                            |ui| {
                                let picker_title =
                                    if app.picker_source == crate::app::PickerSource::Usb {
                                        pack.usb_select_photos
                                    } else {
                                        pack.wired_select_photos
                                    };
                                ui.label(
                                    egui::RichText::new(picker_title)
                                        .size(18.0 * s)
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                            },
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if selected_count > 0 {
                                let import_btn = egui::Button::new(
                                    egui::RichText::new(format!(
                                        "{} ({}) →",
                                        pack.wired_import, selected_count
                                    ))
                                    .size(16.0 * s)
                                    .strong()
                                    .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(0, 140, 100))
                                .rounding(Rounding::same(12.0 * s))
                                .min_size(Vec2::new(180.0 * s, 52.0 * s));
                                if ui.add(import_btn).clicked() {
                                    app.import_selected_phone_photos();
                                }
                            }
                        });
                    },
                );
            });

            ui.add_space(12.0 * s);

            if photo_count == 0 {
                ui.label(
                    egui::RichText::new("Inga bilder hittades")
                        .size(14.0 * s)
                        .color(Color32::from_gray(140)),
                );
            } else {
                // Thumbnail grid using horizontal_wrapped
                let thumb_size = 140.0 * s;
                let gap = 14.0 * s;

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(gap, gap);

                            for idx in 0..photo_count {
                                let (photo, is_selected) = if is_android_direct {
                                    let p = &app.mtp_photos[idx];
                                    let sel = app.mtp_selected.get(idx).copied().unwrap_or(false);
                                    let ph = crate::wired_import::PhonePhoto {
                                        path: std::path::PathBuf::from(&p.file_name),
                                        file_name: p.file_name.clone(),
                                        file_size: p.file_size,
                                    };
                                    (ph, sel)
                                } else {
                                    let p = app.phone_photos[idx].clone();
                                    let sel = app.phone_selected.get(idx).copied().unwrap_or(false);
                                    (p, sel)
                                };

                                let (cell_id, cell_rect) =
                                    ui.allocate_space(Vec2::new(thumb_size, thumb_size));

                                // Interaction
                                let response =
                                    ui.interact(cell_rect, cell_id, egui::Sense::click());
                                if response.clicked() {
                                    if is_android_direct {
                                        if let Some(sel) = app.mtp_selected.get_mut(idx) {
                                            *sel = !*sel;
                                        }
                                    } else {
                                        if let Some(sel) = app.phone_selected.get_mut(idx) {
                                            *sel = !*sel;
                                        }
                                    }
                                }

                                let painter = ui.painter_at(cell_rect);

                                // Background
                                let bg = if is_selected {
                                    Color32::from_rgb(40, 60, 50)
                                } else {
                                    Color32::from_rgb(35, 35, 48)
                                };
                                painter.rect_filled(cell_rect, 8.0 * s, bg);

                                // Thumbnail image (background worker loads these asynchronously)
                                let inner = cell_rect.shrink(4.0 * s);
                                if let Some(tex) = app.cached_texture(&photo.path) {
                                    painter.image(
                                        tex.id(),
                                        inner,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        egui::Color32::WHITE,
                                    );
                                } else {
                                    painter.rect_filled(
                                        inner,
                                        6.0 * s,
                                        Color32::from_rgb(50, 50, 65),
                                    );
                                    let galley = painter.layout(
                                        "🖼".to_string(),
                                        egui::FontId::new(36.0 * s, egui::FontFamily::Proportional),
                                        Color32::from_gray(100),
                                        f32::INFINITY,
                                    );
                                    let text_pos = inner.center() - galley.rect.size() * 0.5;
                                    painter.galley(text_pos, galley, Color32::WHITE);
                                }

                                // Selection outline
                                if is_selected {
                                    painter.rect_stroke(
                                        cell_rect,
                                        8.0 * s,
                                        Stroke::new(3.5 * s, Color32::from_rgb(80, 220, 120)),
                                    );

                                    // Checkmark badge in upper-right
                                    let badge_r = 14.0 * s;
                                    let badge_center = egui::pos2(
                                        cell_rect.right() - badge_r - 2.0 * s,
                                        cell_rect.top() + badge_r + 2.0 * s,
                                    );
                                    painter.circle_filled(
                                        badge_center,
                                        badge_r,
                                        Color32::from_rgb(80, 220, 120),
                                    );
                                    let check_galley = painter.layout(
                                        "✓".to_string(),
                                        egui::FontId::new(16.0 * s, egui::FontFamily::Proportional),
                                        Color32::WHITE,
                                        f32::INFINITY,
                                    );
                                    let check_pos = badge_center - check_galley.rect.size() * 0.5;
                                    painter.galley(check_pos, check_galley, Color32::WHITE);
                                }
                            }
                        });
                    });

                ui.add_space(16.0 * s);

                let btn_width = ui.available_width().min(400.0 * s);
                ui.vertical_centered(|ui| {
                    let toggle_btn = egui::Button::new(
                        egui::RichText::new(if all_selected {
                            pack.deselect_all
                        } else {
                            pack.select_all
                        })
                        .size(15.0 * s)
                        .color(Color32::from_gray(200)),
                    )
                    .fill(Color32::from_rgb(40, 40, 55))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 48.0 * s));
                    if ui.add(toggle_btn).clicked() {
                        let new_val = !all_selected;
                        if is_linux_mtp || is_windows_wpd {
                            for sel in &mut app.mtp_selected {
                                *sel = new_val;
                            }
                        } else {
                            for sel in &mut app.phone_selected {
                                *sel = new_val;
                            }
                        }
                    }
                });
            }
        });
}

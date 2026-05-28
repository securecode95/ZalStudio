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
    style.visuals.panel_fill = Color32::from_rgb(10, 15, 26);
    style.visuals.window_fill = Color32::from_rgb(10, 15, 26);
    style.visuals.extreme_bg_color = Color32::from_rgb(10, 15, 26);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(17, 24, 39);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(17, 24, 39);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 40, 60);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 50, 70);
    style.visuals.selection.bg_fill = Color32::from_rgb(0, 229, 255);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(13, 19, 33);
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
        AppScreen::WiredPhonePicker => draw_wired_phone_picker(ctx, app),
        AppScreen::GoogleDriveAuth => draw_google_drive_auth(ctx, app),
        AppScreen::GoogleDrivePicker => draw_google_drive_picker(ctx, app),
        AppScreen::GooglePhotosPicker => draw_google_photos_picker(ctx, app),
        AppScreen::PrintDone => draw_print_done(ctx, app),
        AppScreen::PrintProgress => draw_print_progress(ctx, app),
        AppScreen::ThankYou => draw_thank_you(ctx, app),
        AppScreen::Payment => draw_payment(ctx, app),
        AppScreen::LayoutSelect => draw_layout_select(ctx, app),
        AppScreen::CollageEditor => draw_collage_editor(ctx, app),
        AppScreen::SettingsAuth => draw_settings_auth(ctx, app),
        AppScreen::Settings => draw_settings(ctx, app),
    }

    draw_toast(ctx, app);
}

// =============================================================================
// PRODUCT SELECT SCREEN (Foto / Album / Collage)
// =============================================================================
fn draw_product_select(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    let bg_main = Color32::from_rgb(10, 15, 26);
    let bg_card = Color32::from_rgb(17, 24, 39);
    let text_dark = Color32::from_rgb(224, 247, 255);
    let text_mid = Color32::from_rgb(142, 202, 230);
    let blue_accent = Color32::from_rgb(0, 229, 255);

    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(24.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

            // Top bar with title and settings button
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Zalstudio")
                        .size(22.0 * s)
                        .strong()
                        .color(blue_accent),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Settings button
                    let settings_btn = egui::Button::new(
                        egui::RichText::new("⚙")
                            .size(18.0 * s)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(40, 50, 70))
                    .rounding(Rounding::same(8.0 * s))
                    .min_size(Vec2::new(72.0 * s, 64.0 * s));
                    if ui.add(settings_btn).clicked() {
                        app.screen = AppScreen::SettingsAuth;
                        app.settings_pin_input.clear();
                        app.settings_auth_failed = false;
                    }
                    ui.add_space(8.0 * s);

                    // Language toggle
                    let lang_btn = egui::Button::new(
                        egui::RichText::new(app.lang.name())
                            .size(13.0 * s)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(50, 60, 80))
                    .rounding(Rounding::same(6.0 * s))
                    .min_size(Vec2::new(88.0 * s, 72.0 * s));
                    if ui.add(lang_btn).clicked() {
                        app.lang = app.lang.toggle();
                    }
                });
            });

            ui.vertical_centered(|ui| {
                ui.add_space(16.0 * s);
                ui.label(
                    egui::RichText::new("VÄLJ TJÄNST")
                        .size(28.0 * s)
                        .strong()
                        .color(text_dark),
                );
                ui.add_space(4.0 * s);
                ui.label(
                    egui::RichText::new(pack.product_select_title)
                        .size(16.0 * s)
                        .color(text_mid),
                );
                ui.add_space(20.0 * s);
            });

            // Build product grid (2 columns)
            let mut products: Vec<(String, String, String, String, f64, &'static str, Color32)> =
                Vec::new();
            for size in &app.config.paper_sizes {
                let price = app.config.price_for_size(size);
                products.push((
                    "foto".to_string(),
                    size.clone(),
                    format!("{} {}", pack.product_foto, size),
                    pack.product_foto_desc.to_string(),
                    price,
                    "🖼",
                    Color32::from_rgb(0, 229, 255),
                ));
                products.push((
                    "album".to_string(),
                    size.clone(),
                    format!("{} {}", pack.product_album, size),
                    pack.product_album_desc.to_string(),
                    price,
                    "📔",
                    Color32::from_rgb(0, 229, 255),
                ));
                products.push((
                    "collage".to_string(),
                    size.clone(),
                    format!("{} {}", pack.product_collage, size),
                    pack.product_collage_desc.to_string(),
                    price,
                    "🎨",
                    Color32::from_rgb(180, 80, 200),
                ));
            }

            let cols = 2;
            let gap = 12.0 * s;
            let avail = ui.available_width();
            let cell_w = (avail - gap * (cols as f32 - 1.0)) / cols as f32;

            for chunk in products.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.set_width(avail);
                    for (product_type, size, title, _desc, price, icon, accent) in chunk {
                        let cell = ui.allocate_ui_with_layout(
                            Vec2::new(cell_w, 140.0 * s),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                let btn_rect = ui.max_rect();
                                let resp = ui.interact(
                                    btn_rect,
                                    ui.id().with(title),
                                    egui::Sense::click(),
                                );

                                let bg = if resp.hovered() {
                                    Color32::from_rgb(10, 15, 26)
                                } else {
                                    bg_card
                                };
                                ui.painter().rect_filled(btn_rect, 12.0 * s, bg);
                                ui.painter().rect_stroke(
                                    btn_rect,
                                    12.0 * s,
                                    Stroke::new(
                                        2.0 * s,
                                        if resp.hovered() {
                                            blue_accent
                                        } else {
                                            Color32::from_rgb(30, 40, 60)
                                        },
                                    ),
                                );

                                // Inner content
                                let inner = btn_rect.shrink(12.0 * s);
                                ui.allocate_ui_at_rect(inner, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(8.0 * s);
                                        ui.label(
                                            egui::RichText::new(*icon)
                                                .size(36.0 * s)
                                                .color(*accent),
                                        );
                                        ui.add_space(6.0 * s);
                                        ui.label(
                                            egui::RichText::new(title)
                                                .size(15.0 * s)
                                                .strong()
                                                .color(text_dark),
                                        );
                                        ui.add_space(4.0 * s);
                                        ui.label(
                                            egui::RichText::new(format!("{:.0} kr", price))
                                                .size(14.0 * s)
                                                .strong()
                                                .color(blue_accent),
                                        );
                                    });
                                });

                                resp
                            },
                        );

                        if cell.inner.clicked() {
                            app.selected_product = product_type.clone();
                            app.selected_product_size = size.clone();
                            if let Some(idx) = app.config.paper_sizes.iter().position(|s| s == size)
                            {
                                app.paper_size_idx = idx;
                            }
                            app.screen = AppScreen::SourceSelect;
                        }

                        if chunk.len() > 1 && ui.available_width() > gap {
                            ui.add_space(gap);
                        }
                    }
                });
                ui.add_space(gap);
            }
        });
}

// =============================================================================
// WELCOME SCREEN
// =============================================================================
fn draw_welcome(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    let bg_main = Color32::from_rgb(10, 15, 26);
    let text_dark = Color32::from_rgb(224, 247, 255);
    let text_mid = Color32::from_rgb(142, 202, 230);
    let blue_accent = Color32::from_rgb(0, 229, 255);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg_main).inner_margin(24.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

            // Top bar: back to product, product label + language toggle
            ui.horizontal(|ui| {
                if nav_button(ui, "◀", s).clicked() {
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
                            .color(Color32::from_rgb(0, 229, 255)),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    let lang_btn = egui::Button::new(
                        egui::RichText::new(app.lang.name())
                            .size(13.0 * s)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(50, 60, 80))
                    .rounding(Rounding::same(6.0 * s))
                    .min_size(Vec2::new(88.0 * s, 72.0 * s));
                    if ui.add(lang_btn).clicked() {
                        app.lang = app.lang.toggle();
                    }
                });
            });

            ui.vertical_centered(|ui| {
                ui.add_space(60.0 * s);

                // Title
                ui.label(
                    egui::RichText::new("Zalstudio Kiosk")
                        .size(38.0 * s)
                        .strong()
                        .color(text_dark),
                );
                ui.add_space(10.0 * s);

                // Subtitle
                ui.label(
                    egui::RichText::new(pack.source_select_subtitle)
                        .size(17.0 * s)
                        .color(text_mid),
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
                .fill(blue_accent)
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 88.0 * s));
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
                .fill(Color32::from_rgb(160, 60, 180))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 88.0 * s));
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
                .fill(Color32::from_rgb(10, 15, 26))
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
                        .color(Color32::from_rgb(0, 229, 255)),
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
                        .color(Color32::from_rgb(142, 202, 230)),
                );
                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new(pack.usb_plug_hint)
                        .size(13.0 * s)
                        .color(Color32::from_rgb(180, 200, 220)),
                );

                ui.add_space(40.0 * s);

                // Animated spinner
                let time = ui.ctx().input(|i| i.time);
                let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                ui.label(
                    egui::RichText::new(spinner)
                        .size(40.0 * s)
                        .color(Color32::from_rgb(0, 229, 255)),
                );

                ui.add_space(16.0 * s);
                ui.label(
                    egui::RichText::new(pack.usb_plug_searching)
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::from_rgb(0, 229, 255)),
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
    .min_size(Vec2::new(width, 72.0 * s));
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
    let bg_main = Color32::from_rgb(10, 15, 26);
    let bg_card = Color32::from_rgb(17, 24, 39);
    let bg_panel = Color32::from_rgb(10, 15, 26);
    let text_dark = Color32::from_rgb(224, 247, 255);
    let text_mid = Color32::from_rgb(142, 202, 230);
    let green_accent = Color32::from_rgb(0, 229, 255);
    let green_dark = Color32::from_rgb(0, 153, 170);
    let red_border = Color32::from_rgb(255, 51, 102);

    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(6.0 * s))
        .show(ctx, |ui| {
            let full_rect = ui.max_rect();
            ui.painter().rect_filled(full_rect, 0.0, bg_main);

            let avail_w = ui.available_width();
            let avail_h = ui.available_height();
            let sidebar_w = 220.0 * s;
            let rightbar_w = 220.0 * s;
            let center_w = avail_w - sidebar_w - rightbar_w - 16.0 * s;
            let bottom_h = 190.0 * s;
            let grid_h = avail_h - bottom_h - 8.0 * s;

            if app.photos.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(pack.no_photos).size(16.0 * s).color(text_mid));
                });
                return;
            }

            // ═══════════════════════════════════════════════════════════════════
            // MAIN HORIZONTAL: Left sidebar | Center (grid+bottom) | Right sidebar
            // ═══════════════════════════════════════════════════════════════════
            ui.horizontal(|ui| {
                ui.set_height(avail_h);

                // ── LEFT SIDEBAR ─────────────────────────────────────────────
                ui.allocate_ui_with_layout(
                    Vec2::new(sidebar_w, avail_h),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        let sb_rect = ui.max_rect();
                        ui.painter().rect_filled(sb_rect, 0.0, bg_panel);

                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0 * s);

                            // Back to source
                            if sidebar_btn(ui, "▲", s).clicked() {
                                app.screen = AppScreen::SourceSelect;
                            }
                            ui.add_space(16.0 * s);

                            if sidebar_btn(ui, "◀", s).clicked() {
                                app.screen = AppScreen::SourceSelect;
                            }
                            ui.add_space(6.0 * s);
                            ui.label(egui::RichText::new("Avsluta").size(12.0 * s).color(text_dark));

                            ui.add_space(28.0 * s);

                            let has_sel = !app.selected_photos.is_empty();

                            // Förhandsvy
                            if sidebar_btn(ui, "🔍", s).clicked() && has_sel {
                                app.selected_photo = app.selected_photos[0];
                                app.current_edit = crate::app::PhotoEdit::default();
                                app.screen = AppScreen::Preview;
                            }
                            ui.add_space(4.0 * s);
                            ui.label(egui::RichText::new("Förhandsvy").size(12.0 * s).color(text_dark));

                            ui.add_space(24.0 * s);

                            // Redigera
                            if sidebar_btn(ui, "✎", s).clicked() && has_sel {
                                app.selected_photo = app.selected_photos[0];
                                app.current_edit = crate::app::PhotoEdit::default();
                                app.screen = AppScreen::Preview;
                            }
                            ui.add_space(4.0 * s);
                            ui.label(egui::RichText::new("Redigera").size(12.0 * s).color(text_dark));

                            ui.add_space(48.0 * s);

                            // Gå vidare (big green arrow at bottom of sidebar)
                            let can_go = app.total_order_copies() > 0;
                            let go_btn = egui::Button::new(
                                egui::RichText::new("▼").size(32.0 * s).color(Color32::WHITE),
                            )
                            .fill(if can_go { green_accent } else { Color32::from_rgb(50, 60, 80) })
                            .rounding(Rounding::same(12.0 * s))
                            .min_size(Vec2::new(88.0 * s, 88.0 * s));
                            if ui.add(go_btn).clicked() && can_go {
                                app.add_selected_to_queue();
                                app.selected_photos.clear();
                                app.photo_copies.clear();
                                app.screen = AppScreen::Payment;
                            }
                            ui.add_space(2.0 * s);
                            ui.label(egui::RichText::new("Gå vidare").size(12.0 * s).color(text_dark));
                        });
                    },
                );

                ui.add_space(8.0 * s);

                // ── CENTER: Photo grid + bottom size controls ─────────────────
                ui.allocate_ui_with_layout(
                    Vec2::new(center_w, avail_h),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        // Photo grid
                        ui.allocate_ui_with_layout(
                            Vec2::new(center_w, grid_h),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                let thumb_base = 220.0;
                                let spacing = 10.0;
                                let cell_w = thumb_base * s + spacing * s;
                                let cols = ((center_w / cell_w).floor() as usize).max(2);
                                let thumb_size = thumb_base * s;
                                let photo_count = app.photos.len();

                                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                                    for row_start in (0..photo_count).step_by(cols) {
                                        ui.horizontal(|ui| {
                                            for i in row_start..(row_start + cols).min(photo_count) {
                                                let photo_path = app.photos[i].path.clone();
                                                let photo_file_name = app.photos[i].file_name.clone();
                                                let is_selected = app.selected_photos.contains(&i);
                                                let total_copies = app.total_copies_for_photo(i);

                                                let _cell = ui.allocate_ui_with_layout(
                                                    Vec2::new(thumb_size + 6.0 * s, thumb_size + 36.0 * s),
                                                    egui::Layout::top_down(egui::Align::Center),
                                                    |ui| {
                                                        let cell_rect = ui.max_rect();
                                                        let img_rect = egui::Rect::from_min_size(
                                                            cell_rect.min + Vec2::new(3.0 * s, 3.0 * s),
                                                            Vec2::new(thumb_size, thumb_size),
                                                        );

                                                        let stroke = if is_selected {
                                                            Stroke::new(2.0 * s, red_border)
                                                        } else {
                                                            Stroke::new(1.0 * s, Color32::from_rgb(0, 229, 255))
                                                        };
                                                        ui.painter().rect_filled(cell_rect, 4.0 * s, bg_card);
                                                        ui.painter().rect_stroke(cell_rect, 4.0 * s, stroke);

                                                        if let Some(texture) = app.texture_for(ctx, &photo_path) {
                                                            let tex_size = texture.size_vec2();
                                                            let aspect = tex_size.x / tex_size.y;
                                                            let (tw, th) = if aspect > 1.0 {
                                                                (thumb_size, thumb_size / aspect)
                                                            } else {
                                                                (thumb_size * aspect, thumb_size)
                                                            };
                                                            let img_pos = img_rect.center() - Vec2::new(tw * 0.5, th * 0.5);
                                                            let displayed_rect = egui::Rect::from_min_size(img_pos, Vec2::new(tw, th));
                                                            ui.painter().image(
                                                                texture.id(),
                                                                displayed_rect,
                                                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                                                Color32::WHITE,
                                                            );

                                                            // Draw red dashed crop guide for selected paper size
                                                            let selected_size = &app.selected_product_size;
                                                            let paper_aspect = paper_size_aspect(selected_size);
                                                            let img_aspect = tw / th.max(1.0);
                                                            let (crop_w, crop_h) = if img_aspect > paper_aspect {
                                                                (th * paper_aspect, th)
                                                            } else {
                                                                (tw, tw / paper_aspect.max(0.01))
                                                            };
                                                            let crop_rect = egui::Rect::from_center_size(
                                                                displayed_rect.center(),
                                                                Vec2::new(crop_w, crop_h),
                                                            );
                                                            draw_dashed_rect(ui.painter(), crop_rect, Stroke::new(1.5 * s, Color32::from_rgb(255, 51, 102)), 6.0 * s, 4.0 * s);
                                                        }

                                                        let resp = ui.interact(cell_rect, ui.id().with(i), egui::Sense::click());
                                                        if resp.clicked() {
                                                            if is_selected {
                                                                app.selected_photos.retain(|&x| x != i);
                                                            } else {
                                                                app.selected_photos.push(i);
                                                            }
                                                            app.selected_photo = i;
                                                        }

                                                        let label_rect = egui::Rect::from_min_max(
                                                            egui::pos2(cell_rect.min.x, img_rect.max.y + 1.0 * s),
                                                            egui::pos2(cell_rect.max.x, cell_rect.max.y),
                                                        );
                                                        ui.allocate_ui_at_rect(label_rect, |ui| {
                                                            ui.vertical_centered(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new(&photo_file_name)
                                                                        .size(8.0 * s)
                                                                        .color(text_dark),
                                                                );
                                                            });
                                                        });

                                                        if total_copies > 0 {
                                                            let badge_r = 16.0 * s;
                                                            let badge_center = cell_rect.max - Vec2::new(badge_r + 3.0 * s, badge_r + 3.0 * s);
                                                            ui.painter().circle_filled(badge_center, badge_r, green_accent);
                                                            let galley = ui.painter().layout(
                                                                total_copies.to_string(),
                                                                egui::FontId::new(13.0 * s, egui::FontFamily::Proportional),
                                                                Color32::WHITE,
                                                                f32::INFINITY,
                                                            );
                                                            let text_pos = badge_center - galley.rect.size() * 0.5;
                                                            ui.painter().galley(text_pos, galley, Color32::WHITE);
                                                        }

                                                        resp
                                                    },
                                                );
                                            }
                                        });
                                        ui.add_space(4.0 * s);
                                    }
                                });
                            },
                        );

                        ui.add_space(6.0 * s);

                        // Bottom size control strips (horizontal)
                        ui.allocate_ui_with_layout(
                            Vec2::new(center_w, bottom_h),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                let bot_rect = ui.max_rect();
                                ui.painter().rect_filled(bot_rect, 6.0 * s, bg_panel);

                                let selected_indices: Vec<usize> = app.selected_photos.clone();
                                let first_sel = selected_indices.first().copied();
                                let has_sel = !selected_indices.is_empty();

                                ui.horizontal(|ui| {
                                    ui.add_space(8.0 * s);

                                    // Warning text (left side of bottom)
                                    ui.vertical(|ui| {
                                        ui.add_space(8.0 * s);
                                        ui.label(egui::RichText::new("⚠").size(24.0 * s).color(Color32::from_rgb(255, 200, 0)));
                                        ui.add_space(2.0 * s);
                                        ui.label(
                                            egui::RichText::new("Visas denna ikon under bilden så är det otillräcklig kvalitet.")
                                                .size(11.0 * s)
                                                .color(text_mid),
                                        );
                                    });

                                    ui.add_space(16.0 * s);

                                    // Size strip for selected product size only
                                    let size = app.selected_product_size.clone();
                                    let price = app.config.price_for_size(&size);
                                    let counts: Vec<u32> = selected_indices.iter().map(|&i| app.photo_copy_count(i, &size)).collect();
                                    let count_text = if counts.len() > 1 {
                                        let first = counts[0];
                                        if counts.iter().all(|&c| c == first) {
                                            format!("{}", first)
                                        } else {
                                            "—".to_string()
                                        }
                                    } else {
                                        format!("{}", counts.first().copied().unwrap_or(0))
                                    };

                                    ui.vertical(|ui| {
                                        ui.add_space(6.0 * s);
                                        ui.label(
                                            egui::RichText::new(format!("Skriv ut {} · {:.0} kr", size, price))
                                                .size(13.0 * s)
                                                .strong()
                                                .color(text_dark),
                                        );
                                        ui.add_space(4.0 * s);
                                        ui.horizontal(|ui| {
                                            let minus = egui::Button::new(
                                                egui::RichText::new("−").size(24.0 * s).color(Color32::WHITE),
                                            )
                                            .fill(Color32::from_rgb(30, 40, 60))
                                            .rounding(Rounding::same(8.0 * s))
                                            .min_size(Vec2::new(56.0 * s, 56.0 * s));
                                            if ui.add(minus).clicked() && has_sel {
                                                for &idx in &selected_indices {
                                                    let cur = app.photo_copy_count(idx, &size);
                                                    if cur > 0 { app.set_photo_copy_count(idx, &size, cur - 1); }
                                                }
                                            }

                                            ui.add_space(4.0 * s);
                                            ui.label(
                                                egui::RichText::new(&count_text)
                                                    .size(24.0 * s)
                                                    .strong()
                                                    .color(text_dark),
                                            );
                                            ui.add_space(4.0 * s);

                                            let plus = egui::Button::new(
                                                egui::RichText::new("+").size(24.0 * s).color(Color32::WHITE),
                                            )
                                            .fill(Color32::from_rgb(30, 40, 60))
                                            .rounding(Rounding::same(8.0 * s))
                                            .min_size(Vec2::new(56.0 * s, 56.0 * s));
                                            if ui.add(plus).clicked() && has_sel {
                                                for &idx in &selected_indices {
                                                    let cur = app.photo_copy_count(idx, &size);
                                                    app.set_photo_copy_count(idx, &size, cur + 1);
                                                }
                                            }
                                        });
                                    });

                                    // Total
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(8.0 * s);
                                        let total_copies = app.total_order_copies();
                                        let total_price = app.total_order_price();
                                        ui.vertical(|ui| {
                                            ui.add_space(8.0 * s);
                                            ui.label(
                                                egui::RichText::new(format!("{} utskrifter · {:.0} kr", total_copies, total_price))
                                                    .size(15.0 * s)
                                                    .strong()
                                                    .color(green_dark),
                                            );
                                        });
                                    });
                                });
                            },
                        );
                    },
                );

                ui.add_space(8.0 * s);

                // ── RIGHT SIDEBAR ────────────────────────────────────────────
                ui.allocate_ui_with_layout(
                    Vec2::new(rightbar_w, avail_h),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        let rs_rect = ui.max_rect();
                        ui.painter().rect_filled(rs_rect, 0.0, bg_panel);

                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0 * s);

                            // Bildordning title
                            ui.label(
                                egui::RichText::new("Bildordning")
                                    .size(14.0 * s)
                                    .strong()
                                    .color(text_dark),
                            );
                            ui.add_space(12.0 * s);

                            // Sortingläge label
                            let sort_label = match app.gallery_sort {
                                crate::app::GallerySort::Date => pack.sort_by_date,
                                crate::app::GallerySort::Name => pack.sort_by_name,
                            };
                            ui.label(
                                egui::RichText::new(sort_label)
                                    .size(12.0 * s)
                                    .color(text_mid),
                            );
                            ui.add_space(8.0 * s);

                            // Sort buttons with clear text labels
                            let date_btn = egui::Button::new(
                                egui::RichText::new("Datum")
                                    .size(18.0 * s)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(if matches!(app.gallery_sort, crate::app::GallerySort::Date) {
                                Color32::from_rgb(0, 229, 255)
                            } else {
                                Color32::from_rgb(40, 50, 70)
                            })
                            .rounding(Rounding::same(10.0 * s))
                            .min_size(Vec2::new(200.0 * s, 88.0 * s));
                            if ui.add(date_btn).clicked() {
                                app.gallery_sort = crate::app::GallerySort::Date;
                                app.apply_gallery_sort();
                            }
                            ui.add_space(8.0 * s);

                            let name_btn = egui::Button::new(
                                egui::RichText::new("Namn")
                                    .size(18.0 * s)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(if matches!(app.gallery_sort, crate::app::GallerySort::Name) {
                                Color32::from_rgb(0, 229, 255)
                            } else {
                                Color32::from_rgb(40, 50, 70)
                            })
                            .rounding(Rounding::same(10.0 * s))
                            .min_size(Vec2::new(200.0 * s, 88.0 * s));
                            if ui.add(name_btn).clicked() {
                                app.gallery_sort = crate::app::GallerySort::Name;
                                app.apply_gallery_sort();
                            }
                            ui.add_space(8.0 * s);

                            let dir_label = if app.gallery_sort_ascending { "Stigande ▲" } else { "Fallande ▼" };
                            let dir_btn = egui::Button::new(
                                egui::RichText::new(dir_label)
                                    .size(16.0 * s)
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(40, 50, 70))
                            .rounding(Rounding::same(10.0 * s))
                            .min_size(Vec2::new(200.0 * s, 88.0 * s));
                            if ui.add(dir_btn).clicked() {
                                app.gallery_sort_ascending = !app.gallery_sort_ascending;
                                app.apply_gallery_sort();
                            }
                        });
                    },
                );
            });
        });
}

/// Small square sidebar icon button
fn sidebar_btn(ui: &mut egui::Ui, label: &str, s: f32) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .size(32.0 * s)
                .color(Color32::WHITE),
        )
        .fill(Color32::from_rgb(40, 50, 70))
        .rounding(Rounding::same(12.0 * s))
        .min_size(Vec2::new(120.0 * s, 120.0 * s)),
    )
}

fn nav_button(ui: &mut egui::Ui, label: &str, s: f32) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .size(26.0 * s)
                .color(Color32::WHITE),
        )
        .fill(Color32::from_rgb(40, 50, 70))
        .rounding(Rounding::same(12.0 * s))
        .min_size(Vec2::new(88.0 * s, 88.0 * s)),
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
/// Draw a rectangle with dashed lines (segments).
fn draw_dashed_rect(
    painter: &egui::Painter,
    rect: egui::Rect,
    stroke: Stroke,
    dash_len: f32,
    gap_len: f32,
) {
    let step = dash_len + gap_len;

    // Top edge
    let mut x = rect.min.x;
    while x < rect.max.x {
        let seg_start = x;
        let seg_end = (x + dash_len).min(rect.max.x);
        painter.line_segment(
            [
                egui::pos2(seg_start, rect.min.y),
                egui::pos2(seg_end, rect.min.y),
            ],
            stroke,
        );
        x += step;
    }

    // Bottom edge
    x = rect.min.x;
    while x < rect.max.x {
        let seg_start = x;
        let seg_end = (x + dash_len).min(rect.max.x);
        painter.line_segment(
            [
                egui::pos2(seg_start, rect.max.y),
                egui::pos2(seg_end, rect.max.y),
            ],
            stroke,
        );
        x += step;
    }

    // Left edge
    let mut y = rect.min.y;
    while y < rect.max.y {
        let seg_start = y;
        let seg_end = (y + dash_len).min(rect.max.y);
        painter.line_segment(
            [
                egui::pos2(rect.min.x, seg_start),
                egui::pos2(rect.min.x, seg_end),
            ],
            stroke,
        );
        y += step;
    }

    // Right edge
    y = rect.min.y;
    while y < rect.max.y {
        let seg_start = y;
        let seg_end = (y + dash_len).min(rect.max.y);
        painter.line_segment(
            [
                egui::pos2(rect.max.x, seg_start),
                egui::pos2(rect.max.x, seg_end),
            ],
            stroke,
        );
        y += step;
    }
}

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

    let bg_main = Color32::from_rgb(10, 15, 26);
    let text_dark = Color32::from_rgb(224, 247, 255);
    let text_mid = Color32::from_rgb(142, 202, 230);
    let panel_bg = Color32::from_rgb(17, 24, 39);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg_main).inner_margin(8.0 * s))
        .show(ctx, |ui| {
            let full_rect = ui.max_rect();
            ui.painter().rect_filled(full_rect, 0.0, bg_main);

            // ── Top bar ─────────────────────────────────────────────────
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::Gallery;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.preview)
                            .size(20.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let reset_btn = egui::Button::new(
                        egui::RichText::new("Återställ")
                            .size(13.0 * s)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(50, 60, 80))
                    .rounding(Rounding::same(8.0 * s))
                    .min_size(Vec2::new(120.0 * s, 64.0 * s));
                    if ui.add(reset_btn).clicked() {
                        app.current_edit = crate::app::PhotoEdit::default();
                    }
                });
            });

            ui.add_space(4.0 * s);

            let paper_size = app.selected_product_size.clone();
            let frame_aspect = paper_size_aspect(&paper_size);
            let saving = app.save_in_progress;

            // Try full-res first, fall back to thumbnail while loading
            let tex_info = app.selected_texture(ctx).map(|t| (t.id(), t.size_vec2()));
            let tex_info = tex_info.or_else(|| {
                let path = app.photos.get(app.selected_photo)?.path.clone();
                app.texture_for(ctx, &path).map(|t| (t.id(), t.size_vec2()))
            });

            if let Some((tex_id, tex_size)) = tex_info {
                let available = ui.available_size();
                let bottom_h = 72.0 * s;
                let main_h = available.y - bottom_h;
                let left_w = (110.0 * s).min(available.x * 0.14);
                let right_w = (170.0 * s).min(available.x * 0.20);
                let gap = 10.0 * s;
                let img_max_w = (available.x - left_w - right_w - gap * 2.0).max(100.0);
                let img_max_h = main_h;

                // ── Main area: Left | Center (image) | Right ──────────────
                ui.horizontal(|ui| {
                    ui.set_height(main_h);

                    // LEFT PANEL
                    ui.allocate_ui_with_layout(
                        Vec2::new(left_w, main_h),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let panel_rect = ui.max_rect();
                            ui.painter().rect_filled(panel_rect, 8.0 * s, panel_bg);

                            ui.vertical_centered(|ui| {
                                ui.add_space(12.0 * s);

                                // Zoom out
                                let zoom_out = egui::Button::new(
                                    egui::RichText::new("−")
                                        .size(20.0 * s)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(50, 60, 80))
                                .rounding(Rounding::same(6.0 * s))
                                .min_size(Vec2::new(56.0 * s, 44.0 * s));
                                if ui.add(zoom_out).clicked()
                                    && !saving
                                    && app.current_edit.zoom > 1.0
                                {
                                    app.current_edit.zoom = (app.current_edit.zoom / 1.2).max(1.0);
                                }

                                ui.label(
                                    egui::RichText::new(format!("{:.1}x", app.current_edit.zoom))
                                        .size(14.0 * s)
                                        .color(text_mid),
                                );

                                // Zoom in
                                let zoom_in = egui::Button::new(
                                    egui::RichText::new("+")
                                        .size(20.0 * s)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(50, 60, 80))
                                .rounding(Rounding::same(6.0 * s))
                                .min_size(Vec2::new(56.0 * s, 44.0 * s));
                                if ui.add(zoom_in).clicked()
                                    && !saving
                                    && app.current_edit.zoom < 5.0
                                {
                                    app.current_edit.zoom = (app.current_edit.zoom * 1.2).min(5.0);
                                }

                                ui.add_space(16.0 * s);

                                // Rotate left
                                let rot_left = egui::Button::new(
                                    egui::RichText::new("↺")
                                        .size(20.0 * s)
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(50, 60, 80))
                                .rounding(Rounding::same(6.0 * s))
                                .min_size(Vec2::new(56.0 * s, 44.0 * s));
                                if ui.add(rot_left).clicked() && !saving {
                                    app.current_edit.rotation =
                                        (app.current_edit.rotation + 270) % 360;
                                }

                                ui.add_space(6.0 * s);

                                // Rotate right
                                let rot_right = egui::Button::new(
                                    egui::RichText::new("↻")
                                        .size(20.0 * s)
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(50, 60, 80))
                                .rounding(Rounding::same(6.0 * s))
                                .min_size(Vec2::new(56.0 * s, 44.0 * s));
                                if ui.add(rot_right).clicked() && !saving {
                                    app.current_edit.rotation =
                                        (app.current_edit.rotation + 90) % 360;
                                }

                                ui.add_space(16.0 * s);

                                // B&W toggle
                                let bw_text = if app.current_edit.grayscale {
                                    "B/W ✓"
                                } else {
                                    "B/W"
                                };
                                let bw_btn = egui::Button::new(
                                    egui::RichText::new(bw_text)
                                        .size(12.0 * s)
                                        .color(Color32::WHITE),
                                )
                                .fill(if app.current_edit.grayscale {
                                    Color32::from_rgb(0, 229, 255)
                                } else {
                                    Color32::from_rgb(50, 60, 80)
                                })
                                .rounding(Rounding::same(6.0 * s))
                                .min_size(Vec2::new(80.0 * s, 44.0 * s));
                                if ui.add(bw_btn).clicked() && !saving {
                                    app.current_edit.grayscale = !app.current_edit.grayscale;
                                }
                            });
                        },
                    );

                    ui.add_space(gap);

                    // CENTER: Image (centered vertically)
                    ui.allocate_ui_with_layout(
                        Vec2::new(img_max_w, main_h),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let top_space = (main_h
                                - img_max_w.min(img_max_h * frame_aspect) / frame_aspect)
                                * 0.5;
                            ui.add_space(top_space.max(0.0));

                            let img_w = img_max_w.min(img_max_h * frame_aspect);
                            let img_h = img_w / frame_aspect;
                            let (img_rect, img_response) = ui
                                .allocate_exact_size(Vec2::new(img_w, img_h), egui::Sense::drag());

                            // Handle drag-to-pan
                            if img_response.dragged() && !saving {
                                let delta = img_response.drag_delta();
                                let sensitivity = 2.0 / app.current_edit.zoom.max(1.0);
                                app.current_edit.pan_x -=
                                    delta.x / (img_w.max(1.0) * 0.5) * sensitivity;
                                app.current_edit.pan_y -=
                                    delta.y / (img_h.max(1.0) * 0.5) * sensitivity;
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
                            painter.rect_filled(img_rect, 4.0 * s, Color32::from_rgb(0, 229, 255));
                            draw_editable_image(painter, tex_id, img_rect, uvs, Color32::WHITE);
                            draw_dashed_rect(
                                painter,
                                img_rect,
                                Stroke::new(2.5 * s, Color32::from_rgb(255, 51, 102)),
                                10.0 * s,
                                6.0 * s,
                            );

                            // Text overlay preview
                            if !app.current_edit.text_overlay.is_empty() {
                                let text_pos = egui::pos2(
                                    img_rect.min.x + app.current_edit.text_x * img_rect.width(),
                                    img_rect.min.y + app.current_edit.text_y * img_rect.height(),
                                );
                                let font_size = (app.current_edit.text_size as f32)
                                    .min(img_h * 0.15)
                                    .max(12.0)
                                    * s;
                                for dx in [-1.0, 1.0] {
                                    for dy in [-1.0, 1.0] {
                                        painter.text(
                                            text_pos + Vec2::new(dx * 2.0 * s, dy * 2.0 * s),
                                            egui::Align2::CENTER_CENTER,
                                            &app.current_edit.text_overlay,
                                            egui::FontId::new(
                                                font_size,
                                                egui::FontFamily::Proportional,
                                            ),
                                            Color32::from_rgb(0, 0, 0),
                                        );
                                    }
                                }
                                painter.text(
                                    text_pos,
                                    egui::Align2::CENTER_CENTER,
                                    &app.current_edit.text_overlay,
                                    egui::FontId::new(font_size, egui::FontFamily::Proportional),
                                    Color32::WHITE,
                                );
                            }
                        },
                    );

                    ui.add_space(gap);

                    // RIGHT PANEL
                    ui.allocate_ui_with_layout(
                        Vec2::new(right_w, main_h),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let panel_rect = ui.max_rect();
                            ui.painter().rect_filled(panel_rect, 8.0 * s, panel_bg);

                            ui.vertical_centered(|ui| {
                                ui.add_space(12.0 * s);

                                ui.label(
                                    egui::RichText::new("Text:").size(14.0 * s).color(text_mid),
                                );
                                ui.add_space(6.0 * s);

                                let text_edit =
                                    egui::TextEdit::singleline(&mut app.current_edit.text_overlay)
                                        .font(egui::TextStyle::Body)
                                        .hint_text("Skriv text...")
                                        .desired_width(140.0 * s);
                                ui.add(text_edit);

                                ui.add_space(12.0 * s);

                                // Arrow pad in cross layout
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(44.0 * s);
                                        let up_btn = egui::Button::new(
                                            egui::RichText::new("▲")
                                                .size(16.0 * s)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(50, 60, 80))
                                        .rounding(Rounding::same(6.0 * s))
                                        .min_size(Vec2::new(44.0 * s, 40.0 * s));
                                        if ui.add(up_btn).clicked() && !saving {
                                            app.current_edit.text_y =
                                                (app.current_edit.text_y - 0.05).max(0.05);
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        let left_btn = egui::Button::new(
                                            egui::RichText::new("◀")
                                                .size(16.0 * s)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(50, 60, 80))
                                        .rounding(Rounding::same(6.0 * s))
                                        .min_size(Vec2::new(44.0 * s, 40.0 * s));
                                        if ui.add(left_btn).clicked() && !saving {
                                            app.current_edit.text_x =
                                                (app.current_edit.text_x - 0.05).max(0.05);
                                        }
                                        ui.add_space(4.0 * s);
                                        let down_btn = egui::Button::new(
                                            egui::RichText::new("▼")
                                                .size(16.0 * s)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(50, 60, 80))
                                        .rounding(Rounding::same(6.0 * s))
                                        .min_size(Vec2::new(44.0 * s, 40.0 * s));
                                        if ui.add(down_btn).clicked() && !saving {
                                            app.current_edit.text_y =
                                                (app.current_edit.text_y + 0.05).min(0.95);
                                        }
                                        ui.add_space(4.0 * s);
                                        let right_btn = egui::Button::new(
                                            egui::RichText::new("▶")
                                                .size(16.0 * s)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(50, 60, 80))
                                        .rounding(Rounding::same(6.0 * s))
                                        .min_size(Vec2::new(44.0 * s, 40.0 * s));
                                        if ui.add(right_btn).clicked() && !saving {
                                            app.current_edit.text_x =
                                                (app.current_edit.text_x + 0.05).min(0.95);
                                        }
                                    });
                                });

                                ui.add_space(16.0 * s);

                                // Text size
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Storlek: {}",
                                        app.current_edit.text_size
                                    ))
                                    .size(13.0 * s)
                                    .color(text_mid),
                                );
                                ui.add_space(4.0 * s);
                                ui.horizontal(|ui| {
                                    ui.add_space(12.0 * s);
                                    let txt_minus = egui::Button::new(
                                        egui::RichText::new("−")
                                            .size(14.0 * s)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(50, 60, 80))
                                    .rounding(Rounding::same(6.0 * s))
                                    .min_size(Vec2::new(48.0 * s, 40.0 * s));
                                    if ui.add(txt_minus).clicked()
                                        && !saving
                                        && app.current_edit.text_size > 12
                                    {
                                        app.current_edit.text_size -= 4;
                                    }
                                    ui.add_space(4.0 * s);
                                    let txt_plus = egui::Button::new(
                                        egui::RichText::new("+")
                                            .size(16.0 * s)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(50, 60, 80))
                                    .rounding(Rounding::same(6.0 * s))
                                    .min_size(Vec2::new(56.0 * s, 48.0 * s));
                                    if ui.add(txt_plus).clicked()
                                        && !saving
                                        && app.current_edit.text_size < 200
                                    {
                                        app.current_edit.text_size += 4;
                                    }
                                });
                            });
                        },
                    );
                });

                // ── Bottom bar ─────────────────────────────────────────────
                ui.add_space(4.0 * s);
                ui.horizontal(|ui| {
                    let btn_h = 56.0 * s;

                    // Left: paper size info
                    ui.vertical(|ui| {
                        ui.set_min_size(Vec2::new(160.0 * s, btn_h));
                        ui.label(
                            egui::RichText::new(format!(
                                "Format: {} · {}",
                                paper_size,
                                app.printer_for_current_size()
                                    .unwrap_or(pack.no_printer_for_size)
                            ))
                            .size(13.0 * s)
                            .color(text_mid),
                        );
                    });

                    // Right: actions
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if saving {
                            ui.horizontal(|ui| {
                                let time = ui.ctx().input(|i| i.time);
                                let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                                ui.label(
                                    egui::RichText::new(spinner)
                                        .size(28.0 * s)
                                        .color(Color32::from_rgb(0, 229, 255)),
                                );
                                ui.add_space(8.0 * s);
                                ui.label(
                                    egui::RichText::new("Sparar...")
                                        .size(14.0 * s)
                                        .strong()
                                        .color(text_dark),
                                );
                            });
                        } else {
                            let back_btn = egui::Button::new(
                                egui::RichText::new("Tillbaka")
                                    .size(13.0 * s)
                                    .color(text_mid),
                            )
                            .fill(Color32::from_rgb(30, 40, 60))
                            .rounding(Rounding::same(10.0 * s))
                            .min_size(Vec2::new(120.0 * s, btn_h));
                            if ui.add(back_btn).clicked() {
                                app.current_edit = crate::app::PhotoEdit::default();
                                app.screen = AppScreen::Gallery;
                            }

                            ui.add_space(12.0 * s);

                            let save_btn = egui::Button::new(
                                egui::RichText::new("💾 Spara")
                                    .size(16.0 * s)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(0, 229, 255))
                            .rounding(Rounding::same(12.0 * s))
                            .min_size(Vec2::new(180.0 * s, btn_h));
                            if ui.add(save_btn).clicked() {
                                app.save_current_edit();
                            }
                        }
                    });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("—").size(20.0 * s).color(text_mid));
                });
            }
        });
}

// =============================================================================
// QUEUE / CART SCREEN
// =============================================================================
fn draw_queue(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    let bg_main = Color32::from_rgb(10, 15, 26);
    let text_dark = Color32::from_rgb(224, 247, 255);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg_main).inner_margin(12.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

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
                            .color(text_dark),
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
                            .color(Color32::from_rgb(180, 200, 220)),
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
                                .fill(Color32::from_rgb(13, 19, 33))
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
                                                    .color(Color32::from_rgb(130, 150, 170)),
                                            );
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let del_btn = ui.add(
                                                    egui::Button::new(
                                                        egui::RichText::new("×")
                                                            .size(18.0 * s)
                                                            .color(Color32::from_rgb(255, 51, 102)),
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
                                .color(Color32::from_rgb(0, 229, 255)),
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
                    .fill(Color32::from_rgb(0, 229, 255))
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
                    .fill(Color32::from_rgb(0, 153, 170))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 64.0 * s));
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
    let text_dark = Color32::from_rgb(224, 247, 255);
    let text_mid = Color32::from_rgb(142, 202, 230);
    let blue_accent = Color32::from_rgb(0, 229, 255);

    // Keep spinner animating
    ctx.request_repaint();
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(10, 15, 26))
                .inner_margin(20.0 * s),
        )
        .show(ctx, |ui| {
            // Push content to vertical center
            let est_content_h = 340.0 * s;
            let top_pad = (ui.available_height() - est_content_h) * 0.5;
            ui.add_space(top_pad.max(0.0));

            ui.vertical_centered(|ui| {
                let time = ui.ctx().input(|i| i.time);

                // ── Big file counter ──────────────────────────────────────
                if let Some((current, total)) = app.import_progress {
                    if total > 0 {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}", current))
                                    .size(72.0 * s)
                                    .strong()
                                    .color(blue_accent),
                            );
                            ui.label(
                                egui::RichText::new(format!(" / {}", total))
                                    .size(28.0 * s)
                                    .color(text_mid),
                            );
                        });
                    } else {
                        let spinner = ["◆", "▲", "◆", "▼"][(time * 3.0) as usize % 4];
                        ui.label(
                            egui::RichText::new(spinner)
                                .size(56.0 * s)
                                .color(blue_accent),
                        );
                    }
                } else {
                    let spinner = ["◆", "▲", "◆", "▼"][(time * 3.0) as usize % 4];
                    ui.label(
                        egui::RichText::new(spinner)
                            .size(56.0 * s)
                            .color(blue_accent),
                    );
                }

                ui.add_space(16.0 * s);

                // ── Current filename ──────────────────────────────────────
                if !app.import_current_file.is_empty() {
                    let file_label = if app.import_current_file.len() > 30 {
                        format!(
                            "…{}",
                            &app.import_current_file[app.import_current_file.len() - 30..]
                        )
                    } else {
                        app.import_current_file.clone()
                    };
                    ui.label(
                        egui::RichText::new(format!("📁 {}", file_label))
                            .size(16.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                }

                ui.add_space(6.0 * s);

                ui.label(
                    egui::RichText::new(pack.importing)
                        .size(18.0 * s)
                        .color(text_mid),
                );

                ui.add_space(24.0 * s);

                // ── Progress bar ──────────────────────────────────────────
                let bar_width = ui.available_width().min(480.0 * s);
                let bar_height = 28.0 * s;
                let bar_rect = egui::Rect::from_center_size(
                    ui.cursor().center_top() + egui::vec2(0.0, bar_height / 2.0),
                    egui::vec2(bar_width, bar_height),
                );
                ui.painter()
                    .rect_filled(bar_rect, 10.0 * s, Color32::from_rgb(30, 40, 60));

                let fill_w = match app.import_progress {
                    Some((current, total)) if total > 0 => {
                        bar_width * (current as f32 / total as f32).min(1.0)
                    }
                    _ => {
                        // Indeterminate: bounce back and forth
                        let t = ((time as f32) * 1.5).sin() * 0.5 + 0.5;
                        bar_width * 0.3_f32 + (bar_width * 0.7_f32) * t
                    }
                };
                let fill_rect =
                    egui::Rect::from_min_size(bar_rect.min, egui::vec2(fill_w, bar_height));
                ui.painter().rect_filled(fill_rect, 10.0 * s, blue_accent);
                ui.painter().rect_stroke(
                    bar_rect,
                    10.0 * s,
                    Stroke::new(1.5 * s, Color32::from_rgb(50, 60, 80)),
                );
                ui.add_space(bar_height + 16.0 * s);

                // ── Elapsed time + estimate ───────────────────────────────
                let elapsed_secs = app
                    .import_start_time
                    .map(|t| t.elapsed().as_secs())
                    .unwrap_or(0);
                if elapsed_secs > 0 {
                    let time_text = if let Some((current, total)) = app.import_progress {
                        if current > 0 && total > 0 && elapsed_secs > 0 {
                            let rate = current as f64 / elapsed_secs as f64;
                            let remaining = ((total - current) as f64 / rate).max(0.0) as u64;
                            format!("⏱ {}s  ·  ca {}s kvar", elapsed_secs, remaining)
                        } else {
                            format!("⏱ {}s", elapsed_secs)
                        }
                    } else {
                        format!("⏱ {}s", elapsed_secs)
                    };
                    ui.label(
                        egui::RichText::new(time_text)
                            .size(13.0 * s)
                            .color(text_mid),
                    );
                }
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
                .fill(Color32::from_rgb(10, 15, 26))
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
                        .color(Color32::from_rgb(0, 229, 255)),
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
                        .color(Color32::from_rgb(142, 202, 230)),
                );
                ui.add_space(24.0 * s);

                let btn_width = ui.available_width().min(320.0);

                big_button(
                    ui,
                    pack.mobile_android,
                    Color32::from_rgb(0, 255, 170),
                    btn_width,
                    s,
                    || {
                        app.start_phone_flow(crate::app::PhoneType::Android);
                    },
                );
                big_button(
                    ui,
                    pack.mobile_iphone,
                    Color32::from_rgb(255, 51, 102),
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
                        .color(Color32::from_rgb(200, 220, 240)),
                )
                .fill(Color32::from_rgb(13, 19, 33))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 64.0 * s));
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
        .frame(Frame::none().fill(Color32::from_rgb(10, 15, 26)).inner_margin(16.0 * s))
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
                        .color(Color32::from_rgb(0, 229, 255)),
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
                        .color(Color32::from_rgb(180, 200, 220)),
                );
                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new(pack.connect_phone_hint)
                        .size(12.0 * s)
                        .color(Color32::from_rgb(120, 140, 160)),
                );

                ui.add_space(24.0 * s);

                // Animated spinner
                let time = ui.ctx().input(|i| i.time);
                let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                ui.label(
                    egui::RichText::new(spinner)
                        .size(32.0 * s)
                        .color(Color32::from_rgb(0, 229, 255)),
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
                            .color(Color32::from_rgb(120, 140, 160)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!("{} ({}s)", pack.connect_phone_searching, elapsed))
                            .size(14.0 * s)
                            .strong()
                            .color(Color32::from_rgb(0, 229, 255)),
                    );
                }

                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new("Anslut telefonen och tryck Tillåt på skärmen. Se till att telefonen är i filöverföringsläge.")
                        .size(11.0 * s)
                        .color(Color32::from_rgb(180, 200, 220)),
                );

                ui.add_space(16.0 * s);

                let btn_width = ui.available_width().min(280.0 * s);
                let search_btn = egui::Button::new(
                    egui::RichText::new(pack.connect_phone_search)
                        .size(14.0 * s)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 229, 255))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 64.0 * s));
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
                .fill(Color32::from_rgb(10, 15, 26))
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
                        .color(Color32::from_rgb(142, 202, 230)),
                );
                ui.add_space(8.0 * s);

                // ── WiFi info box ─────────────────────────────────────────────
                egui::Frame::none()
                    .fill(Color32::from_rgb(17, 24, 39))
                    .stroke(Stroke::new(1.0 * s, Color32::from_rgb(30, 40, 60)))
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
                                .color(Color32::from_rgb(20, 30, 50)),
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
                    .color(Color32::from_rgb(120, 140, 160)),
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
                                    .color(Color32::from_rgb(0, 229, 255)),
                            );
                            ui.add_space(4.0 * s);
                            if let Some(texture) = app.wifi_qr_texture(ctx) {
                                ui.image((texture.id(), Vec2::new(qr_size, qr_size)));
                            } else {
                                ui.label(
                                    egui::RichText::new("…")
                                        .size(12.0 * s)
                                        .color(Color32::from_rgb(180, 200, 220)),
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
                                    .color(Color32::from_rgb(0, 229, 255)),
                            );
                            ui.add_space(4.0 * s);
                            if let Some(texture) = app.qr_texture(ctx) {
                                ui.image((texture.id(), Vec2::new(qr_size, qr_size)));
                            } else {
                                ui.label(
                                    egui::RichText::new("…")
                                        .size(12.0 * s)
                                        .color(Color32::from_rgb(180, 200, 220)),
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
                            .color(Color32::from_rgb(120, 140, 160)),
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
                .fill(Color32::from_rgb(0, 229, 255))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 64.0 * s));
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
                .fill(Color32::from_rgb(0, 229, 255))
                .rounding(Rounding::same(10.0 * s))
                .min_size(Vec2::new(btn_width, 64.0 * s));
                if ui.add(done_btn).clicked() {
                    app.screen = AppScreen::Gallery;
                }

                ui.add_space(6.0 * s);

                // Small secondary link for cable connection
                let cable_btn = egui::Button::new(
                    egui::RichText::new(pack.mobile_cable_connect)
                        .size(11.0 * s)
                        .color(Color32::from_rgb(120, 140, 160)),
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
// GOOGLE PHOTOS AUTH SCREEN — QR code login
// =============================================================================
fn draw_google_drive_auth(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    egui::CentralPanel::default()
        .frame(
            Frame::none()
                .fill(Color32::from_rgb(10, 15, 26))
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
                        .color(Color32::from_rgb(255, 51, 102)),
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
                        .color(Color32::from_rgb(142, 202, 230)),
                );
                ui.add_space(20.0 * s);

                if app.pkce_state.is_some() {
                    ui.label(
                        egui::RichText::new("🌐 En webbläsare har öppnats")
                            .size(16.0 * s)
                            .color(Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add_space(8.0 * s);
                    ui.label(
                        egui::RichText::new("Logga in på Google i webbläsaren och godkänn åtkomst")
                            .size(14.0 * s)
                            .color(Color32::from_rgb(180, 200, 220)),
                    );
                    ui.add_space(8.0 * s);
                    ui.label(
                        egui::RichText::new("(Stäng webbläsaren när du är klar)")
                            .size(12.0 * s)
                            .color(Color32::from_rgb(180, 200, 220)),
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
                            .fill(Color32::from_rgb(13, 19, 33))
                            .rounding(Rounding::same(8.0 * s))
                            .min_size(Vec2::new(220.0 * s, 64.0 * s)),
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
                            .color(Color32::from_rgb(255, 51, 102)),
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
                .fill(Color32::from_rgb(10, 15, 26))
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
                            .color(Color32::from_rgb(120, 140, 160)),
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
                                        Color32::from_rgb(0, 229, 255)
                                    } else {
                                        Color32::from_rgb(13, 19, 33)
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
                                                        .color(Color32::from_rgb(200, 220, 240)),
                                                );

                                                // Checkmark if selected
                                                if is_selected {
                                                    ui.label(
                                                        egui::RichText::new("✅").size(14.0 * s),
                                                    );
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
                    .fill(Color32::from_rgb(0, 229, 255))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 64.0 * s));

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
                .fill(Color32::from_rgb(10, 15, 26))
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
                        .color(Color32::from_rgb(200, 220, 240)),
                )
                .fill(Color32::from_rgb(13, 19, 33))
                .rounding(Rounding::same(8.0 * s))
                .min_size(Vec2::new(ui.available_width() * 0.45, 56.0 * s));
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
                .fill(Color32::from_rgb(255, 51, 102))
                .rounding(Rounding::same(8.0 * s))
                .min_size(Vec2::new(ui.available_width(), 56.0 * s));
                ui.add(photos_btn);
            });

            ui.add_space(8.0 * s);

            if app.photo_items.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Inga bilder i Google Foto")
                            .size(13.0 * s)
                            .color(Color32::from_rgb(120, 140, 160)),
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
                                Color32::from_rgb(255, 51, 102)
                            } else {
                                Color32::from_rgb(13, 19, 33)
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
                    .fill(Color32::from_rgb(255, 51, 102))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 64.0 * s));

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
                    .fill(Color32::from_rgba_unmultiplied(10, 15, 26, alpha))
                    .rounding(Rounding::same(10.0 * s))
                    .stroke(Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(0, 229, 255, alpha),
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
                                .color(Color32::from_rgb(0, 229, 255)),
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
    let bg_main = Color32::from_rgb(10, 15, 26);
    let text_dark = Color32::from_rgb(224, 247, 255);
    let blue_accent = Color32::from_rgb(0, 229, 255);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg_main).inner_margin(20.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("🖨").size(64.0 * s).color(blue_accent));
                    ui.add_space(20.0 * s);
                    ui.label(
                        egui::RichText::new(pack.print_done_title)
                            .size(26.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                    ui.add_space(12.0 * s);
                    ui.label(
                        egui::RichText::new(pack.print_done_subtitle)
                            .size(14.0 * s)
                            .color(Color32::from_rgb(40, 50, 70)),
                    );
                    ui.add_space(40.0 * s);

                    let time = ui.ctx().input(|i| i.time);
                    let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                    ui.label(
                        egui::RichText::new(spinner)
                            .size(32.0 * s)
                            .color(blue_accent),
                    );
                    ui.add_space(8.0 * s);
                    ui.label(
                        egui::RichText::new(format!("{:.0}s", app.print_done_timer))
                            .size(12.0 * s)
                            .color(Color32::from_rgb(40, 50, 70)),
                    );
                });
            });
        });
}

fn draw_print_progress(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    let bg_main = Color32::from_rgb(10, 15, 26);
    let text_dark = Color32::from_rgb(224, 247, 255);
    let text_mid = Color32::from_rgb(142, 202, 230);
    let blue_accent = Color32::from_rgb(0, 229, 255);
    let green_ok = Color32::from_rgb(0, 255, 170);
    let red_err = Color32::from_rgb(255, 51, 102);

    let total = app.print_progress_total.max(1);
    let done = app.print_progress_done;
    let failed = app.print_progress_failed;
    let progress = (done + failed) as f32 / total as f32;

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg_main).inner_margin(24.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("🖨").size(56.0 * s).color(blue_accent));
                    ui.add_space(16.0 * s);
                    ui.label(
                        egui::RichText::new(pack.print_progress_title)
                            .size(28.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                    ui.add_space(8.0 * s);
                    ui.label(
                        egui::RichText::new(pack.print_progress_subtitle)
                            .size(16.0 * s)
                            .color(text_mid),
                    );

                    ui.add_space(32.0 * s);

                    // Progress bar
                    let bar_width = ui.available_width().min(400.0 * s);
                    let bar_height = 24.0 * s;
                    let bar_rect = egui::Rect::from_center_size(
                        ui.cursor().center_top() + egui::vec2(0.0, bar_height / 2.0),
                        egui::vec2(bar_width, bar_height),
                    );
                    ui.painter()
                        .rect_filled(bar_rect, 8.0 * s, Color32::from_rgb(30, 40, 60));
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        egui::vec2(bar_width * progress, bar_height),
                    );
                    ui.painter().rect_filled(fill_rect, 8.0 * s, blue_accent);
                    ui.painter().rect_stroke(
                        bar_rect,
                        8.0 * s,
                        Stroke::new(1.0 * s, Color32::from_rgb(50, 60, 80)),
                    );
                    ui.add_space(bar_height + 12.0 * s);

                    ui.label(
                        egui::RichText::new(format!("{} / {}", done + failed, total))
                            .size(22.0 * s)
                            .strong()
                            .color(text_dark),
                    );

                    ui.add_space(24.0 * s);

                    // Job status list
                    for printer in &app.printers {
                        for job in printer.jobs() {
                            let (icon, color, status_text) = match &job.status {
                                crate::printer::JobStatus::Queued => {
                                    ("⏳", text_mid, pack.print_progress_queued)
                                }
                                crate::printer::JobStatus::Printing => {
                                    ("🖨", blue_accent, pack.print_progress_printing)
                                }
                                crate::printer::JobStatus::Done => {
                                    ("✅", green_ok, pack.print_progress_done)
                                }
                                crate::printer::JobStatus::Failed(_) => {
                                    ("❌", red_err, pack.print_progress_failed)
                                }
                            };
                            let file = std::path::Path::new(&job.photo_path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy();
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}  {} — {}",
                                    icon, file, status_text
                                ))
                                .size(14.0 * s)
                                .color(color),
                            );
                        }
                    }
                });
            });
        });
}

fn draw_thank_you(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    let bg_main = Color32::from_rgb(10, 15, 26);
    let text_dark = Color32::from_rgb(224, 247, 255);
    let blue_accent = Color32::from_rgb(0, 229, 255);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg_main).inner_margin(24.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("👋").size(72.0 * s).color(blue_accent));
                    ui.add_space(24.0 * s);
                    ui.label(
                        egui::RichText::new(pack.thank_you_title)
                            .size(36.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                    ui.add_space(12.0 * s);
                    ui.label(
                        egui::RichText::new(pack.thank_you_subtitle)
                            .size(18.0 * s)
                            .color(Color32::from_rgb(142, 202, 230)),
                    );
                    ui.add_space(32.0 * s);
                    ui.label(
                        egui::RichText::new(format!("{:.0}s", app.thank_you_timer))
                            .size(14.0 * s)
                            .color(Color32::from_rgb(60, 80, 110)),
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

    let bg_main = Color32::from_rgb(10, 15, 26);
    let text_dark = Color32::from_rgb(224, 247, 255);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg_main).inner_margin(20.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

            ui.vertical_centered(|ui| {
                ui.add_space(24.0 * s);
                ui.label(egui::RichText::new("💳").size(56.0 * s));
                ui.add_space(16.0 * s);
                ui.label(
                    egui::RichText::new(pack.payment_title)
                        .size(26.0 * s)
                        .strong()
                        .color(text_dark),
                );
                ui.add_space(8.0 * s);
                ui.label(
                    egui::RichText::new(format!("{}: {:.0} kr", pack.total_label, total))
                        .size(22.0 * s)
                        .strong()
                        .color(Color32::from_rgb(0, 255, 170)),
                );
                ui.add_space(4.0 * s);
                ui.label(
                    egui::RichText::new(format!("{}: {}", pack.print_queue, count))
                        .size(14.0 * s)
                        .color(Color32::from_rgb(100, 160, 220)),
                );

                ui.add_space(32.0 * s);

                let btn_width = ui.available_width().min(320.0);

                if ui
                    .add_sized(
                        Vec2::new(btn_width, 56.0 * s),
                        egui::Button::new(
                            egui::RichText::new(pack.payment_store)
                                .size(18.0 * s)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(0, 229, 255))
                        .rounding(Rounding::same(10.0 * s)),
                    )
                    .clicked()
                {
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
                        .fill(Color32::from_rgb(100, 60, 220))
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
                .fill(Color32::from_rgb(10, 15, 26))
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
                        .fill(Color32::from_rgb(50, 60, 80))
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
                                .fill(Color32::from_rgb(0, 229, 255))
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
                        .color(Color32::from_rgb(120, 140, 160)),
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
                                    Color32::from_rgb(0, 229, 255)
                                } else {
                                    Color32::from_rgb(17, 24, 39)
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
                                        Color32::from_rgb(30, 40, 60),
                                    );
                                    let galley = painter.layout(
                                        "🖼".to_string(),
                                        egui::FontId::new(36.0 * s, egui::FontFamily::Proportional),
                                        Color32::from_rgb(180, 200, 220),
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
                                        Stroke::new(3.5 * s, Color32::from_rgb(0, 229, 255)),
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
                                        Color32::from_rgb(0, 229, 255),
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
                        .color(Color32::from_rgb(200, 220, 240)),
                    )
                    .fill(Color32::from_rgb(17, 24, 39))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 64.0 * s));
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

// =============================================================================
// LAYOUT SELECT SCREEN
// =============================================================================
fn draw_layout_select(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(24.0 * s))
        .show(ctx, |ui| {
            // Soft solid background
            let rect = ui.max_rect();
            ui.painter()
                .rect_filled(rect, 0.0, Color32::from_rgb(10, 15, 26));

            // Top bar
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.layout_select_title)
                            .size(22.0 * s)
                            .strong()
                            .color(Color32::from_rgb(30, 40, 60)),
                    );
                });
            });

            ui.add_space(20.0 * s);

            let layouts = crate::collage::all_layouts();
            let card_width = (ui.available_width() / 2.0 - 16.0 * s).max(200.0 * s);
            let card_height = 180.0 * s;

            ui.vertical_centered(|ui| {
                for layout in layouts {
                    let rects = layout.slot_rects();
                    let card = Frame::none()
                        .fill(Color32::from_rgb(17, 24, 39))
                        .rounding(Rounding::same(10.0 * s))
                        .inner_margin(12.0 * s)
                        .stroke(Stroke::new(1.0 * s, Color32::from_rgb(30, 40, 60)));

                    let response = card
                        .show(ui, |ui| {
                            ui.set_width(card_width);
                            ui.set_min_height(card_height);
                            ui.horizontal(|ui| {
                                // Preview canvas
                                let preview_size = 120.0 * s;
                                let preview_rect =
                                    ui.allocate_space(Vec2::new(preview_size, preview_size)).1;
                                ui.painter().rect_filled(
                                    preview_rect,
                                    Rounding::same(6.0 * s),
                                    Color32::from_rgb(17, 24, 39),
                                );
                                for (nx, ny, nw, nh) in rects.iter().copied() {
                                    let slot_rect = egui::Rect::from_min_size(
                                        egui::pos2(
                                            preview_rect.left() + nx * preview_size,
                                            preview_rect.top() + ny * preview_size,
                                        ),
                                        Vec2::new(nw * preview_size, nh * preview_size),
                                    );
                                    // inset slightly for visual separation
                                    let inset = 2.0 * s;
                                    let slot_rect = slot_rect.shrink(inset);
                                    ui.painter().rect_filled(
                                        slot_rect,
                                        Rounding::same(3.0 * s),
                                        Color32::from_rgb(30, 40, 60),
                                    );
                                }

                                ui.add_space(16.0 * s);

                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(layout.name())
                                            .size(16.0 * s)
                                            .strong()
                                            .color(Color32::from_rgb(220, 240, 255)),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} {} {}",
                                            pack.select_photos_hint,
                                            layout.photo_count(),
                                            pack.selected_count
                                        ))
                                        .size(12.0 * s)
                                        .color(Color32::from_rgb(80, 90, 110)),
                                    );
                                });
                            });
                        })
                        .response;

                    if response.clicked() {
                        app.selected_layout = Some(layout);
                        app.collage_photo_indices.clear();
                        app.screen = AppScreen::CollageEditor;
                    }
                    ui.add_space(12.0 * s);
                }
            });
        });
}

// =============================================================================
// COLLAGE EDITOR SCREEN
// =============================================================================
fn draw_collage_editor(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);

    let layout = match app.selected_layout {
        Some(l) => l,
        None => {
            app.screen = AppScreen::LayoutSelect;
            return;
        }
    };
    let required = layout.photo_count();

    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(12.0 * s))
        .show(ctx, |ui| {
            // Soft solid background
            let rect = ui.max_rect();
            ui.painter()
                .rect_filled(rect, 0.0, Color32::from_rgb(10, 15, 26));

            // Top bar
            ui.horizontal(|ui| {
                if nav_button(ui, "<", s).clicked() {
                    app.screen = AppScreen::LayoutSelect;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.collage_editor_title)
                            .size(20.0 * s)
                            .strong()
                            .color(Color32::from_rgb(30, 40, 60)),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let selected = app.collage_photo_indices.len();
                    let count_text = format!("{} / {}", selected, required);
                    ui.label(
                        egui::RichText::new(count_text)
                            .size(14.0 * s)
                            .strong()
                            .color(Color32::from_rgb(0, 229, 255)),
                    );
                });
            });

            ui.add_space(8.0 * s);

            if app.photos.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(pack.no_photos)
                            .size(16.0 * s)
                            .color(Color32::from_rgb(80, 90, 110)),
                    );
                });
                return;
            }

            // Grid layout (same math as Gallery)
            let available_width = ui.available_width();
            let thumb_base = 130.0;
            let spacing = 12.0;
            let cell_size = thumb_base * s + spacing * s;
            let cols = ((available_width / cell_size).floor() as usize).max(3);
            let thumb_size = thumb_base * s;
            let photo_count = app.photos.len();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for row_start in (0..photo_count).step_by(cols) {
                        ui.horizontal(|ui| {
                            for i in row_start..(row_start + cols).min(photo_count) {
                                let photo_path = app.photos[i].path.clone();
                                let photo_file_name = app.photos[i].file_name.clone();
                                let photo_date = app.photos[i].date_taken;
                                let is_selected = app.collage_photo_indices.contains(&i);

                                let bg = if is_selected {
                                    Color32::from_rgb(0, 229, 255)
                                } else {
                                    Color32::from_rgb(17, 24, 39)
                                };

                                let stroke_color = if is_selected {
                                    Color32::from_rgb(0, 255, 170)
                                } else {
                                    Color32::from_rgb(80, 90, 110)
                                };

                                let frame = Frame::none()
                                    .fill(bg)
                                    .rounding(Rounding::same(8.0 * s))
                                    .inner_margin(6.0 * s)
                                    .stroke(Stroke::new(1.5 * s, stroke_color));

                                let response = frame
                                    .show(ui, |ui| {
                                        ui.set_width(thumb_size);
                                        ui.vertical_centered(|ui| {
                                            if let Some(texture) = app.texture_for(ctx, &photo_path)
                                            {
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

                                            ui.label(
                                                egui::RichText::new(&photo_file_name)
                                                    .size(10.0 * s)
                                                    .strong()
                                                    .color(if is_selected {
                                                        Color32::WHITE
                                                    } else {
                                                        Color32::from_rgb(40, 50, 65)
                                                    }),
                                            );
                                            ui.label(
                                                egui::RichText::new(
                                                    crate::gallery::format_date_short(photo_date),
                                                )
                                                .size(9.0 * s)
                                                .color(if is_selected {
                                                    Color32::from_rgb(17, 24, 39)
                                                } else {
                                                    Color32::from_rgb(80, 90, 110)
                                                }),
                                            );
                                            if is_selected {
                                                ui.label(
                                                    egui::RichText::new("✓")
                                                        .size(14.0 * s)
                                                        .color(Color32::from_rgb(0, 255, 170)),
                                                );
                                            }
                                        });
                                    })
                                    .response;

                                if response.clicked() {
                                    if is_selected {
                                        app.collage_photo_indices.retain(|&idx| idx != i);
                                    } else if app.collage_photo_indices.len() < required {
                                        app.collage_photo_indices.push(i);
                                    }
                                }
                            }
                        });
                        ui.add_space(8.0 * s);
                    }
                });

            // Bottom create button
            if app.collage_photo_indices.len() == required {
                ui.add_space(12.0 * s);
                ui.vertical_centered(|ui| {
                    let btn_width = ui.available_width().min(400.0 * s);
                    let create_btn = egui::Button::new(
                        egui::RichText::new(pack.create_collage)
                            .size(16.0 * s)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 229, 255))
                    .rounding(Rounding::same(10.0 * s))
                    .min_size(Vec2::new(btn_width, 72.0 * s));
                    if ui.add(create_btn).clicked() {
                        // Render collage
                        let size = app.selected_product_size.clone();
                        let layout = app.selected_layout.unwrap();
                        let paths: Vec<std::path::PathBuf> = app
                            .collage_photo_indices
                            .iter()
                            .map(|&idx| app.photos[idx].path.clone())
                            .collect();
                        let temp_dir = &app.config.temp_directory;
                        let output_path = temp_dir.join(format!(
                            "collage_{}_{}.jpg",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            fastrand::u32(0..10000)
                        ));
                        let path_refs: Vec<&std::path::Path> =
                            paths.iter().map(|p| p.as_path()).collect();
                        match crate::collage::render_collage(
                            layout,
                            &path_refs,
                            &output_path,
                            &size,
                        ) {
                            Ok(()) => {
                                app.collage_preview_path = Some(output_path.clone());
                                // Add the rendered collage as a new photo
                                app.rescan();
                                // Find the newly added photo and select it
                                if let Some(idx) =
                                    app.photos.iter().position(|p| p.path == output_path)
                                {
                                    app.selected_photo = idx;
                                    app.current_edit = crate::app::PhotoEdit::default();
                                    app.screen = AppScreen::Preview;
                                    app.show_toast("Kollage skapat".to_string());
                                }
                            }
                            Err(e) => {
                                app.show_toast_long(format!("Kollagefel: {}", e));
                            }
                        }
                    }
                });
            }
        });
}

// =============================================================================
// SETTINGS AUTH SCREEN — PIN entry to access admin
// =============================================================================
fn draw_settings_auth(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    let s = touch_scale(ctx);
    let bg_main = Color32::from_rgb(10, 15, 26);
    let blue_accent = Color32::from_rgb(0, 229, 255);
    let text_dark = Color32::from_rgb(224, 247, 255);

    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(24.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

            ui.vertical_centered(|ui| {
                ui.add_space(80.0 * s);
                ui.label(
                    egui::RichText::new("⚙  Inställningar")
                        .size(32.0 * s)
                        .strong()
                        .color(text_dark),
                );
                ui.add_space(20.0 * s);
                ui.label(
                    egui::RichText::new("Ange PIN-kod")
                        .size(18.0 * s)
                        .color(Color32::from_rgb(100, 160, 220)),
                );
                ui.add_space(24.0 * s);

                // PIN display
                let mask = if app.settings_pin_input.is_empty() {
                    "____".to_string()
                } else {
                    app.settings_pin_input
                        .chars()
                        .map(|_| '●')
                        .collect::<String>()
                };
                ui.label(
                    egui::RichText::new(mask)
                        .size(40.0 * s)
                        .monospace()
                        .strong()
                        .color(blue_accent),
                );
                ui.add_space(16.0 * s);

                if app.settings_auth_failed {
                    ui.label(
                        egui::RichText::new("Fel PIN-kod")
                            .size(14.0 * s)
                            .color(Color32::from_rgb(255, 51, 102)),
                    );
                    ui.add_space(8.0 * s);
                }

                // Numpad
                let btn_size = Vec2::new(72.0 * s, 60.0 * s);
                for row in &[
                    ['1', '2', '3'],
                    ['4', '5', '6'],
                    ['7', '8', '9'],
                    ['C', '0', '←'],
                ] {
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - (btn_size.x * 3.0 + 20.0 * s)) / 2.0);
                        for &ch in row {
                            let label = ch.to_string();
                            let btn = egui::Button::new(
                                egui::RichText::new(&label)
                                    .size(22.0 * s)
                                    .strong()
                                    .color(text_dark),
                            )
                            .fill(Color32::from_rgb(13, 19, 33))
                            .stroke(Stroke::new(1.5 * s, Color32::from_rgb(0, 153, 170)))
                            .rounding(Rounding::same(10.0 * s))
                            .min_size(btn_size);
                            if ui.add(btn).clicked() {
                                match ch {
                                    'C' => app.settings_pin_input.clear(),
                                    '←' => {
                                        app.settings_pin_input.pop();
                                    }
                                    _ => {
                                        if app.settings_pin_input.len() < 6 {
                                            app.settings_pin_input.push(ch);
                                        }
                                    }
                                }
                            }
                            ui.add_space(10.0 * s);
                        }
                    });
                    ui.add_space(10.0 * s);
                }

                ui.add_space(20.0 * s);

                let btn_width = ui.available_width().min(240.0 * s);
                if ui
                    .add_sized(
                        Vec2::new(btn_width, 52.0 * s),
                        egui::Button::new(
                            egui::RichText::new("OK")
                                .size(18.0 * s)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(0, 229, 255))
                        .rounding(Rounding::same(10.0 * s)),
                    )
                    .clicked()
                {
                    if app.settings_pin_input == "1234" {
                        app.screen = AppScreen::Settings;
                        app.settings_auth_failed = false;
                        app.settings_pin_input.clear();
                        // Initialize edit state from current config
                        app.settings_price_edit.clear();
                        for size in &app.config.paper_sizes {
                            let price = app.config.price_for_size(size);
                            app.settings_price_edit
                                .insert(size.clone(), format!("{:.0}", price));
                        }
                    } else {
                        app.settings_auth_failed = true;
                    }
                }

                ui.add_space(16.0 * s);
                if nav_button(ui, pack.back, s).clicked() {
                    app.screen = AppScreen::ProductSelect;
                    app.settings_pin_input.clear();
                    app.settings_auth_failed = false;
                }
            });
        });
}

// =============================================================================
// SETTINGS SCREEN — Admin panel for prices, products, general config
// =============================================================================
fn draw_settings(ctx: &egui::Context, app: &mut ZalStudio) {
    let s = touch_scale(ctx);
    let bg_main = Color32::from_rgb(10, 15, 26);
    let bg_panel = Color32::from_rgb(10, 15, 26);
    let blue_accent = Color32::from_rgb(0, 229, 255);
    let text_dark = Color32::from_rgb(224, 247, 255);
    let text_mid = Color32::from_rgb(142, 202, 230);

    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(16.0 * s))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg_main);

            // Top bar
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚙  Inställningar")
                        .size(22.0 * s)
                        .strong()
                        .color(blue_accent),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if nav_button(ui, "✕", s).clicked() {
                        app.screen = AppScreen::ProductSelect;
                    }
                });
            });
            ui.add_space(12.0 * s);

            // Tabs
            ui.horizontal(|ui| {
                let tabs = [
                    (crate::app::SettingsTab::Products, "Produkter"),
                    (crate::app::SettingsTab::Prices, "Priser"),
                    (crate::app::SettingsTab::General, "Allmänt"),
                    (crate::app::SettingsTab::Dispatcher, "Dispatcher"),
                    (crate::app::SettingsTab::Color, "Färg"),
                ];
                for (tab, label) in tabs {
                    let active = app.settings_tab == tab;
                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .size(14.0 * s)
                            .strong()
                            .color(if active { Color32::WHITE } else { text_mid }),
                    )
                    .fill(if active {
                        Color32::from_rgb(0, 153, 170)
                    } else {
                        bg_panel
                    })
                    .rounding(Rounding::same(8.0 * s))
                    .min_size(Vec2::new(120.0 * s, 56.0 * s));
                    if ui.add(btn).clicked() {
                        app.settings_tab = tab;
                    }
                    ui.add_space(8.0 * s);
                }
            });
            ui.add_space(16.0 * s);

            match app.settings_tab {
                crate::app::SettingsTab::Prices => {
                    ui.label(
                        egui::RichText::new("Redigera priser")
                            .size(18.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                    ui.add_space(12.0 * s);

                    for size in &app.config.paper_sizes.clone() {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Format {}:", size))
                                    .size(15.0 * s)
                                    .color(text_mid),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let edit = app
                                        .settings_price_edit
                                        .get(size)
                                        .cloned()
                                        .unwrap_or_else(|| "0".to_string());
                                    let mut buf = edit;
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut buf)
                                            .desired_width(80.0 * s)
                                            .font(egui::TextStyle::Body),
                                    );
                                    if resp.changed() {
                                        app.settings_price_edit.insert(size.clone(), buf);
                                    }
                                    ui.label(
                                        egui::RichText::new("kr").size(14.0 * s).color(text_mid),
                                    );
                                },
                            );
                        });
                        ui.add_space(8.0 * s);
                    }

                    ui.add_space(16.0 * s);
                    if ui
                        .add_sized(
                            Vec2::new(180.0 * s, 48.0 * s),
                            egui::Button::new(
                                egui::RichText::new("💾 Spara priser")
                                    .size(15.0 * s)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(0, 229, 255))
                            .rounding(Rounding::same(8.0 * s)),
                        )
                        .clicked()
                    {
                        for (size, val_str) in &app.settings_price_edit {
                            if let Ok(price) = val_str.parse::<f64>() {
                                app.config.price_per_format.insert(size.clone(), price);
                            }
                        }
                        if let Err(e) = app.config.save() {
                            app.show_toast_long(format!("Kunde inte spara: {}", e));
                        } else {
                            app.settings_save_confirm = 3.0;
                            app.show_toast("Priser sparade".to_string());
                        }
                    }
                    if app.settings_save_confirm > 0.0 {
                        ui.add_space(8.0 * s);
                        ui.label(
                            egui::RichText::new("✓ Sparat!")
                                .size(14.0 * s)
                                .color(Color32::from_rgb(0, 255, 170)),
                        );
                    }
                }
                crate::app::SettingsTab::Products => {
                    ui.label(
                        egui::RichText::new("Produkter")
                            .size(18.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                    ui.add_space(12.0 * s);
                    ui.label(
                        egui::RichText::new(
                            "Här kan du aktivera/avaktivera produkter (kommer snart)",
                        )
                        .size(13.0 * s)
                        .color(text_mid),
                    );
                }
                crate::app::SettingsTab::General => {
                    ui.label(
                        egui::RichText::new("Allmänna inställningar")
                            .size(18.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                    ui.add_space(12.0 * s);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Temp-katalog:")
                                .size(14.0 * s)
                                .color(text_mid),
                        );
                        ui.label(
                            egui::RichText::new(
                                app.config.temp_directory.to_string_lossy().to_string(),
                            )
                            .size(13.0 * s)
                            .color(text_dark),
                        );
                    });
                    ui.add_space(8.0 * s);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Foto-katalog:")
                                .size(14.0 * s)
                                .color(text_mid),
                        );
                        ui.label(
                            egui::RichText::new(
                                app.config.photo_directory.to_string_lossy().to_string(),
                            )
                            .size(13.0 * s)
                            .color(text_dark),
                        );
                    });
                    ui.add_space(8.0 * s);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Server-port:")
                                .size(14.0 * s)
                                .color(text_mid),
                        );
                        ui.label(
                            egui::RichText::new(app.config.server_port.to_string())
                                .size(13.0 * s)
                                .color(text_dark),
                        );
                    });
                    ui.add_space(8.0 * s);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Pappersformat:")
                                .size(14.0 * s)
                                .color(text_mid),
                        );
                        ui.label(
                            egui::RichText::new(app.config.paper_sizes.join(", "))
                                .size(13.0 * s)
                                .color(text_dark),
                        );
                    });
                }
                crate::app::SettingsTab::Dispatcher => {
                    ui.label(
                        egui::RichText::new("Dispatcher — Utskriftshistorik")
                            .size(18.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                    ui.add_space(12.0 * s);

                    let entries = crate::print_history::list_history(&app.config.temp_directory);
                    if entries.is_empty() {
                        ui.label(
                            egui::RichText::new("Inga utskrifter i historiken än.")
                                .size(14.0 * s)
                                .color(text_mid),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} utskrifter sparade (max 30)",
                                entries.len()
                            ))
                            .size(13.0 * s)
                            .color(text_mid),
                        );
                        ui.add_space(8.0 * s);

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for entry in entries {
                                let status_color = match entry.record.status.as_str() {
                                    "completed" | "done" => Color32::from_rgb(0, 255, 170),
                                    "failed" => Color32::from_rgb(255, 51, 102),
                                    _ => Color32::from_rgb(0, 229, 255),
                                };

                                let card = Frame::none()
                                    .fill(Color32::from_rgb(17, 24, 39))
                                    .rounding(Rounding::same(8.0 * s))
                                    .inner_margin(10.0 * s)
                                    .stroke(Stroke::new(1.0 * s, Color32::from_rgb(30, 40, 60)));

                                card.show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(&entry.record.photo_name)
                                                    .size(14.0 * s)
                                                    .strong()
                                                    .color(text_dark),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{}  |  {} kopior  |  {}  |  {}",
                                                    entry.record.paper_size,
                                                    entry.record.copies,
                                                    entry.record.printer,
                                                    entry.folder_name
                                                ))
                                                .size(12.0 * s)
                                                .color(text_mid),
                                            );
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new("Status:")
                                                        .size(12.0 * s)
                                                        .color(text_mid),
                                                );
                                                ui.label(
                                                    egui::RichText::new(&entry.record.status)
                                                        .size(12.0 * s)
                                                        .strong()
                                                        .color(status_color),
                                                );
                                                if let Some(ref err) = entry.record.error {
                                                    ui.label(
                                                        egui::RichText::new(format!("— {}", err))
                                                            .size(11.0 * s)
                                                            .color(Color32::from_rgb(255, 51, 102)),
                                                    );
                                                }
                                            });
                                        });

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .add_sized(
                                                        Vec2::new(100.0 * s, 40.0 * s),
                                                        egui::Button::new(
                                                            egui::RichText::new("🖨  Skriv ut igen")
                                                                .size(12.0 * s)
                                                                .strong()
                                                                .color(Color32::WHITE),
                                                        )
                                                        .fill(Color32::from_rgb(0, 153, 170))
                                                        .rounding(Rounding::same(8.0 * s)),
                                                    )
                                                    .clicked()
                                                {
                                                    if let Err(e) =
                                                        app.reprint_from_history(&entry.folder_path)
                                                    {
                                                        app.show_toast_long(format!(
                                                            "Utskrift misslyckades: {}",
                                                            e
                                                        ));
                                                    } else {
                                                        app.show_toast(format!(
                                                            "{} skickad till skrivaren igen",
                                                            entry.record.photo_name
                                                        ));
                                                    }
                                                }
                                            },
                                        );
                                    });
                                });
                                ui.add_space(8.0 * s);
                            }
                        });
                    }
                }
                crate::app::SettingsTab::Color => {
                    ui.label(
                        egui::RichText::new("Färgkalibrering – Live förhandsvisning")
                            .size(18.0 * s)
                            .strong()
                            .color(text_dark),
                    );
                    ui.add_space(4.0 * s);
                    ui.label(
                        egui::RichText::new("Ändra värden och se direkt hur utskriften kommer se ut. Standard = None.")
                            .size(12.0 * s)
                            .color(text_mid),
                    );
                    ui.add_space(8.0 * s);

                    let printers: Vec<String> = app.config.all_printers().iter().map(|&s| s.to_string()).collect();
                    if printers.is_empty() {
                        ui.label(egui::RichText::new("Inga skrivare konfigurerade.").size(14.0 * s).color(text_mid));
                    } else {
                        // Printer selector
                        let mut selected = app.color_preview_printer.clone();
                        if selected.is_empty() {
                            selected = printers[0].clone();
                            app.color_preview_printer = selected.clone();
                        }
                        egui::ComboBox::from_label("")
                            .selected_text(egui::RichText::new(format!("🖨  {}", selected)).size(13.0 * s).color(text_dark))
                            .show_ui(ui, |ui| {
                                for p in &printers {
                                    if ui.selectable_label(selected == *p, p).clicked() {
                                        selected = p.clone();
                                    }
                                }
                            });
                        if selected != app.color_preview_printer {
                            app.color_preview_printer = selected.clone();
                            app.color_preview_dirty = true;
                        }
                        ui.add_space(8.0 * s);

                        let preview_w = (ui.available_width() * 0.55).max(300.0 * s);
                        let preview_h = (preview_w * 0.75).min(ui.available_height() - 60.0 * s);

                        ui.horizontal(|ui| {
                            // ── Left: live preview ──
                            ui.vertical(|ui| {
                                ui.set_width(preview_w);
                                if let Some(ref tex) = app.color_test_texture {
                                    let size = tex.size_vec2();
                                    let aspect = size.x / size.y.max(1.0);
                                    let h = preview_w / aspect;
                                    let display_h = h.min(preview_h);
                                    ui.image((tex.id(), egui::Vec2::new(preview_w, display_h)));
                                } else {
                                    ui.add_sized(
                                        egui::Vec2::new(preview_w, preview_h),
                                        egui::Spinner::new(),
                                    );
                                }
                                ui.add_space(4.0 * s);
                                ui.label(
                                    egui::RichText::new("Testbild: projectbilder/liminal.png")
                                        .size(10.0 * s)
                                        .color(text_mid),
                                );
                            });

                            ui.add_space(12.0 * s);

                            // ── Right: controls ──
                            ui.vertical(|ui| {
                                let mut opts = app.config.options_for_printer(&selected).clone();
                                let mut changed = false;

                                let defaults = crate::config::PrinterOptions::default();

                                let mut row = |ui: &mut egui::Ui, label: &str, val: &mut String, def: &str| -> bool {
                                    let mut changed = false;
                                    ui.horizontal(|ui| {
                                        ui.set_width(ui.available_width());
                                        ui.label(
                                            egui::RichText::new(label).size(11.0 * s).color(text_mid),
                                        );
                                        // Show empty + hint when value equals default
                                        let is_default = val == def;
                                        let mut buf = if is_default { String::new() } else { val.clone() };
                                        let resp = ui.add(
                                            egui::TextEdit::singleline(&mut buf)
                                                .desired_width(90.0 * s)
                                                .hint_text(def)
                                                .font(egui::TextStyle::Body),
                                        );
                                        if resp.changed() {
                                            if buf.trim().is_empty() {
                                                *val = def.to_string();
                                            } else {
                                                *val = buf;
                                            }
                                            changed = true;
                                        }
                                    });
                                    ui.add_space(2.0 * s);
                                    changed
                                };

                                changed |= row(ui, "ColorCorrection", &mut opts.color_correction, &defaults.color_correction);
                                changed |= row(ui, "Brightness", &mut opts.brightness, &defaults.brightness);
                                changed |= row(ui, "Contrast", &mut opts.contrast, &defaults.contrast);
                                changed |= row(ui, "Saturation", &mut opts.saturation, &defaults.saturation);
                                changed |= row(ui, "Gamma", &mut opts.gamma, &defaults.gamma);
                                ui.add_space(4.0 * s);
                                ui.label(
                                    egui::RichText::new("Per-kanal justering (motverkar gulhet)")
                                        .size(10.0 * s)
                                        .color(Color32::from_rgb(100, 160, 220)),
                                );
                                changed |= row(ui, "Cyan Gamma", &mut opts.cyan_gamma, &defaults.cyan_gamma);
                                changed |= row(ui, "Magenta Gamma", &mut opts.magenta_gamma, &defaults.magenta_gamma);
                                changed |= row(ui, "Yellow Gamma", &mut opts.yellow_gamma, &defaults.yellow_gamma);
                                changed |= row(ui, "Cyan Balance", &mut opts.cyan_balance, &defaults.cyan_balance);
                                changed |= row(ui, "Magenta Balance", &mut opts.magenta_balance, &defaults.magenta_balance);
                                changed |= row(ui, "Yellow Balance", &mut opts.yellow_balance, &defaults.yellow_balance);

                                if changed {
                                    app.config.printer_options.insert(selected.clone(), opts);
                                    app.color_preview_dirty = true;
                                }

                                ui.add_space(8.0 * s);
                                if ui
                                    .add_sized(
                                        Vec2::new(140.0 * s, 36.0 * s),
                                        egui::Button::new(
                                            egui::RichText::new("↺ Återställ default")
                                                .size(11.0 * s)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(120, 120, 120))
                                        .rounding(Rounding::same(6.0 * s)),
                                    )
                                    .clicked()
                                {
                                    let defaults = crate::config::PrinterOptions::default();
                                    app.config.printer_options.insert(selected.clone(), defaults);
                                    app.color_preview_dirty = true;
                                }

                                ui.add_space(8.0 * s);
                                if ui
                                    .add_sized(
                                        Vec2::new(140.0 * s, 40.0 * s),
                                        egui::Button::new(
                                            egui::RichText::new("💾 Spara")
                                                .size(13.0 * s)
                                                .strong()
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(0, 229, 255))
                                        .rounding(Rounding::same(6.0 * s)),
                                    )
                                    .clicked()
                                {
                                    if let Err(e) = app.config.save() {
                                        app.show_toast_long(format!("Kunde inte spara: {}", e));
                                    } else {
                                        app.settings_save_confirm = 3.0;
                                        app.show_toast("Färgprofiler sparade".to_string());
                                    }
                                }
                                if app.settings_save_confirm > 0.0 {
                                    ui.add_space(4.0 * s);
                                    ui.label(
                                        egui::RichText::new("✓ Sparat!")
                                            .size(12.0 * s)
                                            .color(Color32::from_rgb(0, 255, 170)),
                                    );
                                }
                            });
                        });
                    }
                }
            }
        });
}

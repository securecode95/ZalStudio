use egui::{Color32, Frame, Rounding, Stroke, Vec2};

use crate::app::{AppScreen, ZalStudio};
use crate::gallery::{format_dimensions, format_file_size};
use crate::lang::l;

pub fn apply_style(ctx: &egui::Context) {
    let mut style = egui::Style::default();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = Color32::from_rgb(14, 14, 22);
    style.visuals.window_fill = Color32::from_rgb(20, 20, 30);
    style.visuals.extreme_bg_color = Color32::from_rgb(10, 10, 16);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 50);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 65);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(65, 65, 95);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(85, 85, 120);
    style.visuals.selection.bg_fill = Color32::from_rgb(0, 150, 200);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(25, 25, 38);
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 12.0);
    ctx.set_style(style);
}

pub fn draw(ctx: &egui::Context, app: &mut ZalStudio) {
    apply_style(ctx);

    match app.screen {
        AppScreen::SourceSelect => draw_welcome(ctx, app),
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
    }

    draw_toast(ctx, app);
}

// =============================================================================
// WELCOME SCREEN
// =============================================================================
fn draw_welcome(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(16.0))
        .show(ctx, |ui| {
            // Language toggle top-right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let lang_btn = egui::Button::new(
                    egui::RichText::new(app.lang.name())
                        .size(13.0)
                        .color(Color32::from_rgb(0, 200, 255)),
                )
                .fill(Color32::from_rgb(30, 30, 45))
                .rounding(Rounding::same(6.0));
                if ui.add(lang_btn).clicked() {
                    app.lang = app.lang.toggle();
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("◆")
                        .size(40.0)
                        .color(Color32::from_rgb(0, 150, 220)),
                );
                ui.label(
                    egui::RichText::new("Zalstudio Kiosk")
                        .size(28.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    egui::RichText::new(pack.source_select_subtitle)
                        .size(14.0)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(16.0);

                let btn_width = ui.available_width().min(400.0);

                big_button(ui, "💾  USB / KAMERA MINNE", Color32::from_rgb(0, 120, 180), btn_width, || {
                    app.import_usb();
                });
                big_button(ui, "📱  MOBIL", Color32::from_rgb(180, 60, 160), btn_width, || {
                    app.open_mobile_upload();
                });
                // big_button(ui, "📸  GOOGLE FOTO", Color32::from_rgb(234, 67, 53), btn_width, || {
                //     app.start_google_photos_auth();
                // });
                // big_button(ui, "🔵  BLUETOOTH", Color32::from_rgb(80, 80, 120), btn_width, || {
                //     app.show_toast(pack.source_bluetooth_soon.to_string());
                // });
            });
        });
}

fn big_button(ui: &mut egui::Ui, label: &str, color: Color32, width: f32, mut on_click: impl FnMut()) {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .size(16.0)
            .strong()
            .color(Color32::WHITE),
    )
    .fill(color)
    .rounding(Rounding::same(10.0))
    .min_size(Vec2::new(width, 52.0));
    if ui.add(btn).clicked() {
        on_click();
    }
    ui.add_space(8.0);
}

// =============================================================================
// GALLERY SCREEN
// =============================================================================
fn draw_gallery(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(12.0))
        .show(ctx, |ui| {
            // Top bar
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.gallery)
                            .size(18.0)
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
                    if nav_button(ui, &cart_label).clicked() {
                        app.screen = AppScreen::Queue;
                    }
                    ui.add_space(4.0);
                    if nav_button(ui, "🔄").clicked() {
                        app.rescan();
                    }
                });
            });

            ui.add_space(8.0);

            if app.photos.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(pack.no_photos)
                            .size(14.0)
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
                            .rounding(Rounding::same(8.0))
                            .inner_margin(10.0)
                            .stroke(Stroke::new(
                                1.5,
                                if is_selected {
                                    Color32::from_rgb(0, 200, 255)
                                } else {
                                    Color32::TRANSPARENT
                                },
                            ));

                        let response = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Thumbnail
                                let thumb_size = 48.0;
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
                                            .rounding(Rounding::same(4.0)),
                                    );
                                } else {
                                    ui.add_space(thumb_size);
                                }
                                ui.add_space(8.0);

                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&photo.file_name)
                                            .size(14.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} · {}",
                                            format_dimensions(photo.dimensions),
                                            format_file_size(photo.file_size)
                                        ))
                                        .size(11.0)
                                        .color(Color32::from_gray(150)),
                                    );
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if in_queue {
                                            ui.label(
                                                egui::RichText::new("✓")
                                                    .size(18.0)
                                                    .color(Color32::from_rgb(0, 255, 150)),
                                            );
                                        }
                                    },
                                );
                            });
                        }).response;

                        if response.clicked() {
                            app.selected_photo = i;
                            app.screen = AppScreen::Preview;
                        }
                    }
                });
        });
}

fn nav_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).size(16.0).color(Color32::WHITE))
            .fill(Color32::from_rgb(40, 40, 58))
            .rounding(Rounding::same(8.0))
            .min_size(Vec2::new(44.0, 44.0)),
    )
}

// =============================================================================
// PREVIEW SCREEN
// =============================================================================
fn draw_preview(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(12.0))
        .show(ctx, |ui| {
            // Top bar
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::Gallery;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.preview)
                            .size(18.0)
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
                    if nav_button(ui, &cart_label).clicked() {
                        app.screen = AppScreen::Queue;
                    }
                });
            });

            ui.add_space(8.0);

            // Image preview (centered, takes available space)
            if let Some(texture) = app.selected_texture(ctx) {
                let available = ui.available_size();
                let tex_size = texture.size_vec2();
                // Reserve ~38 % of height for controls below the image
                let scale = (available.x / tex_size.x)
                    .min((available.y * 0.62) / tex_size.y)
                    .min(1.0);
                let display_size = tex_size * scale;

                ui.vertical_centered(|ui| {
                    let frame = Frame::none()
                        .fill(Color32::from_rgb(8, 8, 14))
                        .rounding(Rounding::same(10.0))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(50, 50, 70)))
                        .inner_margin(6.0);

                    frame.show(ui, |ui| {
                        ui.image((texture.id(), display_size));
                    });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("—")
                            .size(20.0)
                            .color(Color32::from_gray(100)),
                    );
                });
            }

            ui.add_space(10.0);

            // Controls - centered
            ui.vertical_centered(|ui| {
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
                    .size(12.0)
                    .color(Color32::from_gray(160)),
                );

                ui.add_space(6.0);

                // Paper size buttons centered
                ui.horizontal(|ui| {
                    for (i, size) in app.config.paper_sizes.iter().enumerate() {
                        let is_selected = i == app.paper_size_idx;
                        let btn = egui::Button::new(
                            egui::RichText::new(size)
                                .size(14.0)
                                .strong()
                                .color(if is_selected {
                                    Color32::WHITE
                                } else {
                                    Color32::from_gray(200)
                                }),
                        )
                        .fill(if is_selected {
                            Color32::from_rgb(0, 130, 190)
                        } else {
                            Color32::from_rgb(40, 40, 55)
                        })
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(80.0, 40.0));
                        if ui.add(btn).clicked() {
                            app.paper_size_idx = i;
                        }
                        ui.add_space(6.0);
                    }
                });

                ui.add_space(8.0);

                // Copies centered
                ui.horizontal(|ui| {
                    let minus = egui::Button::new(
                        egui::RichText::new("−").size(18.0).strong(),
                    )
                    .fill(Color32::from_rgb(45, 45, 65))
                    .rounding(Rounding::same(8.0))
                    .min_size(Vec2::new(44.0, 40.0));
                    if ui.add(minus).clicked() && app.copies > 1 {
                        app.copies -= 1;
                    }

                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(format!("{} {}", app.copies, pack.copies))
                            .size(16.0)
                            .strong()
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    ui.add_space(12.0);

                    let plus = egui::Button::new(
                        egui::RichText::new("+").size(18.0).strong(),
                    )
                    .fill(Color32::from_rgb(45, 45, 65))
                    .rounding(Rounding::same(8.0))
                    .min_size(Vec2::new(44.0, 40.0));
                    if ui.add(plus).clicked() && app.copies < 99 {
                        app.copies += 1;
                    }
                });

                ui.add_space(10.0);

                let btn_width = ui.available_width().min(320.0);

                // Add to queue
                let add_btn = egui::Button::new(
                    egui::RichText::new(format!("➕ {}", pack.add_to_queue))
                        .size(15.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 140, 100))
                .rounding(Rounding::same(10.0))
                .min_size(Vec2::new(btn_width, 48.0));
                if ui.add(add_btn).clicked() {
                    app.add_to_queue();
                }

                ui.add_space(6.0);

                // Go to cart
                let queue_count = app.queue.len();
                let cart_btn = egui::Button::new(
                    egui::RichText::new(format!("🛒 {} ({})", pack.print_queue_bottom, queue_count))
                        .size(15.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 120, 180))
                .rounding(Rounding::same(10.0))
                .min_size(Vec2::new(btn_width, 48.0));
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

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(12.0))
        .show(ctx, |ui| {
            // Top bar
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::Gallery;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(pack.print_queue_bottom)
                            .size(18.0)
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
                    if nav_button(ui, &label).clicked() {
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

            ui.add_space(8.0);

            if app.queue.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(pack.queue_empty)
                            .size(14.0)
                            .color(Color32::from_gray(120)),
                    );
                });
            } else {
                let queue_items: Vec<(usize, crate::app::QueueItem)> = app.queue.iter().cloned().enumerate().collect();
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for (i, item) in queue_items.iter().cloned() {
                            let photo_name = app.photos.get(item.photo_idx)
                                .map(|p| p.file_name.clone())
                                .unwrap_or_default();
                            let copy_word = crate::lang::copies_word(app.lang, item.copies);

                            let frame = Frame::none()
                                .fill(Color32::from_rgb(30, 30, 45))
                                .rounding(Rounding::same(8.0))
                                .inner_margin(10.0);

                            let response = frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Thumbnail
                                    let thumb_size = 40.0;
                                    if let Some(path) = app.photos.get(item.photo_idx).map(|p| p.path.clone()) {
                                        if let Some(texture) = app.texture_for(ctx, &path) {
                                            let tex_size = texture.size_vec2();
                                            let aspect = tex_size.x / tex_size.y;
                                            let (tw, th) = if aspect > 1.0 {
                                                (thumb_size, thumb_size / aspect)
                                            } else {
                                                (thumb_size * aspect, thumb_size)
                                            };
                                            ui.add(
                                                egui::Image::new((texture.id(), Vec2::new(tw, th)))
                                                    .rounding(Rounding::same(4.0)),
                                            );
                                        } else {
                                            ui.add_space(thumb_size);
                                        }
                                    }
                                    ui.add_space(6.0);

                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(&photo_name)
                                                .size(13.0)
                                                .strong()
                                                .color(Color32::WHITE),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} · {} {}",
                                                item.paper_size, item.copies, copy_word
                                            ))
                                            .size(11.0)
                                            .color(Color32::from_gray(150)),
                                        );
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let del_btn = ui.add(
                                                egui::Button::new(
                                                    egui::RichText::new("×")
                                                        .size(18.0)
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
                            }).response;

                            if response.clicked() {
                                app.queue_selected = i;
                            }
                        }
                    });

                ui.add_space(10.0);

                // Print button
                let count = app.queue.len();
                let btn_width = ui.available_width().min(320.0);
                ui.vertical_centered(|ui| {
                    let print_btn = egui::Button::new(
                        egui::RichText::new(format!("🖨 {} ({})", pack.print_queue, count))
                            .size(16.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 120, 180))
                    .rounding(Rounding::same(10.0))
                    .min_size(Vec2::new(btn_width, 52.0));
                    if ui.add(print_btn).clicked() {
                        app.print_queue();
                        app.screen = AppScreen::Gallery;
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
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(20.0))
        .show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    let time = ui.ctx().input(|i| i.time);
                    let spinner = ["◆", "▲", "◆", "▼"][(time * 3.0) as usize % 4];
                    ui.label(
                        egui::RichText::new(spinner)
                            .size(56.0)
                            .color(Color32::from_rgb(0, 150, 220)),
                    );
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new(pack.importing)
                            .size(24.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(&app.import_status)
                            .size(14.0)
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
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("🔌")
                        .size(40.0)
                        .color(Color32::from_rgb(0, 200, 255)),
                );
                ui.label(
                    egui::RichText::new("Anslut med kabel")
                        .size(20.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    egui::RichText::new("Om WiFi inte fungerar, koppla in telefonen med USB")
                        .size(12.0)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(24.0);

                let btn_width = ui.available_width().min(320.0);

                big_button(ui, pack.mobile_android, Color32::from_rgb(50, 180, 80), btn_width, || {
                    app.start_phone_flow(crate::app::PhoneType::Android);
                });
                big_button(ui, pack.mobile_iphone, Color32::from_rgb(200, 60, 60), btn_width, || {
                    app.start_phone_flow(crate::app::PhoneType::IPhone);
                });

                ui.add_space(24.0);

                // Back to QR/WiFi
                let back_btn = egui::Button::new(
                    egui::RichText::new(pack.wifi_qr_upload)
                        .size(13.0)
                        .color(Color32::from_gray(200)),
                )
                .fill(Color32::from_rgb(35, 35, 50))
                .rounding(Rounding::same(10.0))
                .min_size(Vec2::new(btn_width, 44.0));
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
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::MobileMenu;
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("📱")
                        .size(48.0)
                        .color(Color32::from_rgb(0, 200, 255)),
                );
                ui.label(
                    egui::RichText::new(pack.connect_phone_title)
                        .size(22.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(16.0);

                let hint = match app.phone_type {
                    crate::app::PhoneType::Android => pack.connect_phone_android,
                    crate::app::PhoneType::IPhone => pack.connect_phone_iphone,
                };
                ui.label(
                    egui::RichText::new(hint)
                        .size(14.0)
                        .color(Color32::from_gray(180)),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(pack.connect_phone_hint)
                        .size(12.0)
                        .color(Color32::from_gray(140)),
                );

                ui.add_space(24.0);

                // Animated spinner
                let time = ui.ctx().input(|i| i.time);
                let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                ui.label(
                    egui::RichText::new(spinner)
                        .size(32.0)
                        .color(Color32::from_rgb(0, 200, 255)),
                );
                ui.add_space(8.0);

                let elapsed = app.phone_connect_start.map(|t| t.elapsed().as_secs()).unwrap_or(0);

                // Show searching status with elapsed time — we switch screens immediately
                // when results arrive, so this is the only message the user sees while waiting.
                if elapsed > 10 {
                    ui.label(
                        egui::RichText::new(format!("{} ({}s)", pack.connect_phone_scanning, elapsed))
                            .size(14.0)
                            .strong()
                            .color(Color32::from_rgb(255, 200, 0)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Detta kan ta upp till en minut...")
                            .size(11.0)
                            .color(Color32::from_gray(140)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!("{} ({}s)", pack.connect_phone_searching, elapsed))
                            .size(14.0)
                            .strong()
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                }

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Anslut telefonen och tryck Tillåt på skärmen. Se till att telefonen är i filöverföringsläge.")
                        .size(11.0)
                        .color(Color32::from_gray(120)),
                );

                ui.add_space(16.0);

                let btn_width = ui.available_width().min(280.0);
                let search_btn = egui::Button::new(
                    egui::RichText::new(pack.connect_phone_search)
                        .size(14.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 130, 190))
                .rounding(Rounding::same(10.0))
                .min_size(Vec2::new(btn_width, 44.0));
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
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if nav_button(ui, app.lang.name()).clicked() {
                        app.lang = app.lang.toggle();
                    }
                });
            });

            ui.vertical_centered(|ui| {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(pack.mobile_upload_title)
                        .size(22.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    egui::RichText::new(pack.mobile_upload_hint)
                        .size(12.0)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(8.0);

                // ── WiFi info box ─────────────────────────────────────────────
                egui::Frame::none()
                    .fill(Color32::from_rgb(25, 25, 40))
                    .rounding(Rounding::same(12.0))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_max_width(340.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!("📶  {}  ·  🔑  {}",
                                    app.config.wifi_ssid,
                                    app.config.wifi_password
                                ))
                                .size(14.0)
                                .strong()
                                .color(Color32::from_rgb(255, 220, 100)),
                            );
                        });
                    });

                ui.add_space(6.0);

                // ── Helper text for kiosk operators ───────────────────────────
                ui.label(
                    egui::RichText::new(if app.lang == crate::lang::Language::Swedish {
                        "💡  Om WiFi inte syns: Starta Mobile Hotspot i Windows-inställningarna"
                    } else {
                        "💡  If WiFi is not visible: Turn on Mobile Hotspot in Windows Settings"
                    })
                    .size(11.0)
                    .color(Color32::from_gray(140)),
                );

                ui.add_space(10.0);

                // ── Two QR codes side by side (centered) ──────────────────────
                let qr_size = 140.0_f32;
                let gap = 32.0_f32;
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
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::from_rgb(0, 200, 255)),
                            );
                            ui.add_space(4.0);
                            if let Some(texture) = app.wifi_qr_texture(ctx) {
                                ui.image((texture.id(), Vec2::new(qr_size, qr_size)));
                            } else {
                                ui.label(
                                    egui::RichText::new("…")
                                        .size(12.0)
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
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::from_rgb(0, 200, 255)),
                            );
                            ui.add_space(4.0);
                            if let Some(texture) = app.qr_texture(ctx) {
                                ui.image((texture.id(), Vec2::new(qr_size, qr_size)));
                            } else {
                                ui.label(
                                    egui::RichText::new("…")
                                        .size(12.0)
                                        .color(Color32::from_gray(120)),
                                );
                            }
                        });
                    });
                });

                ui.add_space(4.0);
                if let Some(url) = &app.server_url {
                    ui.label(
                        egui::RichText::new(format!("{}: {}", pack.mobile_upload_url, url))
                            .size(10.0)
                            .color(Color32::from_gray(140)),
                    );
                }

                ui.add_space(8.0);

                let btn_width = ui.available_width().min(280.0);
                let refresh_btn = egui::Button::new(
                    egui::RichText::new(pack.mobile_upload_refresh)
                        .size(14.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 130, 190))
                .rounding(Rounding::same(10.0))
                .min_size(Vec2::new(btn_width, 44.0));
                if ui.add(refresh_btn).clicked() {
                    app.rescan();
                }

                ui.add_space(6.0);

                let done_btn = egui::Button::new(
                    egui::RichText::new(pack.mobile_upload_done)
                        .size(14.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 140, 100))
                .rounding(Rounding::same(10.0))
                .min_size(Vec2::new(btn_width, 44.0));
                if ui.add(done_btn).clicked() {
                    app.screen = AppScreen::Gallery;
                }

                ui.add_space(6.0);

                // Small secondary link for cable connection
                let cable_btn = egui::Button::new(
                    egui::RichText::new(pack.mobile_cable_connect)
                        .size(11.0)
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
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::MobileMenu;
                    #[cfg(windows)] {
                        app.wpd_cmd_tx = None;
                        app.wpd_res_rx = None;
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(pack.phone_folders_title)
                    .size(20.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(pack.phone_folders_hint)
                    .size(12.0)
                    .color(Color32::from_gray(140)),
            );
            ui.add_space(12.0);

            let non_empty: Vec<_> = app.wpd_folders.iter().filter(|f| f.item_count > 0).cloned().collect();
            if non_empty.is_empty() {
                ui.label(
                    egui::RichText::new("Inga bildmappar hittades i DCIM")
                        .size(13.0)
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
                                .size(14.0)
                                .strong()
                                .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(35, 35, 55))
                            .rounding(Rounding::same(10.0))
                            .min_size(Vec2::new(ui.available_width(), 52.0));

                            if ui.add(btn).clicked() {
                                let path = folder.full_path.clone();
                                app.open_phone_folder(path);
                            }
                            ui.add_space(6.0);
                        }
                    });
            }
        });
}

#[cfg(not(windows))]
fn draw_phone_folder_select(_ctx: &egui::Context, _app: &mut ZalStudio) {}

// =============================================================================
// WIRED PHONE PICKER SCREEN
// =============================================================================
fn draw_wired_phone_picker(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    // On Linux: Android uses direct MTP (mtp_photos)
    // On Windows: Android uses PowerShell + Shell.Application (wpd_photos)
    // On fallback: uses GVFS filesystem (phone_photos)
    let is_linux_mtp = app.phone_type == crate::app::PhoneType::Android
        && cfg!(target_os = "linux");
    let is_windows_wpd = app.phone_type == crate::app::PhoneType::Android
        && cfg!(windows);
    let is_android_direct = is_linux_mtp || is_windows_wpd;

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(12.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::MobileMenu;
                }
            });

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(pack.wired_select_photos)
                    .size(18.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(8.0);

            let (photo_count, selected_count, all_selected) = if is_android_direct {
                let count = app.mtp_photos.len();
                let sel = app.mtp_selected.iter().filter(|&&s| s).count();
                (count, sel, sel == count)
            } else {
                let count = app.phone_photos.len();
                let sel = app.phone_selected.iter().filter(|&&s| s).count();
                (count, sel, sel == count)
            };

            if photo_count == 0 {
                ui.label(
                    egui::RichText::new("Inga bilder hittades")
                        .size(13.0)
                        .color(Color32::from_gray(140)),
                );
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if is_linux_mtp {
                            for (i, photo) in app.mtp_photos.iter().enumerate() {
                                let mut is_selected =
                                    app.mtp_selected.get(i).copied().unwrap_or(false);

                                let bg = if is_selected {
                                    Color32::from_rgb(0, 110, 160)
                                } else {
                                    Color32::from_rgb(30, 30, 45)
                                };

                                let frame = Frame::none()
                                    .fill(bg)
                                    .rounding(Rounding::same(8.0))
                                    .inner_margin(10.0);

                                let response = frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let checkbox = ui.add(egui::Checkbox::new(
                                            &mut is_selected,
                                            egui::RichText::new(&photo.file_name)
                                                .size(13.0)
                                                .strong()
                                                .color(Color32::WHITE),
                                        ));
                                        if checkbox.clicked() {
                                            if let Some(sel) = app.mtp_selected.get_mut(i) {
                                                *sel = !*sel;
                                            }
                                        }
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(
                                                        format_file_size(photo.file_size),
                                                    )
                                                    .size(11.0)
                                                    .color(Color32::from_gray(150)),
                                                );
                                            },
                                        );
                                    });
                                }).response;

                                if response.clicked() {
                                    if let Some(sel) = app.mtp_selected.get_mut(i) {
                                        *sel = !*sel;
                                    }
                                }
                            }
                        } else if is_windows_wpd {
                            for (i, photo) in app.wpd_photos.iter().enumerate() {
                                let mut is_selected =
                                    app.wpd_selected.get(i).copied().unwrap_or(false);

                                let bg = if is_selected {
                                    Color32::from_rgb(0, 110, 160)
                                } else {
                                    Color32::from_rgb(30, 30, 45)
                                };

                                let frame = Frame::none()
                                    .fill(bg)
                                    .rounding(Rounding::same(8.0))
                                    .inner_margin(10.0);

                                let response = frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let checkbox = ui.add(egui::Checkbox::new(
                                            &mut is_selected,
                                            egui::RichText::new(&photo.name)
                                                .size(13.0)
                                                .strong()
                                                .color(Color32::WHITE),
                                        ));
                                        if checkbox.clicked() {
                                            if let Some(sel) = app.wpd_selected.get_mut(i) {
                                                *sel = !*sel;
                                            }
                                        }
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(
                                                        format_file_size(photo.size),
                                                    )
                                                    .size(11.0)
                                                    .color(Color32::from_gray(150)),
                                                );
                                            },
                                        );
                                    });
                                }).response;

                                if response.clicked() {
                                    if let Some(sel) = app.wpd_selected.get_mut(i) {
                                        *sel = !*sel;
                                    }
                                }
                            }
                        } else {
                            for (i, photo) in app.phone_photos.iter().enumerate() {
                                let mut is_selected =
                                    app.phone_selected.get(i).copied().unwrap_or(false);

                                let bg = if is_selected {
                                    Color32::from_rgb(0, 110, 160)
                                } else {
                                    Color32::from_rgb(30, 30, 45)
                                };

                                let frame = Frame::none()
                                    .fill(bg)
                                    .rounding(Rounding::same(8.0))
                                    .inner_margin(10.0);

                                let response = frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let checkbox = ui.add(egui::Checkbox::new(
                                            &mut is_selected,
                                            egui::RichText::new(&photo.file_name)
                                                .size(13.0)
                                                .strong()
                                                .color(Color32::WHITE),
                                        ));
                                        if checkbox.clicked() {
                                            if let Some(sel) = app.phone_selected.get_mut(i) {
                                                *sel = !*sel;
                                            }
                                        }
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(
                                                        format_file_size(photo.file_size),
                                                    )
                                                    .size(11.0)
                                                    .color(Color32::from_gray(150)),
                                                );
                                            },
                                        );
                                    });
                                }).response;

                                if response.clicked() {
                                    if let Some(sel) = app.phone_selected.get_mut(i) {
                                        *sel = !*sel;
                                    }
                                }
                            }
                        }
                    });

                ui.add_space(8.0);

                let btn_width = ui.available_width().min(320.0);
                ui.vertical_centered(|ui| {
                    let toggle_btn = egui::Button::new(
                        egui::RichText::new(if all_selected {
                            pack.deselect_all
                        } else {
                            pack.select_all
                        })
                        .size(13.0)
                        .color(Color32::from_gray(200)),
                    )
                    .fill(Color32::from_rgb(40, 40, 55))
                    .rounding(Rounding::same(8.0))
                    .min_size(Vec2::new(btn_width, 36.0));
                    if ui.add(toggle_btn).clicked() {
                        let new_val = !all_selected;
                        if is_linux_mtp {
                            for sel in &mut app.mtp_selected {
                                *sel = new_val;
                            }
                        } else if is_windows_wpd {
                            for sel in &mut app.wpd_selected {
                                *sel = new_val;
                            }
                        } else {
                            for sel in &mut app.phone_selected {
                                *sel = new_val;
                            }
                        }
                    }

                    ui.add_space(6.0);

                    let import_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} ({})",
                            pack.wired_import, selected_count
                        ))
                        .size(15.0)
                        .strong()
                        .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 140, 100))
                    .rounding(Rounding::same(10.0))
                    .min_size(Vec2::new(btn_width, 48.0));

                    if ui.add(import_btn).clicked() && selected_count > 0 {
                        app.import_selected_phone_photos();
                    }
                });
            }
        });
}

// =============================================================================
// GOOGLE PHOTOS AUTH SCREEN — QR code login
// =============================================================================
fn draw_google_drive_auth(ctx: &egui::Context, app: &mut ZalStudio) {
    let pack = l(app.lang);
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::SourceSelect;
                    app.pkce_state = None;
                    app.auth_qr_texture = None;
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("📸")
                        .size(36.0)
                        .color(Color32::from_rgb(234, 67, 53)),
                );
                ui.label(
                    egui::RichText::new(pack.google_drive_title)
                        .size(22.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(pack.google_drive_auth_hint)
                        .size(12.0)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(20.0);

                if app.pkce_state.is_some() {
                    ui.label(
                        egui::RichText::new("🌐 En webbläsare har öppnats")
                            .size(16.0)
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Logga in på Google i webbläsaren och godkänn åtkomst")
                            .size(14.0)
                            .color(Color32::from_gray(180)),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("(Stäng webbläsaren när du är klar)")
                            .size(12.0)
                            .color(Color32::from_gray(120)),
                    );
                    ui.add_space(16.0);

                    // Retry button in case browser didn't open
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new("Öppna webbläsaren igen")
                                .size(14.0)
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(40, 40, 60))
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(220.0, 44.0))
                    ).clicked() {
                        if let Some(pkce) = &app.pkce_state {
                            let auth_url = crate::google_drive::build_auth_url(
                                &app.config.google_client_id,
                                &pkce.redirect_uri,
                                &pkce.state,
                                &pkce.code_challenge,
                            );
                            #[cfg(windows)] {
                                let _ = std::process::Command::new("rundll32")
                                    .args(["url.dll,FileProtocolHandler", &auth_url])
                                    .spawn();
                            }
                            #[cfg(target_os = "linux")] {
                                let _ = std::process::Command::new("xdg-open")
                                    .arg(&auth_url)
                                    .spawn();
                            }
                        }
                    }

                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(&app.drive_status)
                            .size(12.0)
                            .color(Color32::from_rgb(255, 200, 0)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(&app.drive_status)
                            .size(13.0)
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
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(12.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
            });

            ui.add_space(4.0);
            ui.heading(
                egui::RichText::new(pack.tab_drive)
                    .size(18.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(8.0);

            if app.drive_files.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Inga bilder i Drive")
                            .size(13.0)
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
                                    let is_selected = app.drive_selected.get(idx).copied().unwrap_or(false);

                                    let bg = if is_selected {
                                        Color32::from_rgb(0, 150, 220)
                                    } else {
                                        Color32::from_rgb(30, 30, 45)
                                    };

                                    let frame = Frame::none()
                                        .fill(bg)
                                        .rounding(Rounding::same(8.0))
                                        .inner_margin(4.0);

                                    let response = frame.show(ui, |ui| {
                                        ui.set_max_width(thumb_size);
                                        ui.set_min_width(thumb_size);
                                        ui.vertical_centered(|ui| {
                                            // Thumbnail
                                            let img_size = thumb_size - 8.0;
                                            if let Some(tex) = app.drive_thumbnails.get(&file.id) {
                                                ui.image((tex.id(), Vec2::new(img_size, img_size)));
                                            } else {
                                                ui.add_sized(
                                                    Vec2::new(img_size, img_size),
                                                    egui::Label::new(egui::RichText::new("📷").size(thumb_size * 0.25)),
                                                );
                                            }

                                            // Filename (truncated)
                                            let max_chars = (thumb_size / 8.0) as usize;
                                            let name = if file.name.len() > max_chars {
                                                format!("{}...", &file.name[..max_chars.saturating_sub(3)])
                                            } else {
                                                file.name.clone()
                                            };
                                            ui.label(
                                                egui::RichText::new(name)
                                                    .size(10.0)
                                                    .color(Color32::from_gray(200)),
                                            );

                                            // Checkmark if selected
                                            if is_selected {
                                                ui.label(
                                                    egui::RichText::new("✅")
                                                        .size(14.0),
                                                );
                                            }
                                        });
                                    }).response;

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

                ui.add_space(12.0);

                let selected_count = app.drive_selected.iter().filter(|&&s| s).count();
                let btn_width = ui.available_width().min(320.0);
                ui.vertical_centered(|ui| {
                    let download_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} ({})",
                            pack.google_drive_download, selected_count
                        ))
                        .size(15.0)
                        .strong()
                        .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(66, 133, 244))
                    .rounding(Rounding::same(10.0))
                    .min_size(Vec2::new(btn_width, 48.0));

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
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(12.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if nav_button(ui, "←").clicked() {
                    app.screen = AppScreen::SourceSelect;
                }
            });

            ui.add_space(4.0);

            // Tabs
            ui.horizontal(|ui| {
                let drive_btn = egui::Button::new(
                    egui::RichText::new(pack.tab_drive)
                        .size(14.0)
                        .strong()
                        .color(Color32::from_gray(200)),
                )
                .fill(Color32::from_rgb(30, 30, 45))
                .rounding(Rounding::same(8.0))
                .min_size(Vec2::new(ui.available_width() * 0.45, 36.0));
                if ui.add(drive_btn).clicked() {
                    app.screen = AppScreen::GoogleDrivePicker;
                }

                ui.add_space(8.0);

                let photos_btn = egui::Button::new(
                    egui::RichText::new(pack.tab_photos)
                        .size(14.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(200, 60, 120))
                .rounding(Rounding::same(8.0))
                .min_size(Vec2::new(ui.available_width(), 36.0));
                ui.add(photos_btn);
            });

            ui.add_space(8.0);

            if app.photo_items.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Inga bilder i Google Foto")
                            .size(13.0)
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
                                .rounding(Rounding::same(8.0))
                                .inner_margin(10.0);

                            let response = frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let checkbox = ui.add(egui::Checkbox::new(
                                        &mut is_selected,
                                        egui::RichText::new(&photo.filename)
                                            .size(13.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    ));
                                    if checkbox.clicked() {
                                        if let Some(sel) = app.photo_selected.get_mut(i) {
                                            *sel = !*sel;
                                        }
                                    }
                                });
                            }).response;

                            if response.clicked() {
                                if let Some(sel) = app.photo_selected.get_mut(i) {
                                    *sel = !*sel;
                                }
                            }
                        }
                    });

                ui.add_space(8.0);

                let selected_count = app.photo_selected.iter().filter(|&&s| s).count();
                let btn_width = ui.available_width().min(320.0);
                ui.vertical_centered(|ui| {
                    let download_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} ({})",
                            pack.google_drive_download, selected_count
                        ))
                        .size(15.0)
                        .strong()
                        .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(200, 60, 120))
                    .rounding(Rounding::same(10.0))
                    .min_size(Vec2::new(btn_width, 48.0));

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
    if let Some((ref msg, t)) = app.toast {
        let alpha = (t * 255.0 / 0.5).min(255.0) as u8;
        let screen = ctx.screen_rect();
        let toast_width = 340.0;
        let toast_height = 50.0;
        let pos = egui::pos2(
            screen.center().x - toast_width / 2.0,
            screen.max.y - toast_height - 16.0,
        );

        egui::Area::new("toast".into())
            .fixed_pos(pos)
            .show(ctx, |ui| {
                let frame = Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(30, 30, 45, alpha))
                    .rounding(Rounding::same(10.0))
                    .stroke(Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(60, 60, 90, alpha),
                    ))
                    .inner_margin(14.0);

                frame.show(ui, |ui| {
                    ui.set_min_width(toast_width);
                    ui.set_min_height(toast_height);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(msg)
                                .size(14.0)
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
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::from_rgb(14, 14, 22)).inner_margin(20.0))
        .show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("🖨")
                            .size(64.0)
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new(pack.print_done_title)
                            .size(26.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(pack.print_done_subtitle)
                            .size(14.0)
                            .color(Color32::from_gray(160)),
                    );
                    ui.add_space(40.0);

                    let time = ui.ctx().input(|i| i.time);
                    let spinner = ["◐", "◑", "◒", "◓"][(time * 4.0) as usize % 4];
                    ui.label(
                        egui::RichText::new(spinner)
                            .size(32.0)
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("{:.0}s", app.print_done_timer))
                            .size(12.0)
                            .color(Color32::from_gray(120)),
                    );
                });
            });
        });
}

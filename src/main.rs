mod app;
mod collage;
mod config;
mod gallery;
mod google_drive;
mod import;
mod lang;
mod mtp_backend;
mod printer;
mod print_history;
mod server;
mod ui;
mod usb_detect;
mod wired_import;

use app::ZalStudio;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_resizable(false)
            .with_decorations(false)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Zalstudio Kiosk",
        options,
        Box::new(|cc| Ok(Box::new(ZalStudio::new(cc)))),
    )
}

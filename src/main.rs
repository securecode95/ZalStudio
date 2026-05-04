mod app;
mod config;
mod gallery;
mod google_drive;
mod import;
mod lang;
mod printer;
mod server;
mod ui;
mod mtp_backend;
#[cfg(windows)]
mod powershell_mtp_backend;
mod wired_import;

use app::ZalStudio;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_min_inner_size([640.0, 480.0])
            .with_resizable(true)
            .with_maximized(true)
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "Zalstudio Kiosk",
        options,
        Box::new(|cc| Ok(Box::new(ZalStudio::new(cc)))),
    )
}

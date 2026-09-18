#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod lan_receiver;
mod theme;
mod views;

use app::ProteusClientApp;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Proteus Client — Service BOS")
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([960.0, 620.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Proteus Client",
        native_options,
        Box::new(|cc| Ok(Box::new(ProteusClientApp::new(cc)))),
    )
}

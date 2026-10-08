#![cfg_attr(not(test), windows_subsystem = "windows")]

mod app;
mod lan_receiver;
mod replication_daemon;
mod theme;
mod views;

use app::ProteusClientApp;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    std::panic::set_hook(Box::new(|info| {
        use std::io::Write;
        let p = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("proteus_client_panic.txt")));
        if let Some(path) = p {
            if let Ok(mut f) = std::fs::File::create(path) {
                let _ = writeln!(f, "PANIC: {:#?}", info);
            }
        }
    }));

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Proteus Client — Service BOS")
            .with_icon(egui::IconData {
                rgba: include_bytes!("../../assets/proteus_emblem_128.rgba").to_vec(),
                width: 128,
                height: 128,
            })
            .with_maximized(true)
            .with_min_inner_size([960.0, 600.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Proteus Client",
        native_options,
        Box::new(|cc| Ok(Box::new(ProteusClientApp::new(cc)))),
    )
}

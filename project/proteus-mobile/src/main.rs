//! Standalone Mobile Simulator Runner for Proteus Mobile.
//! Launches a phone-viewport window (393 × 852) to test and interact with the mobile PDS client.

use eframe::egui;
use proteus_mobile::{render_mobile_view, MobileAppState};

#[derive(Default)]
struct MobileSimulatorApp {
    state: MobileAppState,
}

impl eframe::App for MobileSimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(proteus_mobile::BG_BASE))
            .show(ctx, |ui| {
                render_mobile_view(ui, &mut self.state);
            });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([393.0, 852.0]) // iPhone 16 Pro / Android flagship resolution
            .with_min_inner_size([320.0, 568.0])
            .with_title("Proteus Mobile — PDS Handheld Client"),
        ..Default::default()
    };

    eframe::run_native(
        "Proteus Mobile",
        native_options,
        Box::new(|_cc| Ok(Box::new(MobileSimulatorApp::default()))),
    )
}

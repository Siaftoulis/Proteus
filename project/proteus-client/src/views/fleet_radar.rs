//! Screen: Live Fleet & Telemetry Radar View for Proteus Client.
//! Offline-first real-time tracking for Maritime (AIS VHF) and Aviation (ADS-B 1090MHz).
//! Includes predictive dead-reckoning extrapolation for signal dead zones.

use proteus_core::telemetry::{
    AircraftCategory, AssetKind, AssetPosition, TelemetrySignalSource, TrackedAsset, VesselCategory,
};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleetFilter {
    All,
    Maritime,
    Aviation,
    OfflineDeadReckoning,
}

pub struct FleetRadarState {
    pub filter: FleetFilter,
    pub assets: Vec<TrackedAsset>,
    pub selected_asset_id: Option<String>,
    pub simulation_step_count: u32,
}

impl Default for FleetRadarState {
    fn default() -> Self {
        Self {
            filter: FleetFilter::All,
            assets: sample_fleet_assets(),
            selected_asset_id: None,
            simulation_step_count: 0,
        }
    }
}

impl FleetRadarState {
    pub fn simulate_dead_reckoning_step(&mut self) {
        self.simulation_step_count += 1;
        for asset in &mut self.assets {
            let next_pos = asset.position.extrapolate_dead_reckoning(60.0);
            asset.position = next_pos;
            asset.signal_source = TelemetrySignalSource::DeadReckoningPredictive;
            asset.last_seen_epoch += 60;
        }
    }

    pub fn reset_assets(&mut self) {
        self.assets = sample_fleet_assets();
        self.simulation_step_count = 0;
    }
}

fn sample_fleet_assets() -> Vec<TrackedAsset> {
    vec![
        TrackedAsset::new_vessel(
            "VSL-DELOS",
            "Blue Star Delos",
            "MMSI-239123400",
            VesselCategory::Ferry,
            AssetPosition::new(37.4415, 25.3284, 24.2, 118.0, None),
            TelemetrySignalSource::AisVhfClassA,
            1710000000,
        ),
        TrackedAsset::new_vessel(
            "VSL-CARGO",
            "Aegean Transporter",
            "IMO-9481203",
            VesselCategory::ContainerCargo,
            AssetPosition::new(36.1204, 23.8540, 18.5, 210.0, None),
            TelemetrySignalSource::AisVhfClassA,
            1710000000,
        ),
        TrackedAsset::new_vessel(
            "VSL-LNG",
            "Hellas Pioneer",
            "IMO-9763321",
            VesselCategory::Tanker,
            AssetPosition::new(37.9100, 23.4150, 11.2, 45.0, None),
            TelemetrySignalSource::AisVhfClassA,
            1710000000,
        ),
        TrackedAsset::new_vessel(
            "VSL-CAIQUE",
            "Άγιος Νικόλαος (Καΐκι)",
            "ΚΑΛΥΜΝΟΣ-412",
            VesselCategory::FishingCaique,
            AssetPosition::new(36.9820, 27.0540, 7.8, 310.0, None),
            TelemetrySignalSource::DeadReckoningPredictive,
            1710000000,
        ),
        TrackedAsset::new_aircraft(
            "AIR-A320",
            "Airbus A320neo (A3-314)",
            "SX-NEA",
            AircraftCategory::PassengerAirliner,
            AssetPosition::new(38.2510, 23.9100, 440.0, 195.0, Some(28000.0)),
            1710000000,
        ),
        TrackedAsset::new_aircraft(
            "AIR-ATR72",
            "ATR 72-600 (GQ-210)",
            "SX-SEV",
            AircraftCategory::CargoTransport,
            AssetPosition::new(37.1500, 25.1000, 235.0, 82.0, Some(14000.0)),
            1710000000,
        ),
    ]
}

pub fn draw_fleet_radar_view(ui: &mut Ui, state: &mut FleetRadarState) {
    ui.vertical(|ui| {
        // Section Header
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🛰 Live Fleet & Telemetry Radar").strong().size(20.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚡ Προσομοίωση 60s Dead-Reckoning").clicked() {
                    state.simulate_dead_reckoning_step();
                }
                if state.simulation_step_count > 0 && ui.button("↺ Επαναφορά").clicked() {
                    state.reset_assets();
                }
            });
        });

        ui.add_space(6.0);
        ui.label(
            RichText::new("Πολυμορφική παρακολούθηση σε πραγματικό χρόνο: Ναυτιλία (AIS VHF), Αεροπορία (ADS-B) και Offline Dead-Reckoning.")
                .size(12.0)
                .color(crate::theme::TEXT_MUTED),
        );

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(10.0);

        // Filter Pills
        ui.horizontal(|ui| {
            let filter_btn = |ui: &mut Ui, label: &str, active: bool| {
                let fill = if active { crate::theme::STEEL_SLATE } else { crate::theme::BG_CARD };
                let text_col = if active { Color32::WHITE } else { crate::theme::TEXT_MUTED };
                ui.add(egui::Button::new(RichText::new(label).color(text_col).strong()).fill(fill))
            };

            if filter_btn(ui, "Όλα τα Assets", state.filter == FleetFilter::All).clicked() {
                state.filter = FleetFilter::All;
            }
            if filter_btn(ui, "🚢 Ναυτιλία (Πλοία/Καΐκια)", state.filter == FleetFilter::Maritime).clicked() {
                state.filter = FleetFilter::Maritime;
            }
            if filter_btn(ui, "✈️ Αεροπορία (ADS-B)", state.filter == FleetFilter::Aviation).clicked() {
                state.filter = FleetFilter::Aviation;
            }
            if filter_btn(ui, "📡 Dead-Reckoning (Offline)", state.filter == FleetFilter::OfflineDeadReckoning).clicked() {
                state.filter = FleetFilter::OfflineDeadReckoning;
            }
        });

        ui.add_space(12.0);

        // Asset Cards Grid
        egui::ScrollArea::vertical().show(ui, |ui| {
            let filtered_assets: Vec<&TrackedAsset> = state
                .assets
                .iter()
                .filter(|a| match state.filter {
                    FleetFilter::All => true,
                    FleetFilter::Maritime => matches!(a.kind, AssetKind::Maritime(_)),
                    FleetFilter::Aviation => matches!(a.kind, AssetKind::Aviation(_)),
                    FleetFilter::OfflineDeadReckoning => a.signal_source == TelemetrySignalSource::DeadReckoningPredictive,
                })
                .collect();

            if filtered_assets.is_empty() {
                ui.label(RichText::new("Δεν βρέθηκαν assets για το επιλεγμένο φίλτρο.").color(crate::theme::TEXT_MUTED));
                return;
            }

            for asset in filtered_assets {
                let is_selected = state.selected_asset_id.as_deref() == Some(&asset.asset_id);
                let border_stroke = if is_selected {
                    Stroke::new(1.5, crate::theme::ACCENT_PRIMARY)
                } else {
                    Stroke::new(1.0, crate::theme::BORDER_SUBTLE)
                };

                let card_resp = Frame::new()
                    .fill(crate::theme::BG_CARD)
                    .stroke(border_stroke)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let icon = match &asset.kind {
                                AssetKind::Maritime(VesselCategory::Ferry) => "🚢 [FERRY]",
                                AssetKind::Maritime(VesselCategory::ContainerCargo) => "🚢 [CARGO]",
                                AssetKind::Maritime(VesselCategory::Tanker) => "🚢 [TANKER]",
                                AssetKind::Maritime(VesselCategory::FishingCaique) => "⛵ [CAIQUE]",
                                AssetKind::Aviation(AircraftCategory::PassengerAirliner) => "✈️ [AIRLINER]",
                                AssetKind::Aviation(AircraftCategory::CargoTransport) => "✈️ [CARGO-AIR]",
                                AssetKind::Aviation(AircraftCategory::LightAircraft) => "🛩 [LIGHT-AIR]",
                                AssetKind::GroundFleet => "🚚 [FLEET]",
                            };

                            ui.label(RichText::new(icon).strong().color(crate::theme::ACCENT_PRIMARY).size(12.0));
                            ui.label(RichText::new(&asset.name).strong().size(14.0).color(Color32::WHITE));
                            ui.label(RichText::new(format!("({})", asset.identifier)).color(crate::theme::TEXT_MUTED).size(11.0));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (source_text, source_color) = match asset.signal_source {
                                    TelemetrySignalSource::AisVhfClassA => ("AIS VHF Class A", Color32::from_rgb(52, 211, 153)),
                                    TelemetrySignalSource::AisVhfClassB => ("AIS VHF Class B", Color32::from_rgb(52, 211, 153)),
                                    TelemetrySignalSource::Adsb1090Mhz => ("ADS-B 1090MHz", Color32::from_rgb(96, 165, 250)),
                                    TelemetrySignalSource::GpsSatellite => ("GPS Satellite", Color32::from_rgb(167, 139, 250)),
                                    TelemetrySignalSource::DeadReckoningPredictive => ("Dead-Reckoning (Offline)", Color32::from_rgb(251, 191, 36)),
                                };
                                ui.label(RichText::new(source_text).strong().size(11.0).color(source_color));
                            });
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(format!("Συντεταγμένες: {:.4}° N, {:.4}° E", asset.position.latitude, asset.position.longitude));
                            ui.separator();
                            ui.label(format!("Ταχύτητα: {:.1} kn", asset.position.speed_knots));
                            ui.separator();
                            ui.label(format!("Πορεία: {:.0}°", asset.position.heading_degrees));
                            if let Some(alt) = asset.position.altitude_feet {
                                ui.separator();
                                ui.label(format!("Υψόμετρο: {:.0} ft", alt));
                            }
                        });
                    });

                if card_resp.response.interact(egui::Sense::click()).clicked() {
                    state.selected_asset_id = Some(asset.asset_id.clone());
                }
                ui.add_space(8.0);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_radar_state_defaults() {
        let state = FleetRadarState::default();
        assert_eq!(state.filter, FleetFilter::All);
        assert!(!state.assets.is_empty());
        assert_eq!(state.simulation_step_count, 0);
    }

    #[test]
    fn test_fleet_radar_dead_reckon_simulation() {
        let mut state = FleetRadarState::default();
        let initial_lat = state.assets[0].position.latitude;
        state.simulate_dead_reckoning_step();

        assert_eq!(state.simulation_step_count, 1);
        assert_eq!(state.assets[0].signal_source, TelemetrySignalSource::DeadReckoningPredictive);
        // Position changed after 60s at 24.2 knots
        assert_ne!(state.assets[0].position.latitude, initial_lat);
    }
}

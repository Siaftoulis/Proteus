//! Unified Telemetry & Geolocation Tracking Engine for Proteus.
//! Standardized offline-first tracking for Maritime (AIS), Aviation (ADS-B), and Ground Fleet.
//! Includes predictive dead-reckoning extrapolation for signal dead zones.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VesselCategory {
    Ferry,
    ContainerCargo,
    Tanker,
    FishingCaique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AircraftCategory {
    PassengerAirliner,
    CargoTransport,
    LightAircraft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    Maritime(VesselCategory),
    Aviation(AircraftCategory),
    GroundFleet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetrySignalSource {
    AisVhfClassA,
    AisVhfClassB,
    Adsb1090Mhz,
    GpsSatellite,
    DeadReckoningPredictive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub speed_knots: f64,
    pub heading_degrees: f64,
    pub altitude_feet: Option<f64>,
}

impl AssetPosition {
    pub fn new(lat: f64, lon: f64, speed_knots: f64, heading: f64, altitude: Option<f64>) -> Self {
        Self {
            latitude: lat.clamp(-90.0, 90.0),
            longitude: lon.clamp(-180.0, 180.0),
            speed_knots: speed_knots.max(0.0),
            heading_degrees: heading.rem_euclid(360.0),
            altitude_feet: altitude,
        }
    }

    /// Extrapolates next position using spherical dead-reckoning when telemetry drops out.
    /// Distance (NM) = speed_knots * (elapsed_secs / 3600.0).
    pub fn extrapolate_dead_reckoning(&self, elapsed_seconds: f64) -> Self {
        if self.speed_knots <= 0.01 || elapsed_seconds <= 0.0 {
            return self.clone();
        }

        let dist_nm = self.speed_knots * (elapsed_seconds / 3600.0);
        let heading_rad = self.heading_degrees.to_radians();
        let current_lat_rad = self.latitude.to_radians();

        // 1 NM = 1/60 degree of latitude
        let delta_lat = (dist_nm * heading_rad.cos()) / 60.0;
        let new_lat = (self.latitude + delta_lat).clamp(-90.0, 90.0);

        // Longitude convergence factor = cos(lat)
        let cos_lat = current_lat_rad.cos().abs().max(0.0001);
        let delta_lon = (dist_nm * heading_rad.sin()) / (60.0 * cos_lat);
        let mut new_lon = self.longitude + delta_lon;
        if new_lon > 180.0 {
            new_lon -= 360.0;
        } else if new_lon < -180.0 {
            new_lon += 360.0;
        }

        Self {
            latitude: new_lat,
            longitude: new_lon,
            speed_knots: self.speed_knots,
            heading_degrees: self.heading_degrees,
            altitude_feet: self.altitude_feet,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackedAsset {
    pub asset_id: String,
    pub name: String,
    pub identifier: String, // IMO, MMSI, Tail Number, or License Plate
    pub kind: AssetKind,
    pub position: AssetPosition,
    pub signal_source: TelemetrySignalSource,
    pub destination: Option<String>,
    pub eta_epoch: Option<i64>,
    pub last_seen_epoch: i64,
    pub is_online: bool,
}

impl TrackedAsset {
    pub fn new_vessel(
        id: impl Into<String>,
        name: impl Into<String>,
        mmsi_or_imo: impl Into<String>,
        category: VesselCategory,
        position: AssetPosition,
        source: TelemetrySignalSource,
        epoch: i64,
    ) -> Self {
        Self {
            asset_id: id.into(),
            name: name.into(),
            identifier: mmsi_or_imo.into(),
            kind: AssetKind::Maritime(category),
            position,
            signal_source: source,
            destination: None,
            eta_epoch: None,
            last_seen_epoch: epoch,
            is_online: true,
        }
    }

    pub fn new_aircraft(
        id: impl Into<String>,
        name: impl Into<String>,
        tail_number: impl Into<String>,
        category: AircraftCategory,
        position: AssetPosition,
        epoch: i64,
    ) -> Self {
        Self {
            asset_id: id.into(),
            name: name.into(),
            identifier: tail_number.into(),
            kind: AssetKind::Aviation(category),
            position,
            signal_source: TelemetrySignalSource::Adsb1090Mhz,
            destination: None,
            eta_epoch: None,
            last_seen_epoch: epoch,
            is_online: true,
        }
    }

    /// Updates current position or applies dead-reckoning if offline signal is missing.
    pub fn update_or_dead_reckon(&mut self, current_epoch: i64) {
        let elapsed = (current_epoch - self.last_seen_epoch).max(0) as f64;
        if elapsed > 15.0 && self.is_online {
            self.position = self.position.extrapolate_dead_reckoning(elapsed);
            self.signal_source = TelemetrySignalSource::DeadReckoningPredictive;
            self.last_seen_epoch = current_epoch;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_reckoning_north_movement() {
        // Heading 0 deg (North), 60 knots speed
        // In 1 hour (3600s), should travel 60 NM = 1.0 degree of latitude
        let initial = AssetPosition::new(37.0, 25.0, 60.0, 0.0, None);
        let predicted = initial.extrapolate_dead_reckoning(3600.0);

        assert!((predicted.latitude - 38.0).abs() < 0.01);
        assert!((predicted.longitude - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_dead_reckoning_stationary_asset() {
        let initial = AssetPosition::new(37.98, 23.72, 0.0, 180.0, None);
        let predicted = initial.extrapolate_dead_reckoning(7200.0);

        assert_eq!(predicted.latitude, 37.98);
        assert_eq!(predicted.longitude, 23.72);
    }

    #[test]
    fn test_tracked_asset_dead_reckon_transition() {
        let mut vessel = TrackedAsset::new_vessel(
            "VSL-01",
            "Blue Star Delos",
            "MMSI-239123400",
            VesselCategory::Ferry,
            AssetPosition::new(37.5, 24.5, 24.0, 90.0, None),
            TelemetrySignalSource::AisVhfClassA,
            1000,
        );

        // Advance time by 60 seconds without live packet
        vessel.update_or_dead_reckon(1060);
        assert_eq!(vessel.signal_source, TelemetrySignalSource::DeadReckoningPredictive);
        assert_eq!(vessel.last_seen_epoch, 1060);
        assert!(vessel.position.longitude > 24.5);
    }
}

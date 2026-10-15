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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeofenceZoneType {
    Warehouse,
    CustomerSite,
    ServiceHub,
    RestrictedZone,
}

impl GeofenceZoneType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Warehouse => "WAREHOUSE",
            Self::CustomerSite => "CUSTOMER_SITE",
            Self::ServiceHub => "SERVICE_HUB",
            Self::RestrictedZone => "RESTRICTED_ZONE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "WAREHOUSE" => Self::Warehouse,
            "CUSTOMER_SITE" => Self::CustomerSite,
            "SERVICE_HUB" => Self::ServiceHub,
            _ => Self::RestrictedZone,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeofenceZone {
    pub zone_id: String,
    pub name: String,
    pub zone_type: GeofenceZoneType,
    pub center_latitude: f64,
    pub center_longitude: f64,
    pub radius_meters: f64,
    pub speed_limit_kmh: Option<f64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetVehicle {
    pub vehicle_id: String,
    pub plate_number: String,
    pub model: String,
    pub driver_name: String,
    pub assigned_van_stock_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleBreadcrumb {
    pub breadcrumb_id: String,
    pub vehicle_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub speed_kmh: f64,
    pub heading_degrees: f64,
    pub battery_percentage: Option<f64>,
    pub odometer_km: f64,
    pub recorded_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeofenceEventType {
    ZoneEnter,
    ZoneExit,
    SpeedViolation,
}

impl GeofenceEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ZoneEnter => "ENTER",
            Self::ZoneExit => "EXIT",
            Self::SpeedViolation => "SPEED_VIOLATION",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ENTER" => Self::ZoneEnter,
            "EXIT" => Self::ZoneExit,
            _ => Self::SpeedViolation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeofenceEvent {
    pub event_id: String,
    pub vehicle_id: String,
    pub zone_id: Option<String>,
    pub event_type: GeofenceEventType,
    pub latitude: f64,
    pub longitude: f64,
    pub speed_kmh: f64,
    pub recorded_at: i64,
}

/// Sovereign Great-Circle Haversine distance in meters between two GPS coordinates.
/// Earth mean radius R = 6,371,000 meters.
pub fn haversine_distance_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6_371_000.0;
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let delta_phi = (lat2 - lat1).to_radians();
    let delta_lambda = (lon2 - lon1).to_radians();

    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).max(0.0).sqrt());

    r * c
}

/// Checks whether GPS coordinate is contained within a circular geofence zone.
pub fn is_point_in_zone(lat: f64, lon: f64, zone: &GeofenceZone) -> bool {
    let dist = haversine_distance_meters(lat, lon, zone.center_latitude, zone.center_longitude);
    dist <= zone.radius_meters
}

/// Initializes sovereign SQLite tables for fleet vehicles, breadcrumbs, zones, and geofence events.
pub fn init_telemetry_schema(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS fleet_vehicles (
            vehicle_id TEXT PRIMARY KEY,
            plate_number TEXT NOT NULL UNIQUE,
            model TEXT NOT NULL,
            driver_name TEXT NOT NULL,
            assigned_van_stock_id TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS telemetry_breadcrumbs (
            breadcrumb_id TEXT PRIMARY KEY,
            vehicle_id TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            speed_kmh REAL NOT NULL,
            heading_degrees REAL NOT NULL,
            battery_percentage REAL,
            odometer_km REAL NOT NULL,
            recorded_at INTEGER NOT NULL,
            FOREIGN KEY(vehicle_id) REFERENCES fleet_vehicles(vehicle_id)
        );
        CREATE INDEX IF NOT EXISTS idx_telemetry_breadcrumbs_vehicle ON telemetry_breadcrumbs(vehicle_id, recorded_at);
        CREATE TABLE IF NOT EXISTS geofence_zones (
            zone_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            zone_type TEXT NOT NULL,
            center_latitude REAL NOT NULL,
            center_longitude REAL NOT NULL,
            radius_meters REAL NOT NULL,
            speed_limit_kmh REAL,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS geofence_events (
            event_id TEXT PRIMARY KEY,
            vehicle_id TEXT NOT NULL,
            zone_id TEXT,
            event_type TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            speed_kmh REAL NOT NULL,
            recorded_at INTEGER NOT NULL,
            FOREIGN KEY(vehicle_id) REFERENCES fleet_vehicles(vehicle_id)
        );
        CREATE INDEX IF NOT EXISTS idx_geofence_events_vehicle ON geofence_events(vehicle_id, recorded_at);"
    )?;
    Ok(())
}

pub fn register_fleet_vehicle(conn: &rusqlite::Connection, vehicle: &FleetVehicle) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO fleet_vehicles (vehicle_id, plate_number, model, driver_name, assigned_van_stock_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(vehicle_id) DO UPDATE SET
            plate_number = excluded.plate_number,
            model = excluded.model,
            driver_name = excluded.driver_name,
            assigned_van_stock_id = excluded.assigned_van_stock_id;",
        rusqlite::params![
            vehicle.vehicle_id,
            vehicle.plate_number,
            vehicle.model,
            vehicle.driver_name,
            vehicle.assigned_van_stock_id,
            vehicle.created_at,
        ],
    )?;
    Ok(())
}

pub fn create_geofence_zone(conn: &rusqlite::Connection, zone: &GeofenceZone) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO geofence_zones (zone_id, name, zone_type, center_latitude, center_longitude, radius_meters, speed_limit_kmh, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(zone_id) DO UPDATE SET
            name = excluded.name,
            zone_type = excluded.zone_type,
            center_latitude = excluded.center_latitude,
            center_longitude = excluded.center_longitude,
            radius_meters = excluded.radius_meters,
            speed_limit_kmh = excluded.speed_limit_kmh;",
        rusqlite::params![
            zone.zone_id,
            zone.name,
            zone.zone_type.as_str(),
            zone.center_latitude,
            zone.center_longitude,
            zone.radius_meters,
            zone.speed_limit_kmh,
            zone.created_at,
        ],
    )?;
    Ok(())
}

pub fn list_fleet_vehicles(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<FleetVehicle>> {
    let mut stmt = conn.prepare(
        "SELECT vehicle_id, plate_number, model, driver_name, assigned_van_stock_id, created_at
         FROM fleet_vehicles ORDER BY plate_number ASC;"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(FleetVehicle {
            vehicle_id: row.get(0)?,
            plate_number: row.get(1)?,
            model: row.get(2)?,
            driver_name: row.get(3)?,
            assigned_van_stock_id: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn list_geofence_zones(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<GeofenceZone>> {
    let mut stmt = conn.prepare(
        "SELECT zone_id, name, zone_type, center_latitude, center_longitude, radius_meters, speed_limit_kmh, created_at
         FROM geofence_zones ORDER BY name ASC;"
    )?;
    let rows = stmt.query_map([], |row| {
        let type_str: String = row.get(2)?;
        Ok(GeofenceZone {
            zone_id: row.get(0)?,
            name: row.get(1)?,
            zone_type: GeofenceZoneType::from_str(&type_str),
            center_latitude: row.get(3)?,
            center_longitude: row.get(4)?,
            radius_meters: row.get(5)?,
            speed_limit_kmh: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_reckoning_north_movement() {
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
        vessel.update_or_dead_reckon(1060);
        assert_eq!(vessel.signal_source, TelemetrySignalSource::DeadReckoningPredictive);
        assert_eq!(vessel.last_seen_epoch, 1060);
        assert!(vessel.position.longitude > 24.5);
    }

    #[test]
    fn test_haversine_distance_athens_to_piraeus() {
        let d = haversine_distance_meters(37.9753, 23.7361, 37.9429, 23.6469);
        assert!((d - 8550.0).abs() < 100.0, "Expected ~8.55km, got {}", d);
    }

    #[test]
    fn test_geofence_zone_containment() {
        let zone = GeofenceZone {
            zone_id: "ZONE-ATH-01".into(),
            name: "Athens Central Hub".into(),
            zone_type: GeofenceZoneType::Warehouse,
            center_latitude: 37.9838,
            center_longitude: 23.7275,
            radius_meters: 500.0,
            speed_limit_kmh: Some(30.0),
            created_at: 1000,
        };

        assert!(is_point_in_zone(37.9845, 23.7280, &zone));
        assert!(!is_point_in_zone(38.0000, 23.7275, &zone));
    }

    #[test]
    fn test_telemetry_schema_and_crud() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        init_telemetry_schema(&conn).unwrap();
        init_telemetry_schema(&conn).unwrap();

        let vehicle = FleetVehicle {
            vehicle_id: "VEH-VAN-01".into(),
            plate_number: "IXX-9821".into(),
            model: "Ford Transit Custom".into(),
            driver_name: "Κώστας Παπαδόπουλος".into(),
            assigned_van_stock_id: Some("VAN-STOCK-01".into()),
            created_at: 1700000000,
        };
        register_fleet_vehicle(&conn, &vehicle).unwrap();

        let vehicles = list_fleet_vehicles(&conn).unwrap();
        assert_eq!(vehicles.len(), 1);
        assert_eq!(vehicles[0].plate_number, "IXX-9821");

        let zone = GeofenceZone {
            zone_id: "ZONE-WH-01".into(),
            name: "Κεντρική Αποθήκη Ασπροπύργου".into(),
            zone_type: GeofenceZoneType::Warehouse,
            center_latitude: 38.0583,
            center_longitude: 23.5901,
            radius_meters: 1500.0,
            speed_limit_kmh: Some(40.0),
            created_at: 1700000000,
        };
        create_geofence_zone(&conn, &zone).unwrap();

        let zones = list_geofence_zones(&conn).unwrap();
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].name, "Κεντρική Αποθήκη Ασπροπύργου");
        assert_eq!(zones[0].zone_type, GeofenceZoneType::Warehouse);
    }
}

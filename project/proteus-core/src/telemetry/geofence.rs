//! Sovereign Ground Fleet Telemetry & Native Geofencing Engine (`proteus-core::telemetry::geofence`).
//! Pure spatial trigonometry and SQLite audit logging for delivery vans, service trucks, and ground assets.

use serde::{Deserialize, Serialize};

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

/// Ingests a new vehicle telemetry breadcrumb, records it to SQLite,
/// evaluates geofence zone boundary transitions (Enter / Exit) and speed violations,
/// and logs triggered events to the persistent audit table.
pub fn ingest_vehicle_breadcrumb(
    conn: &rusqlite::Connection,
    breadcrumb: &VehicleBreadcrumb,
) -> rusqlite::Result<Vec<GeofenceEvent>> {
    // 1. Fetch previous location for this vehicle
    let prev_location: Option<(f64, f64)> = conn
        .query_row(
            "SELECT latitude, longitude FROM telemetry_breadcrumbs
             WHERE vehicle_id = ?1 AND recorded_at < ?2
             ORDER BY recorded_at DESC LIMIT 1;",
            rusqlite::params![breadcrumb.vehicle_id, breadcrumb.recorded_at],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    // 2. Insert new breadcrumb
    conn.execute(
        "INSERT INTO telemetry_breadcrumbs (
            breadcrumb_id, vehicle_id, latitude, longitude, speed_kmh, heading_degrees, battery_percentage, odometer_km, recorded_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        rusqlite::params![
            breadcrumb.breadcrumb_id,
            breadcrumb.vehicle_id,
            breadcrumb.latitude,
            breadcrumb.longitude,
            breadcrumb.speed_kmh,
            breadcrumb.heading_degrees,
            breadcrumb.battery_percentage,
            breadcrumb.odometer_km,
            breadcrumb.recorded_at,
        ],
    )?;

    // 3. Load all active geofence zones
    let zones = list_geofence_zones(conn)?;
    let mut triggered_events = Vec::new();

    for zone in &zones {
        let prev_inside = prev_location
            .map(|(lat, lon)| is_point_in_zone(lat, lon, zone))
            .unwrap_or(false);
        let curr_inside = is_point_in_zone(breadcrumb.latitude, breadcrumb.longitude, zone);

        // Transition: Enter
        if !prev_inside && curr_inside {
            let event = GeofenceEvent {
                event_id: format!("EVT-ENT-{}-{}", breadcrumb.vehicle_id, breadcrumb.recorded_at),
                vehicle_id: breadcrumb.vehicle_id.clone(),
                zone_id: Some(zone.zone_id.clone()),
                event_type: GeofenceEventType::ZoneEnter,
                latitude: breadcrumb.latitude,
                longitude: breadcrumb.longitude,
                speed_kmh: breadcrumb.speed_kmh,
                recorded_at: breadcrumb.recorded_at,
            };
            triggered_events.push(event);
        }

        // Transition: Exit
        if prev_inside && !curr_inside {
            let event = GeofenceEvent {
                event_id: format!("EVT-EXT-{}-{}", breadcrumb.vehicle_id, breadcrumb.recorded_at),
                vehicle_id: breadcrumb.vehicle_id.clone(),
                zone_id: Some(zone.zone_id.clone()),
                event_type: GeofenceEventType::ZoneExit,
                latitude: breadcrumb.latitude,
                longitude: breadcrumb.longitude,
                speed_kmh: breadcrumb.speed_kmh,
                recorded_at: breadcrumb.recorded_at,
            };
            triggered_events.push(event);
        }

        // Condition: Speed Violation within Zone
        if curr_inside {
            if let Some(limit) = zone.speed_limit_kmh {
                if breadcrumb.speed_kmh > limit {
                    let event = GeofenceEvent {
                        event_id: format!("EVT-SPD-{}-{}", breadcrumb.vehicle_id, breadcrumb.recorded_at),
                        vehicle_id: breadcrumb.vehicle_id.clone(),
                        zone_id: Some(zone.zone_id.clone()),
                        event_type: GeofenceEventType::SpeedViolation,
                        latitude: breadcrumb.latitude,
                        longitude: breadcrumb.longitude,
                        speed_kmh: breadcrumb.speed_kmh,
                        recorded_at: breadcrumb.recorded_at,
                    };
                    triggered_events.push(event);
                }
            }
        }
    }

    // 4. Persist all generated events
    for ev in &triggered_events {
        conn.execute(
            "INSERT INTO geofence_events (
                event_id, vehicle_id, zone_id, event_type, latitude, longitude, speed_kmh, recorded_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
            rusqlite::params![
                ev.event_id,
                ev.vehicle_id,
                ev.zone_id,
                ev.event_type.as_str(),
                ev.latitude,
                ev.longitude,
                ev.speed_kmh,
                ev.recorded_at,
            ],
        )?;
    }

    Ok(triggered_events)
}

pub fn list_vehicle_breadcrumbs(
    conn: &rusqlite::Connection,
    vehicle_id: &str,
    limit: usize,
) -> rusqlite::Result<Vec<VehicleBreadcrumb>> {
    let mut stmt = conn.prepare(
        "SELECT breadcrumb_id, vehicle_id, latitude, longitude, speed_kmh, heading_degrees, battery_percentage, odometer_km, recorded_at
         FROM telemetry_breadcrumbs
         WHERE vehicle_id = ?1
         ORDER BY recorded_at DESC
         LIMIT ?2;",
    )?;
    let rows = stmt.query_map(rusqlite::params![vehicle_id, limit as i64], |row| {
        Ok(VehicleBreadcrumb {
            breadcrumb_id: row.get(0)?,
            vehicle_id: row.get(1)?,
            latitude: row.get(2)?,
            longitude: row.get(3)?,
            speed_kmh: row.get(4)?,
            heading_degrees: row.get(5)?,
            battery_percentage: row.get(6)?,
            odometer_km: row.get(7)?,
            recorded_at: row.get(8)?,
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn list_geofence_events(
    conn: &rusqlite::Connection,
    vehicle_id: Option<&str>,
    limit: usize,
) -> rusqlite::Result<Vec<GeofenceEvent>> {
    let mut list = Vec::new();
    if let Some(vid) = vehicle_id {
        let mut stmt = conn.prepare(
            "SELECT event_id, vehicle_id, zone_id, event_type, latitude, longitude, speed_kmh, recorded_at
             FROM geofence_events
             WHERE vehicle_id = ?1
             ORDER BY recorded_at DESC
             LIMIT ?2;",
        )?;
        let rows = stmt.query_map(rusqlite::params![vid, limit as i64], |row| {
            let t_str: String = row.get(3)?;
            Ok(GeofenceEvent {
                event_id: row.get(0)?,
                vehicle_id: row.get(1)?,
                zone_id: row.get(2)?,
                event_type: GeofenceEventType::from_str(&t_str),
                latitude: row.get(4)?,
                longitude: row.get(5)?,
                speed_kmh: row.get(6)?,
                recorded_at: row.get(7)?,
            })
        })?;
        for r in rows {
            list.push(r?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT event_id, vehicle_id, zone_id, event_type, latitude, longitude, speed_kmh, recorded_at
             FROM geofence_events
             ORDER BY recorded_at DESC
             LIMIT ?1;",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
            let t_str: String = row.get(3)?;
            Ok(GeofenceEvent {
                event_id: row.get(0)?,
                vehicle_id: row.get(1)?,
                zone_id: row.get(2)?,
                event_type: GeofenceEventType::from_str(&t_str),
                latitude: row.get(4)?,
                longitude: row.get(5)?,
                speed_kmh: row.get(6)?,
                recorded_at: row.get(7)?,
            })
        })?;
        for r in rows {
            list.push(r?);
        }
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance_calculation() {
        let d = haversine_distance_meters(37.9753, 23.7361, 37.9429, 23.6469);
        assert!((d - 8550.0).abs() < 100.0);
    }

    #[test]
    fn test_breadcrumb_ingestion_and_geofence_transitions() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        init_telemetry_schema(&conn).unwrap();

        let vehicle = FleetVehicle {
            vehicle_id: "VAN-01".into(),
            plate_number: "IXX-1234".into(),
            model: "Ford Transit".into(),
            driver_name: "Νίκος Αντωνίου".into(),
            assigned_van_stock_id: Some("STOCK-VAN-01".into()),
            created_at: 1000,
        };
        register_fleet_vehicle(&conn, &vehicle).unwrap();

        let zone = GeofenceZone {
            zone_id: "ZONE-ATH".into(),
            name: "Athens Hub".into(),
            zone_type: GeofenceZoneType::Warehouse,
            center_latitude: 37.9838,
            center_longitude: 23.7275,
            radius_meters: 500.0,
            speed_limit_kmh: Some(30.0),
            created_at: 1000,
        };
        create_geofence_zone(&conn, &zone).unwrap();

        // 1. Initial point far away outside zone (lat: 38.1500, distance > 18km)
        let b1 = VehicleBreadcrumb {
            breadcrumb_id: "BC-1".into(),
            vehicle_id: "VAN-01".into(),
            latitude: 38.1500,
            longitude: 23.7275,
            speed_kmh: 50.0,
            heading_degrees: 180.0,
            battery_percentage: Some(95.0),
            odometer_km: 12050.0,
            recorded_at: 1010,
        };
        let ev1 = ingest_vehicle_breadcrumb(&conn, &b1).unwrap();
        assert!(ev1.is_empty(), "Outside zone should trigger no events");

        // 2. Point inside zone with compliant speed (20 km/h) -> ZoneEnter event
        let b2 = VehicleBreadcrumb {
            breadcrumb_id: "BC-2".into(),
            vehicle_id: "VAN-01".into(),
            latitude: 37.9839,
            longitude: 23.7276,
            speed_kmh: 20.0,
            heading_degrees: 180.0,
            battery_percentage: Some(94.0),
            odometer_km: 12070.0,
            recorded_at: 1020,
        };
        let ev2 = ingest_vehicle_breadcrumb(&conn, &b2).unwrap();
        assert_eq!(ev2.len(), 1);
        assert_eq!(ev2[0].event_type, GeofenceEventType::ZoneEnter);
        assert_eq!(ev2[0].zone_id.as_deref(), Some("ZONE-ATH"));

        // 3. Point inside zone with speeding (55 km/h vs limit 30 km/h) -> SpeedViolation event
        let b3 = VehicleBreadcrumb {
            breadcrumb_id: "BC-3".into(),
            vehicle_id: "VAN-01".into(),
            latitude: 37.9840,
            longitude: 23.7277,
            speed_kmh: 55.0,
            heading_degrees: 90.0,
            battery_percentage: Some(93.0),
            odometer_km: 12071.0,
            recorded_at: 1030,
        };
        let ev3 = ingest_vehicle_breadcrumb(&conn, &b3).unwrap();
        assert_eq!(ev3.len(), 1);
        assert_eq!(ev3[0].event_type, GeofenceEventType::SpeedViolation);

        // 4. Point exiting zone (back to 38.0500) -> ZoneExit event
        let b4 = VehicleBreadcrumb {
            breadcrumb_id: "BC-4".into(),
            vehicle_id: "VAN-01".into(),
            latitude: 38.0500,
            longitude: 23.7275,
            speed_kmh: 60.0,
            heading_degrees: 0.0,
            battery_percentage: Some(92.0),
            odometer_km: 12080.0,
            recorded_at: 1040,
        };
        let ev4 = ingest_vehicle_breadcrumb(&conn, &b4).unwrap();
        assert_eq!(ev4.len(), 1);
        assert_eq!(ev4[0].event_type, GeofenceEventType::ZoneExit);

        // Verify stored events and breadcrumbs
        let all_events = list_geofence_events(&conn, Some("VAN-01"), 10).unwrap();
        assert_eq!(all_events.len(), 3);

        let crumbs = list_vehicle_breadcrumbs(&conn, "VAN-01", 10).unwrap();
        assert_eq!(crumbs.len(), 4);
    }
}

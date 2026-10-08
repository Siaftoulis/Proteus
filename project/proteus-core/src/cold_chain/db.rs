//! SQLite Persistence & Telemetry Ingestion Engine for Cold Chain HACCP.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::types::{
    BreachSeverity, BreachStatus, ColdChainSensor, ColdStorageType, HaccpBreachEvent,
    TelemetryReading,
};

/// Computes SHA-256 hexadecimal hash string over a byte slice.
pub(crate) fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Initializes cold chain tables and composite indexes in SQLite.
pub fn init_cold_chain_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS cold_chain_sensors (
            sensor_id TEXT PRIMARY KEY, target_id TEXT NOT NULL, storage_type TEXT NOT NULL,
            min_temp_celsius REAL NOT NULL, max_temp_celsius REAL NOT NULL,
            min_humidity_pct REAL, max_humidity_pct REAL, is_active INTEGER NOT NULL DEFAULT 1,
            last_reading_epoch INTEGER
        );
        CREATE TABLE IF NOT EXISTS cold_chain_logs (
            reading_id TEXT PRIMARY KEY, sensor_id TEXT NOT NULL, target_id TEXT NOT NULL,
            temperature_celsius REAL NOT NULL, humidity_pct REAL,
            door_open INTEGER NOT NULL DEFAULT 0, battery_level_pct INTEGER,
            recorded_at TEXT NOT NULL, recorded_epoch INTEGER NOT NULL, merkle_hash TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS haccp_breach_events (
            breach_id TEXT PRIMARY KEY, sensor_id TEXT NOT NULL, target_id TEXT NOT NULL,
            severity TEXT NOT NULL, status TEXT NOT NULL, excursion_temp REAL NOT NULL,
            threshold_limit REAL NOT NULL, started_at TEXT NOT NULL, resolved_at TEXT,
            corrective_action_note TEXT, operator_name TEXT, merkle_hash TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_cold_logs_target_epoch ON cold_chain_logs(target_id, recorded_epoch);
        CREATE INDEX IF NOT EXISTS idx_haccp_breaches_status ON haccp_breach_events(status, target_id);",
    )
}

/// Registers or updates an IoT cold-chain sensor.
pub fn register_cold_sensor(conn: &Connection, sensor: &ColdChainSensor) -> Result<()> {
    conn.execute(
        "INSERT INTO cold_chain_sensors (
            sensor_id, target_id, storage_type, min_temp_celsius, max_temp_celsius,
            min_humidity_pct, max_humidity_pct, is_active, last_reading_epoch
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(sensor_id) DO UPDATE SET
            target_id = excluded.target_id, storage_type = excluded.storage_type,
            min_temp_celsius = excluded.min_temp_celsius, max_temp_celsius = excluded.max_temp_celsius,
            min_humidity_pct = excluded.min_humidity_pct, max_humidity_pct = excluded.max_humidity_pct,
            is_active = excluded.is_active;",
        params![
            sensor.sensor_id, sensor.target_id, sensor.storage_type.as_str(),
            sensor.min_temp_celsius, sensor.max_temp_celsius,
            sensor.min_humidity_pct, sensor.max_humidity_pct,
            if sensor.is_active { 1 } else { 0 }, sensor.last_reading_epoch,
        ],
    )?;
    Ok(())
}

/// Retrieves all registered cold chain sensors.
pub fn list_cold_sensors(conn: &Connection) -> Result<Vec<ColdChainSensor>> {
    let mut stmt = conn.prepare(
        "SELECT sensor_id, target_id, storage_type, min_temp_celsius, max_temp_celsius,
                min_humidity_pct, max_humidity_pct, is_active, last_reading_epoch
         FROM cold_chain_sensors ORDER BY target_id ASC, sensor_id ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        let storage_str: String = row.get(2)?;
        let active_int: i32 = row.get(7)?;
        Ok(ColdChainSensor {
            sensor_id: row.get(0)?,
            target_id: row.get(1)?,
            storage_type: ColdStorageType::from_str(&storage_str),
            min_temp_celsius: row.get(3)?,
            max_temp_celsius: row.get(4)?,
            min_humidity_pct: row.get(5)?,
            max_humidity_pct: row.get(6)?,
            is_active: active_int == 1,
            last_reading_epoch: row.get(8)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Ingests a live or buffered telemetry reading, seals it with SHA-256 Merkle hash,
/// and automatically detects/creates HACCP breach events on temperature excursion.
pub fn ingest_telemetry_reading(
    conn: &Connection,
    sensor_id: &str,
    temp_celsius: f64,
    humidity_pct: Option<f64>,
    door_open: bool,
    battery_pct: Option<u8>,
) -> Result<(TelemetryReading, Option<HaccpBreachEvent>)> {
    let mut sensor_opt = None;
    {
        let mut stmt = conn.prepare(
            "SELECT sensor_id, target_id, storage_type, min_temp_celsius, max_temp_celsius,
                    min_humidity_pct, max_humidity_pct, is_active, last_reading_epoch
             FROM cold_chain_sensors WHERE sensor_id = ?1",
        )?;
        let mut rows = stmt.query(params![sensor_id])?;
        if let Some(row) = rows.next()? {
            let storage_str: String = row.get(2)?;
            let active_int: i32 = row.get(7)?;
            sensor_opt = Some(ColdChainSensor {
                sensor_id: row.get(0)?,
                target_id: row.get(1)?,
                storage_type: ColdStorageType::from_str(&storage_str),
                min_temp_celsius: row.get(3)?,
                max_temp_celsius: row.get(4)?,
                min_humidity_pct: row.get(5)?,
                max_humidity_pct: row.get(6)?,
                is_active: active_int == 1,
                last_reading_epoch: row.get(8)?,
            });
        }
    }

    let sensor = sensor_opt.unwrap_or_else(|| {
        ColdChainSensor::new(sensor_id, "UNASSIGNED", ColdStorageType::Chilled)
    });

    let reading_id = format!("TLM-{}", Uuid::new_v4().simple());
    let now = Utc::now();
    let recorded_at = now.to_rfc3339();
    let recorded_epoch = now.timestamp();

    let raw_payload = format!(
        "{}:{}:{:.2}:{}:{}:{}",
        sensor.sensor_id, sensor.target_id, temp_celsius,
        door_open, recorded_at, battery_pct.unwrap_or(100)
    );
    let reading_hash = compute_sha256(raw_payload.as_bytes());

    conn.execute(
        "INSERT INTO cold_chain_logs (
            reading_id, sensor_id, target_id, temperature_celsius, humidity_pct,
            door_open, battery_level_pct, recorded_at, recorded_epoch, merkle_hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            reading_id, sensor.sensor_id, sensor.target_id, temp_celsius, humidity_pct,
            if door_open { 1 } else { 0 }, battery_pct.map(|b| b as i32),
            recorded_at, recorded_epoch, reading_hash,
        ],
    )?;

    let _ = conn.execute(
        "UPDATE cold_chain_sensors SET last_reading_epoch = ?1 WHERE sensor_id = ?2",
        params![recorded_epoch, sensor.sensor_id],
    );

    let reading = TelemetryReading {
        reading_id,
        sensor_id: sensor.sensor_id.clone(),
        target_id: sensor.target_id.clone(),
        temperature_celsius: temp_celsius,
        humidity_pct,
        door_open,
        battery_level_pct: battery_pct,
        recorded_at: recorded_at.clone(),
        recorded_epoch,
        merkle_hash: reading_hash,
    };

    let mut breach_event = None;
    let is_over = temp_celsius > sensor.max_temp_celsius;
    let is_under = temp_celsius < sensor.min_temp_celsius;

    if is_over || is_under {
        let (delta, threshold) = if is_over {
            (temp_celsius - sensor.max_temp_celsius, sensor.max_temp_celsius)
        } else {
            (sensor.min_temp_celsius - temp_celsius, sensor.min_temp_celsius)
        };

        let severity = if delta <= 2.0 {
            BreachSeverity::MinorWarning
        } else if delta <= 5.0 {
            BreachSeverity::MajorExcursion
        } else {
            BreachSeverity::CriticalSpoilage
        };

        let breach_id = format!("BRC-{}", Uuid::new_v4().simple());
        let breach_payload = format!(
            "{}:{}:{:.2}:{}:{}:{}",
            breach_id, sensor.sensor_id, temp_celsius, threshold, severity.as_str(), recorded_at
        );
        let breach_hash = compute_sha256(breach_payload.as_bytes());

        conn.execute(
            "INSERT INTO haccp_breach_events (
                breach_id, sensor_id, target_id, severity, status, excursion_temp,
                threshold_limit, started_at, resolved_at, corrective_action_note,
                operator_name, merkle_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL, NULL, ?9)",
            params![
                breach_id, sensor.sensor_id, sensor.target_id, severity.as_str(),
                BreachStatus::Active.as_str(), temp_celsius, threshold, recorded_at, breach_hash,
            ],
        )?;

        breach_event = Some(HaccpBreachEvent {
            breach_id,
            sensor_id: sensor.sensor_id,
            target_id: sensor.target_id,
            severity,
            status: BreachStatus::Active,
            excursion_temp: temp_celsius,
            threshold_limit: threshold,
            started_at: recorded_at,
            resolved_at: None,
            corrective_action_note: None,
            operator_name: None,
            merkle_hash: breach_hash,
        });
    }

    Ok((reading, breach_event))
}

/// Lists active or unacknowledged HACCP breaches.
pub fn list_active_breaches(conn: &Connection) -> Result<Vec<HaccpBreachEvent>> {
    let mut stmt = conn.prepare(
        "SELECT breach_id, sensor_id, target_id, severity, status, excursion_temp,
                threshold_limit, started_at, resolved_at, corrective_action_note,
                operator_name, merkle_hash
         FROM haccp_breach_events WHERE status IN ('ACTIVE', 'ACKNOWLEDGED')
         ORDER BY started_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        let sev_str: String = row.get(3)?;
        let stat_str: String = row.get(4)?;
        Ok(HaccpBreachEvent {
            breach_id: row.get(0)?,
            sensor_id: row.get(1)?,
            target_id: row.get(2)?,
            severity: BreachSeverity::from_str(&sev_str),
            status: BreachStatus::from_str(&stat_str),
            excursion_temp: row.get(5)?,
            threshold_limit: row.get(6)?,
            started_at: row.get(7)?,
            resolved_at: row.get(8)?,
            corrective_action_note: row.get(9)?,
            operator_name: row.get(10)?,
            merkle_hash: row.get(11)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Resolves a HACCP temperature excursion event with operator audit note.
pub fn resolve_breach_event(
    conn: &Connection,
    breach_id: &str,
    operator: &str,
    note: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE haccp_breach_events
         SET status = ?1, resolved_at = ?2, operator_name = ?3, corrective_action_note = ?4
         WHERE breach_id = ?5",
        params![BreachStatus::Resolved.as_str(), now, operator, note, breach_id],
    )?;
    Ok(())
}

/// Lists recent telemetry logs for a target vehicle or facility.
pub fn list_recent_readings(
    conn: &Connection,
    target_id: Option<&str>,
    limit: usize,
) -> Result<Vec<TelemetryReading>> {
    let mut sql = String::from(
        "SELECT reading_id, sensor_id, target_id, temperature_celsius, humidity_pct,
                door_open, battery_level_pct, recorded_at, recorded_epoch, merkle_hash
         FROM cold_chain_logs",
    );

    if target_id.is_some() {
        sql.push_str(" WHERE target_id = ?1 ");
    }
    sql.push_str(" ORDER BY recorded_epoch DESC LIMIT ?2");

    let mut stmt = conn.prepare(&sql)?;
    let limit_i64 = limit as i64;

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<TelemetryReading> {
        let door_int: i32 = row.get(5)?;
        let battery_int: Option<i32> = row.get(6)?;
        Ok(TelemetryReading {
            reading_id: row.get(0)?,
            sensor_id: row.get(1)?,
            target_id: row.get(2)?,
            temperature_celsius: row.get(3)?,
            humidity_pct: row.get(4)?,
            door_open: door_int == 1,
            battery_level_pct: battery_int.map(|b| b as u8),
            recorded_at: row.get(7)?,
            recorded_epoch: row.get(8)?,
            merkle_hash: row.get(9)?,
        })
    };

    let mut list = Vec::new();
    if let Some(target) = target_id {
        let mut rows = stmt.query(params![target, limit_i64])?;
        while let Some(row) = rows.next()? {
            list.push(map_row(row)?);
        }
    } else {
        let mut rows = stmt.query(params![limit_i64])?;
        while let Some(row) = rows.next()? {
            list.push(map_row(row)?);
        }
    }
    Ok(list)
}

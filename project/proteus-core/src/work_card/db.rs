//! Database routines and event persistence for Ergani II Work Card.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data).

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::types::{EmployeeProfile, WorkCardError, WorkCardEvent, WorkCardEventType};

pub fn init_work_card_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS work_employees (
            employee_id TEXT PRIMARY KEY,
            afm TEXT NOT NULL UNIQUE,
            amka TEXT NOT NULL UNIQUE,
            full_name TEXT NOT NULL,
            job_title TEXT NOT NULL,
            pin_code TEXT NOT NULL,
            qr_badge_token TEXT NOT NULL UNIQUE,
            is_active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS work_card_events (
            event_id TEXT PRIMARY KEY,
            employee_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            timestamp_utc INTEGER NOT NULL,
            local_time_str TEXT NOT NULL,
            is_offline_fallback INTEGER NOT NULL DEFAULT 0,
            outage_reason TEXT,
            merkle_hash TEXT NOT NULL,
            ergani_submission_id TEXT,
            sync_status TEXT NOT NULL DEFAULT 'PENDING',
            FOREIGN KEY (employee_id) REFERENCES work_employees(employee_id)
        );

        CREATE INDEX IF NOT EXISTS idx_work_events_emp ON work_card_events(employee_id);
        CREATE INDEX IF NOT EXISTS idx_work_events_sync ON work_card_events(sync_status);
        CREATE INDEX IF NOT EXISTS idx_work_events_time ON work_card_events(timestamp_utc);
        "#,
    )
}

pub fn save_employee(conn: &Connection, emp: &EmployeeProfile) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO work_employees (employee_id, afm, amka, full_name, job_title, pin_code, qr_badge_token, is_active)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(employee_id) DO UPDATE SET
            afm = excluded.afm,
            amka = excluded.amka,
            full_name = excluded.full_name,
            job_title = excluded.job_title,
            pin_code = excluded.pin_code,
            qr_badge_token = excluded.qr_badge_token,
            is_active = excluded.is_active
        "#,
        params![
            emp.employee_id,
            emp.afm,
            emp.amka,
            emp.full_name,
            emp.job_title,
            emp.pin_code,
            emp.qr_badge_token,
            if emp.is_active { 1 } else { 0 },
        ],
    )?;
    Ok(())
}

pub fn authenticate_employee_by_pin(
    conn: &Connection,
    afm: &str,
    pin: &str,
) -> Result<EmployeeProfile, WorkCardError> {
    let res = conn.query_row(
        r#"
        SELECT employee_id, afm, amka, full_name, job_title, pin_code, qr_badge_token, is_active
        FROM work_employees
        WHERE (afm = ?1 OR qr_badge_token = ?1) AND is_active = 1
        "#,
        params![afm],
        |row| {
            Ok(EmployeeProfile {
                employee_id: row.get(0)?,
                afm: row.get(1)?,
                amka: row.get(2)?,
                full_name: row.get(3)?,
                job_title: row.get(4)?,
                pin_code: row.get(5)?,
                qr_badge_token: row.get(6)?,
                is_active: row.get::<_, i64>(7)? == 1,
            })
        },
    );

    match res {
        Ok(emp) => {
            if emp.pin_code == pin {
                Ok(emp)
            } else {
                Err(WorkCardError::InvalidPin(emp.employee_id))
            }
        }
        Err(_) => Err(WorkCardError::EmployeeNotFound),
    }
}

pub fn record_clock_event(
    conn: &Connection,
    employee_id: &str,
    event_type: WorkCardEventType,
    is_offline: bool,
    outage_reason: Option<&str>,
) -> Result<WorkCardEvent, WorkCardError> {
    let now_utc = Utc::now();
    let timestamp_ms = now_utc.timestamp_millis();
    let local_time_str = now_utc.format("%Y-%m-%d %H:%M:%S").to_string();
    let event_id = Uuid::now_v7().to_string();

    let prev_hash: String = conn
        .query_row(
            "SELECT merkle_hash FROM work_card_events ORDER BY timestamp_utc DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "0000000000000000000000000000000000000000000000000000000000000000".to_string());

    let payload = format!(
        "{}:{}:{}:{}:{}:{}",
        prev_hash, event_id, employee_id, event_type.as_str(), timestamp_ms, is_offline
    );
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let merkle_hash = format!("{:x}", hasher.finalize());

    let event = WorkCardEvent {
        event_id: event_id.clone(),
        employee_id: employee_id.to_string(),
        event_type,
        timestamp_utc: timestamp_ms,
        local_time_str: local_time_str.clone(),
        is_offline_fallback: is_offline,
        outage_reason: outage_reason.map(|s| s.to_string()),
        merkle_hash: merkle_hash.clone(),
        ergani_submission_id: None,
        sync_status: if is_offline { "PENDING_OUTAGE_SYNC".to_string() } else { "PENDING".to_string() },
    };

    conn.execute(
        r#"
        INSERT INTO work_card_events (
            event_id, employee_id, event_type, timestamp_utc, local_time_str,
            is_offline_fallback, outage_reason, merkle_hash, ergani_submission_id, sync_status
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            event.event_id,
            event.employee_id,
            event.event_type.as_str(),
            event.timestamp_utc,
            event.local_time_str,
            if event.is_offline_fallback { 1 } else { 0 },
            event.outage_reason,
            event.merkle_hash,
            event.ergani_submission_id,
            event.sync_status,
        ],
    ).map_err(|e| WorkCardError::DatabaseError(e.to_string()))?;

    Ok(event)
}

pub fn calculate_shift_duration_hours(
    conn: &Connection,
    employee_id: &str,
    date_str: &str,
) -> Result<(f64, f64), WorkCardError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT event_type, timestamp_utc
            FROM work_card_events
            WHERE employee_id = ?1 AND local_time_str LIKE ?2
            ORDER BY timestamp_utc ASC
            "#,
        )
        .map_err(|e| WorkCardError::DatabaseError(e.to_string()))?;

    let filter = format!("{}%", date_str);
    let rows = stmt
        .query_map(params![employee_id, filter], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map_err(|e| WorkCardError::DatabaseError(e.to_string()))?;

    let mut total_worked_ms = 0i64;
    let mut current_clock_in: Option<i64> = None;

    for row in rows {
        let (ev_type, ts) = row.map_err(|e| WorkCardError::DatabaseError(e.to_string()))?;
        match ev_type.as_str() {
            "CLOCK_IN" | "BREAK_END" => {
                current_clock_in = Some(ts);
            }
            "CLOCK_OUT" | "BREAK_START" => {
                if let Some(start_ts) = current_clock_in {
                    total_worked_ms += (ts - start_ts).max(0);
                    current_clock_in = None;
                }
            }
            _ => {}
        }
    }

    let total_hours = (total_worked_ms as f64) / (1000.0 * 3600.0);
    let regular_hours = total_hours.min(8.0);
    let overtime_hours = (total_hours - 8.0).max(0.0);

    Ok((regular_hours, overtime_hours))
}

pub fn list_employees(conn: &Connection) -> Result<Vec<EmployeeProfile>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT employee_id, afm, amka, full_name, job_title, pin_code, qr_badge_token, is_active
        FROM work_employees
        ORDER BY full_name ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(EmployeeProfile {
            employee_id: row.get(0)?,
            afm: row.get(1)?,
            amka: row.get(2)?,
            full_name: row.get(3)?,
            job_title: row.get(4)?,
            pin_code: row.get(5)?,
            qr_badge_token: row.get(6)?,
            is_active: row.get::<_, i64>(7)? == 1,
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn list_today_events(
    conn: &Connection,
    date_str: &str,
) -> Result<Vec<(WorkCardEvent, String)>> {
    let filter = format!("{}%", date_str);
    let mut stmt = conn.prepare(
        r#"
        SELECT e.event_id, e.employee_id, e.event_type, e.timestamp_utc, e.local_time_str,
               e.is_offline_fallback, e.outage_reason, e.merkle_hash, e.ergani_submission_id,
               e.sync_status, emp.full_name
        FROM work_card_events e
        JOIN work_employees emp ON e.employee_id = emp.employee_id
        WHERE e.local_time_str LIKE ?1
        ORDER BY e.timestamp_utc DESC
        "#,
    )?;

    let rows = stmt.query_map(params![filter], |row| {
        let ev_type_str: String = row.get(2)?;
        let ev_type = WorkCardEventType::from_str(&ev_type_str).unwrap_or(WorkCardEventType::ClockIn);
        let event = WorkCardEvent {
            event_id: row.get(0)?,
            employee_id: row.get(1)?,
            event_type: ev_type,
            timestamp_utc: row.get(3)?,
            local_time_str: row.get(4)?,
            is_offline_fallback: row.get::<_, i64>(5)? == 1,
            outage_reason: row.get(6)?,
            merkle_hash: row.get(7)?,
            ergani_submission_id: row.get(8)?,
            sync_status: row.get(9)?,
        };
        let emp_name: String = row.get(10)?;
        Ok((event, emp_name))
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

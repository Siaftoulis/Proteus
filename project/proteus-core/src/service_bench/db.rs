//! Persistent SQLite storage for diagnostics, parts consumption, and labor logs.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).

use rusqlite::{params, Connection, Result as SqlResult};

use crate::service_bench::types::{
    BenchSummary, ConsumedSparePart, DeviceDiagnosticChecklist, LaborLog,
};

/// Initializes the service bench database schema.
pub fn init_service_bench_schema(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS ticket_diagnostics (
            ticket_id TEXT PRIMARY KEY,
            powers_on INTEGER NOT NULL,
            display_functional INTEGER NOT NULL,
            touch_responsive INTEGER NOT NULL,
            liquid_ingress_detected INTEGER NOT NULL,
            audio_functional INTEGER NOT NULL,
            camera_functional INTEGER NOT NULL,
            battery_health_pct INTEGER,
            cosmetic_condition TEXT NOT NULL,
            customer_accessories TEXT NOT NULL,
            inspected_by TEXT NOT NULL,
            inspected_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ticket_parts_consumed (
            id TEXT PRIMARY KEY,
            ticket_id TEXT NOT NULL,
            part_sku TEXT NOT NULL,
            description TEXT NOT NULL,
            quantity REAL NOT NULL,
            unit_cost_eur REAL NOT NULL,
            unit_retail_eur REAL NOT NULL,
            technician_name TEXT NOT NULL,
            consumed_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_parts_ticket ON ticket_parts_consumed(ticket_id);

        CREATE TABLE IF NOT EXISTS ticket_labor_logs (
            id TEXT PRIMARY KEY,
            ticket_id TEXT NOT NULL,
            technician_name TEXT NOT NULL,
            duration_minutes INTEGER NOT NULL,
            hourly_rate_eur REAL NOT NULL,
            work_performed TEXT NOT NULL,
            logged_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_labor_ticket ON ticket_labor_logs(ticket_id);
        "#,
    )?;
    Ok(())
}

/// Saves or updates the diagnostic checklist for a ticket.
pub fn save_diagnostic_checklist(
    conn: &Connection,
    diag: &DeviceDiagnosticChecklist,
) -> SqlResult<()> {
    conn.execute(
        r#"
        INSERT INTO ticket_diagnostics (
            ticket_id, powers_on, display_functional, touch_responsive,
            liquid_ingress_detected, audio_functional, camera_functional,
            battery_health_pct, cosmetic_condition, customer_accessories,
            inspected_by, inspected_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(ticket_id) DO UPDATE SET
            powers_on = excluded.powers_on,
            display_functional = excluded.display_functional,
            touch_responsive = excluded.touch_responsive,
            liquid_ingress_detected = excluded.liquid_ingress_detected,
            audio_functional = excluded.audio_functional,
            camera_functional = excluded.camera_functional,
            battery_health_pct = excluded.battery_health_pct,
            cosmetic_condition = excluded.cosmetic_condition,
            customer_accessories = excluded.customer_accessories,
            inspected_by = excluded.inspected_by,
            inspected_at = excluded.inspected_at
        "#,
        params![
            diag.ticket_id,
            if diag.powers_on { 1 } else { 0 },
            if diag.display_functional { 1 } else { 0 },
            if diag.touch_responsive { 1 } else { 0 },
            if diag.liquid_ingress_detected { 1 } else { 0 },
            if diag.audio_functional { 1 } else { 0 },
            if diag.camera_functional { 1 } else { 0 },
            diag.battery_health_pct,
            diag.cosmetic_condition,
            diag.customer_accessories,
            diag.inspected_by,
            diag.inspected_at,
        ],
    )?;
    Ok(())
}

/// Retrieves the diagnostic checklist for a ticket.
pub fn get_diagnostic_checklist(
    conn: &Connection,
    ticket_id: &str,
) -> SqlResult<Option<DeviceDiagnosticChecklist>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ticket_id, powers_on, display_functional, touch_responsive,
               liquid_ingress_detected, audio_functional, camera_functional,
               battery_health_pct, cosmetic_condition, customer_accessories,
               inspected_by, inspected_at
        FROM ticket_diagnostics
        WHERE ticket_id = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![ticket_id])?;
    if let Some(r) = rows.next()? {
        let p_on: i32 = r.get(1)?;
        let d_fun: i32 = r.get(2)?;
        let t_res: i32 = r.get(3)?;
        let l_ing: i32 = r.get(4)?;
        let a_fun: i32 = r.get(5)?;
        let c_fun: i32 = r.get(6)?;

        Ok(Some(DeviceDiagnosticChecklist {
            ticket_id: r.get(0)?,
            powers_on: p_on == 1,
            display_functional: d_fun == 1,
            touch_responsive: t_res == 1,
            liquid_ingress_detected: l_ing == 1,
            audio_functional: a_fun == 1,
            camera_functional: c_fun == 1,
            battery_health_pct: r.get(7)?,
            cosmetic_condition: r.get(8)?,
            customer_accessories: r.get(9)?,
            inspected_by: r.get(10)?,
            inspected_at: r.get(11)?,
        }))
    } else {
        Ok(None)
    }
}

/// Records a consumed spare part on the bench.
pub fn add_consumed_part(conn: &Connection, part: &ConsumedSparePart) -> SqlResult<()> {
    conn.execute(
        r#"
        INSERT INTO ticket_parts_consumed (
            id, ticket_id, part_sku, description, quantity,
            unit_cost_eur, unit_retail_eur, technician_name, consumed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            part.id,
            part.ticket_id,
            part.part_sku,
            part.description,
            part.quantity,
            part.unit_cost_eur,
            part.unit_retail_eur,
            part.technician_name,
            part.consumed_at,
        ],
    )?;
    Ok(())
}

/// Removes a consumed spare part record.
pub fn remove_consumed_part(conn: &Connection, part_id: &str) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM ticket_parts_consumed WHERE id = ?1",
        params![part_id],
    )?;
    Ok(())
}

/// Lists all spare parts consumed for a specific ticket.
pub fn list_consumed_parts(
    conn: &Connection,
    ticket_id: &str,
) -> SqlResult<Vec<ConsumedSparePart>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, ticket_id, part_sku, description, quantity,
               unit_cost_eur, unit_retail_eur, technician_name, consumed_at
        FROM ticket_parts_consumed
        WHERE ticket_id = ?1
        ORDER BY consumed_at ASC
        "#,
    )?;

    let rows = stmt.query_map(params![ticket_id], |r| {
        Ok(ConsumedSparePart {
            id: r.get(0)?,
            ticket_id: r.get(1)?,
            part_sku: r.get(2)?,
            description: r.get(3)?,
            quantity: r.get(4)?,
            unit_cost_eur: r.get(5)?,
            unit_retail_eur: r.get(6)?,
            technician_name: r.get(7)?,
            consumed_at: r.get(8)?,
        })
    })?;

    let mut parts = Vec::new();
    for row in rows {
        parts.push(row?);
    }
    Ok(parts)
}

/// Logs technician labor spent on a ticket.
pub fn log_technician_labor(conn: &Connection, log: &LaborLog) -> SqlResult<()> {
    conn.execute(
        r#"
        INSERT INTO ticket_labor_logs (
            id, ticket_id, technician_name, duration_minutes,
            hourly_rate_eur, work_performed, logged_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            log.id,
            log.ticket_id,
            log.technician_name,
            log.duration_minutes,
            log.hourly_rate_eur,
            log.work_performed,
            log.logged_at,
        ],
    )?;
    Ok(())
}

/// Lists all labor logs for a ticket.
pub fn list_labor_logs(conn: &Connection, ticket_id: &str) -> SqlResult<Vec<LaborLog>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, ticket_id, technician_name, duration_minutes,
               hourly_rate_eur, work_performed, logged_at
        FROM ticket_labor_logs
        WHERE ticket_id = ?1
        ORDER BY logged_at ASC
        "#,
    )?;

    let rows = stmt.query_map(params![ticket_id], |r| {
        Ok(LaborLog {
            id: r.get(0)?,
            ticket_id: r.get(1)?,
            technician_name: r.get(2)?,
            duration_minutes: r.get(3)?,
            hourly_rate_eur: r.get(4)?,
            work_performed: r.get(5)?,
            logged_at: r.get(6)?,
        })
    })?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row?);
    }
    Ok(logs)
}

/// Computes the aggregated financial and labor summary for a service ticket.
pub fn compute_bench_summary(conn: &Connection, ticket_id: &str) -> SqlResult<BenchSummary> {
    let parts = list_consumed_parts(conn, ticket_id)?;
    let labor = list_labor_logs(conn, ticket_id)?;

    let parts_total_cost_eur: f64 = parts.iter().map(|p| p.line_cost_total()).sum();
    let parts_total_retail_eur: f64 = parts.iter().map(|p| p.line_retail_total()).sum();
    let labor_total_minutes: u32 = labor.iter().map(|l| l.duration_minutes).sum();
    let labor_total_eur: f64 = labor.iter().map(|l| l.labor_cost_eur()).sum();

    let gross_total_eur = parts_total_retail_eur + labor_total_eur;

    Ok(BenchSummary {
        parts_total_cost_eur,
        parts_total_retail_eur,
        labor_total_minutes,
        labor_total_eur,
        gross_total_eur,
        parts_count: parts.len(),
        labor_logs_count: labor.len(),
    })
}

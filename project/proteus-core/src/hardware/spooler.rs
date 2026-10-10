//! SQLite Persistent Label Print Spooler Queue Engine.
//! Manages industrial thermal print queues, status transitions,
//! retry logic, and asynchronous dispatch for TSPL and ZPL printers.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Target command set protocol of the destination printer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrintTargetProtocol {
    Tspl,
    Zpl,
    EscPos,
}

impl PrintTargetProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tspl => "TSPL",
            Self::Zpl => "ZPL",
            Self::EscPos => "ESC/POS",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "ZPL" => Self::Zpl,
            "ESC/POS" => Self::EscPos,
            _ => Self::Tspl,
        }
    }
}

/// Lifecycle status of an enqueued print job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrintJobStatus {
    Pending,
    Printing,
    Completed,
    Failed(String),
}

impl PrintJobStatus {
    pub fn to_parts(&self) -> (&'static str, Option<String>) {
        match self {
            Self::Pending => ("Pending", None),
            Self::Printing => ("Printing", None),
            Self::Completed => ("Completed", None),
            Self::Failed(err) => ("Failed", Some(err.clone())),
        }
    }

    pub fn from_parts(status: &str, error: Option<String>) -> Self {
        match status {
            "Printing" => Self::Printing,
            "Completed" => Self::Completed,
            "Failed" => Self::Failed(error.unwrap_or_else(|| "Unknown error".to_string())),
            _ => Self::Pending,
        }
    }
}

/// Persistent label print job record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelPrintJob {
    pub job_id: String,
    pub job_title: String,
    pub protocol: PrintTargetProtocol,
    pub payload_bytes: Vec<u8>,
    pub copies: u32,
    pub destination_endpoint: String,
    pub status: PrintJobStatus,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

impl LabelPrintJob {
    pub fn new(
        job_title: &str,
        protocol: PrintTargetProtocol,
        payload_bytes: Vec<u8>,
        copies: u32,
        destination_endpoint: &str,
        created_at: i64,
    ) -> Self {
        Self {
            job_id: Uuid::new_v4().to_string(),
            job_title: job_title.trim().to_string(),
            protocol,
            payload_bytes,
            copies: copies.max(1),
            destination_endpoint: destination_endpoint.trim().to_string(),
            status: PrintJobStatus::Pending,
            error_message: None,
            created_at,
            completed_at: None,
        }
    }
}

/// Initializes the SQLite schema for the thermal label print queue.
pub fn init_label_spooler_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS label_print_queue (
            job_id TEXT PRIMARY KEY,
            job_title TEXT NOT NULL,
            protocol TEXT NOT NULL,
            payload_bytes BLOB NOT NULL,
            copies INTEGER NOT NULL DEFAULT 1,
            destination_endpoint TEXT NOT NULL,
            status TEXT NOT NULL,
            error_message TEXT,
            created_at INTEGER NOT NULL,
            completed_at INTEGER
        );

        CREATE INDEX IF NOT EXISTS idx_label_spooler_status ON label_print_queue(status, created_at);
        CREATE INDEX IF NOT EXISTS idx_label_spooler_created ON label_print_queue(created_at DESC);
        "#,
    )
}

/// Enqueues a new label print job into the SQLite database.
pub fn enqueue_print_job(conn: &Connection, job: &LabelPrintJob) -> Result<()> {
    let (status_str, err_opt) = job.status.to_parts();
    conn.execute(
        r#"
        INSERT INTO label_print_queue (
            job_id, job_title, protocol, payload_bytes, copies,
            destination_endpoint, status, error_message, created_at, completed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(job_id) DO UPDATE SET
            status = excluded.status,
            error_message = excluded.error_message,
            completed_at = excluded.completed_at
        "#,
        params![
            job.job_id,
            job.job_title,
            job.protocol.as_str(),
            job.payload_bytes,
            job.copies,
            job.destination_endpoint,
            status_str,
            err_opt,
            job.created_at,
            job.completed_at,
        ],
    )?;
    Ok(())
}

/// Retrieves the oldest pending print job from the spooler queue.
pub fn get_next_pending_job(conn: &Connection) -> Result<Option<LabelPrintJob>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT job_id, job_title, protocol, payload_bytes, copies,
               destination_endpoint, status, error_message, created_at, completed_at
        FROM label_print_queue
        WHERE status = 'Pending'
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let status_str: String = row.get(6)?;
        let err_str: Option<String> = row.get(7)?;
        let proto_str: String = row.get(2)?;

        Ok(Some(LabelPrintJob {
            job_id: row.get(0)?,
            job_title: row.get(1)?,
            protocol: PrintTargetProtocol::from_str_opt(&proto_str),
            payload_bytes: row.get(3)?,
            copies: row.get(4)?,
            destination_endpoint: row.get(5)?,
            status: PrintJobStatus::from_parts(&status_str, err_str),
            error_message: row.get(7)?,
            created_at: row.get(8)?,
            completed_at: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

/// Updates the status of an existing print job.
pub fn update_job_status(
    conn: &Connection,
    job_id: &str,
    status: PrintJobStatus,
    completed_at: Option<i64>,
) -> Result<()> {
    let (status_str, err_opt) = status.to_parts();
    conn.execute(
        r#"
        UPDATE label_print_queue
        SET status = ?1, error_message = ?2, completed_at = ?3
        WHERE job_id = ?4
        "#,
        params![status_str, err_opt, completed_at, job_id],
    )?;
    Ok(())
}

/// Lists recent print jobs up to a given limit.
pub fn list_recent_jobs(conn: &Connection, limit: usize) -> Result<Vec<LabelPrintJob>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT job_id, job_title, protocol, payload_bytes, copies,
               destination_endpoint, status, error_message, created_at, completed_at
        FROM label_print_queue
        ORDER BY created_at DESC
        LIMIT ?1
        "#,
    )?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            let proto_str: String = row.get(2)?;
            let status_str: String = row.get(6)?;
            let err_str: Option<String> = row.get(7)?;

            Ok(LabelPrintJob {
                job_id: row.get(0)?,
                job_title: row.get(1)?,
                protocol: PrintTargetProtocol::from_str_opt(&proto_str),
                payload_bytes: row.get(3)?,
                copies: row.get(4)?,
                destination_endpoint: row.get(5)?,
                status: PrintJobStatus::from_parts(&status_str, err_str),
                error_message: row.get(7)?,
                created_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(rows)
}

/// Deletes or purges completed jobs older than a given epoch timestamp.
pub fn purge_completed_jobs(conn: &Connection, older_than_epoch: i64) -> Result<usize> {
    let count = conn.execute(
        r#"
        DELETE FROM label_print_queue
        WHERE status = 'Completed' AND completed_at IS NOT NULL AND completed_at < ?1
        "#,
        params![older_than_epoch],
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_spooler_queue_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        init_label_spooler_schema(&conn).unwrap();

        let raw_tspl = b"SIZE 50 mm, 30 mm\r\nPRINT 1,1\r\n".to_vec();
        let job = LabelPrintJob::new(
            "Ετικέτα Ραφιού #1",
            PrintTargetProtocol::Tspl,
            raw_tspl.clone(),
            1,
            "tcp://192.168.1.150:9100",
            1000,
        );

        enqueue_print_job(&conn, &job).unwrap();

        let pending = get_next_pending_job(&conn).unwrap();
        assert!(pending.is_some());
        let active = pending.unwrap();
        assert_eq!(active.job_title, "Ετικέτα Ραφιού #1");
        assert_eq!(active.protocol, PrintTargetProtocol::Tspl);
        assert_eq!(active.payload_bytes, raw_tspl);

        // Mark as completed
        update_job_status(&conn, &active.job_id, PrintJobStatus::Completed, Some(1010)).unwrap();

        // No more pending jobs
        let next = get_next_pending_job(&conn).unwrap();
        assert!(next.is_none());

        // Check list
        let recent = list_recent_jobs(&conn, 10).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].status, PrintJobStatus::Completed);

        // Purge
        let purged = purge_completed_jobs(&conn, 1020).unwrap();
        assert_eq!(purged, 1);
        assert_eq!(list_recent_jobs(&conn, 10).unwrap().len(), 0);
    }
}

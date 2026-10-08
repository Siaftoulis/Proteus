//! Persistent SQLite notification outbox queue & delivery retry management.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).

use chrono::Utc;
use rusqlite::{params, Connection, Result as SqlResult};

use crate::notifications::types::{
    validate_recipient, NotificationCategory, NotificationChannel, NotificationError,
    NotificationStatus, OutboxNotification,
};

/// Initializes the notification outbox database schema.
pub fn init_notifications_schema(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS notification_outbox (
            id TEXT PRIMARY KEY,
            channel TEXT NOT NULL,
            category TEXT NOT NULL,
            recipient TEXT NOT NULL,
            subject TEXT,
            content TEXT NOT NULL,
            status TEXT NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0,
            max_retries INTEGER NOT NULL DEFAULT 3,
            last_error TEXT,
            scheduled_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            dispatched_at INTEGER,
            gdpr_consent_verified INTEGER NOT NULL DEFAULT 0,
            metadata_json TEXT NOT NULL DEFAULT '{}'
        );
        CREATE INDEX IF NOT EXISTS idx_notif_status_sched ON notification_outbox(status, scheduled_at);
        CREATE INDEX IF NOT EXISTS idx_notif_recipient ON notification_outbox(recipient);
        "#,
    )?;
    Ok(())
}

/// Enqueues a notification into the sovereign outbox queue.
/// Enforces GDPR Art. 6 and Greek Law 4624/2019 consent for marketing messages.
pub fn enqueue_notification(
    conn: &Connection,
    notif: &OutboxNotification,
) -> Result<String, NotificationError> {
    validate_recipient(notif.channel, &notif.recipient)?;

    if notif.category == NotificationCategory::Marketing && !notif.gdpr_consent_verified {
        return Err(NotificationError::ConsentRequired);
    }

    conn.execute(
        r#"
        INSERT INTO notification_outbox (
            id, channel, category, recipient, subject, content, status,
            attempts, max_retries, last_error, scheduled_at, created_at,
            dispatched_at, gdpr_consent_verified, metadata_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        "#,
        params![
            notif.id,
            notif.channel.as_str(),
            notif.category.as_str(),
            notif.recipient,
            notif.subject,
            notif.content,
            notif.status.as_str(),
            notif.attempts,
            notif.max_retries,
            notif.last_error,
            notif.scheduled_at,
            notif.created_at,
            notif.dispatched_at,
            if notif.gdpr_consent_verified { 1 } else { 0 },
            notif.metadata_json,
        ],
    )?;

    Ok(notif.id.clone())
}

/// Fetches pending notifications eligible for dispatch.
pub fn fetch_pending_notifications(
    conn: &Connection,
    limit: usize,
) -> SqlResult<Vec<OutboxNotification>> {
    let now = Utc::now().timestamp_millis();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, channel, category, recipient, subject, content, status,
               attempts, max_retries, last_error, scheduled_at, created_at,
               dispatched_at, gdpr_consent_verified, metadata_json
        FROM notification_outbox
        WHERE status = 'PENDING' AND scheduled_at <= ?1
        ORDER BY scheduled_at ASC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![now, limit as i64], |r| {
        let ch_str: String = r.get(1)?;
        let cat_str: String = r.get(2)?;
        let st_str: String = r.get(6)?;
        let consent_int: i32 = r.get(13)?;

        Ok(OutboxNotification {
            id: r.get(0)?,
            channel: NotificationChannel::from_str(&ch_str),
            category: NotificationCategory::from_str(&cat_str),
            recipient: r.get(3)?,
            subject: r.get(4)?,
            content: r.get(5)?,
            status: NotificationStatus::from_str(&st_str),
            attempts: r.get(7)?,
            max_retries: r.get(8)?,
            last_error: r.get(9)?,
            scheduled_at: r.get(10)?,
            created_at: r.get(11)?,
            dispatched_at: r.get(12)?,
            gdpr_consent_verified: consent_int == 1,
            metadata_json: r.get(14)?,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

/// Marks a notification as successfully dispatched.
pub fn mark_notification_dispatched(conn: &Connection, id: &str) -> SqlResult<()> {
    let now = Utc::now().timestamp_millis();
    conn.execute(
        "UPDATE notification_outbox SET status = 'DISPATCHED', dispatched_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

/// Records a failed dispatch attempt with exponential backoff scheduling.
pub fn record_dispatch_failure(
    conn: &Connection,
    id: &str,
    error_msg: &str,
) -> SqlResult<()> {
    let (attempts, max_retries): (u32, u32) = conn.query_row(
        "SELECT attempts, max_retries FROM notification_outbox WHERE id = ?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let new_attempts = attempts + 1;
    let (new_status, next_sched) = if new_attempts >= max_retries {
        ("FAILED", Utc::now().timestamp_millis())
    } else {
        let backoff_secs = 2_i64.pow(new_attempts) * 30; // 60s, 120s, 240s...
        ("PENDING", Utc::now().timestamp_millis() + backoff_secs * 1000)
    };

    conn.execute(
        r#"
        UPDATE notification_outbox
        SET attempts = ?1, status = ?2, last_error = ?3, scheduled_at = ?4
        WHERE id = ?5
        "#,
        params![new_attempts, new_status, error_msg, next_sched, id],
    )?;
    Ok(())
}

/// Lists all notifications from the outbox for monitoring.
pub fn list_notifications(conn: &Connection, limit: usize) -> SqlResult<Vec<OutboxNotification>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, channel, category, recipient, subject, content, status,
               attempts, max_retries, last_error, scheduled_at, created_at,
               dispatched_at, gdpr_consent_verified, metadata_json
        FROM notification_outbox
        ORDER BY created_at DESC
        LIMIT ?1
        "#,
    )?;

    let rows = stmt.query_map(params![limit as i64], |r| {
        let ch_str: String = r.get(1)?;
        let cat_str: String = r.get(2)?;
        let st_str: String = r.get(6)?;
        let consent_int: i32 = r.get(13)?;

        Ok(OutboxNotification {
            id: r.get(0)?,
            channel: NotificationChannel::from_str(&ch_str),
            category: NotificationCategory::from_str(&cat_str),
            recipient: r.get(3)?,
            subject: r.get(4)?,
            content: r.get(5)?,
            status: NotificationStatus::from_str(&st_str),
            attempts: r.get(7)?,
            max_retries: r.get(8)?,
            last_error: r.get(9)?,
            scheduled_at: r.get(10)?,
            created_at: r.get(11)?,
            dispatched_at: r.get(12)?,
            gdpr_consent_verified: consent_int == 1,
            metadata_json: r.get(14)?,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

/// GDPR Article 5(1)(e) & Greek Law 4624/2019: Storage limitation purge.
/// Deletes completed/dispatched notifications older than statutory retention days.
pub fn purge_dispatched_notifications(
    conn: &Connection,
    retention_days: u32,
    operator: &str,
) -> SqlResult<usize> {
    let cutoff_millis = Utc::now().timestamp_millis() - (retention_days as i64 * 86_400_000);
    let deleted = conn.execute(
        r#"
        DELETE FROM notification_outbox
        WHERE status IN ('DISPATCHED', 'FAILED') AND created_at < ?1
        "#,
        params![cutoff_millis],
    )?;

    if deleted > 0 {
        let event_id = uuid::Uuid::now_v7().to_string();
        let now = Utc::now().timestamp_millis();
        let _ = conn.execute(
            "INSERT INTO system_events (event_id, entity_id, event_type, payload, created_at)
             VALUES (?1, 'NOTIFICATIONS_GATEWAY', 'GDPR_NOTIFICATIONS_PURGED', ?2, ?3)",
            params![
                event_id,
                format!(r#"{{"operator":"{}","deleted_count":{},"retention_days":{}}}"#, operator, deleted, retention_days),
                now
            ],
        );
    }
    Ok(deleted)
}

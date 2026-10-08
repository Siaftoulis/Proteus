//! Sovereign Automated Notifications Gateway & Customer Live Repair Tracker.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Complies with GDPR Art. 6, Greek Law 4624/2019, and ISO 27001 auditability.

pub mod db;
pub mod token;
pub mod types;

pub use db::{
    fetch_pending_notifications, init_notifications_schema, list_notifications,
    mark_notification_dispatched, record_dispatch_failure, enqueue_notification,
    purge_dispatched_notifications,
};
pub use token::{
    format_payment_receipt_message, format_repair_tracking_message, generate_tracking_token,
    verify_tracking_token,
};
pub use types::{
    validate_recipient, NotificationCategory, NotificationChannel, NotificationError,
    NotificationStatus, OutboxNotification,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rusqlite::Connection;
    use uuid::Uuid;

    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_notifications_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_live_tracking_token_generation_and_verification() {
        let secret = b"proteus-sovereign-live-tracker-key-2026";
        let ticket_id = "TCK-2026-9901";

        let token = generate_tracking_token(ticket_id, secret);
        assert!(verify_tracking_token(ticket_id, &token, secret));

        // Wrong ticket ID fails
        assert!(!verify_tracking_token("TCK-2026-9902", &token, secret));

        // Tampered token fails
        assert!(!verify_tracking_token(ticket_id, "corrupted.token", secret));
    }

    #[test]
    fn test_gdpr_marketing_consent_enforcement() {
        let conn = in_memory_db();

        let notif_without_consent = OutboxNotification {
            id: Uuid::new_v4().to_string(),
            channel: NotificationChannel::Sms,
            category: NotificationCategory::Marketing,
            recipient: "+306912345678".to_string(),
            subject: None,
            content: "Ειδική έκπτωση 20% σε όλα τα αξεσουάρ!".to_string(),
            status: NotificationStatus::Pending,
            attempts: 0,
            max_retries: 3,
            last_error: None,
            scheduled_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
            dispatched_at: None,
            gdpr_consent_verified: false,
            metadata_json: "{}".to_string(),
        };

        // Should be rejected under GDPR Art. 6 / Law 4624/2019
        let res = enqueue_notification(&conn, &notif_without_consent);
        assert!(matches!(res, Err(NotificationError::ConsentRequired)));

        // Transactional message without consent is permitted under contract necessity (Art. 6(1)(b))
        let notif_transactional = OutboxNotification {
            category: NotificationCategory::Transactional,
            ..notif_without_consent
        };
        let res = enqueue_notification(&conn, &notif_transactional);
        assert!(res.is_ok());
    }

    #[test]
    fn test_dispatch_retry_backoff_and_exhaustion() {
        let conn = in_memory_db();
        let notif_id = Uuid::new_v4().to_string();

        let notif = OutboxNotification {
            id: notif_id.clone(),
            channel: NotificationChannel::Viber,
            category: NotificationCategory::Transactional,
            recipient: "+306912345678".to_string(),
            subject: None,
            content: "Η συσκευή σας είναι έτοιμη.".to_string(),
            status: NotificationStatus::Pending,
            attempts: 0,
            max_retries: 2,
            last_error: None,
            scheduled_at: Utc::now().timestamp_millis() - 1000,
            created_at: Utc::now().timestamp_millis() - 1000,
            dispatched_at: None,
            gdpr_consent_verified: true,
            metadata_json: "{}".to_string(),
        };
        enqueue_notification(&conn, &notif).unwrap();

        // 1st failure -> attempts = 1, status = PENDING (backed off)
        record_dispatch_failure(&conn, &notif_id, "Viber Gateway Timeout").unwrap();
        let list = list_notifications(&conn, 10).unwrap();
        assert_eq!(list[0].attempts, 1);
        assert_eq!(list[0].status, NotificationStatus::Pending);

        // 2nd failure -> reaches max_retries (2) -> FAILED
        record_dispatch_failure(&conn, &notif_id, "Destination unreachable").unwrap();
        let list = list_notifications(&conn, 10).unwrap();
        assert_eq!(list[0].attempts, 2);
        assert_eq!(list[0].status, NotificationStatus::Failed);

        // GDPR Art. 5(1)(e): Purge failed/dispatched older than 0 days (immediate retention test)
        let purged = purge_dispatched_notifications(&conn, 0, "DPO_Admin").unwrap();
        assert_eq!(purged, 1);
        let list_after = list_notifications(&conn, 10).unwrap();
        assert!(list_after.is_empty());
    }
}

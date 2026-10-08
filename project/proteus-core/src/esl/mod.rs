//! Electronic Shelf Label (ESL) Gateway & Dynamic Expiry Pricing Engine for Proteus BOS.
//! Manages e-ink retail shelf labels (2.9" and 4.2" displays) via 433/868MHz and Bluetooth.
//! Automatically discounts products nearing expiry (GS1 AI 17) and broadcasts price tag updates.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

pub mod db;
pub mod gateway;
pub mod packet;
pub mod types;

pub use db::{
    enqueue_esl_update, init_esl_schema, list_esl_tags, list_pending_broadcasts,
    load_esl_gateway_config, mark_packet_transmitted, register_esl_tag,
    save_esl_gateway_config, sync_esl_tags_for_sku,
};
pub use gateway::{
    broadcast_all_pending, encode_gateway_datagram, ping_esl_gateway,
    transmit_frame_to_gateway,
};
pub use packet::{evaluate_dynamic_expiry_discount, generate_esl_radio_packet, hex_decode, hex_encode};
pub use types::{
    EslBatchBroadcastResult, EslBroadcastPacket, EslDisplaySize, EslGatewayConfig, EslTag,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_dynamic_expiry_discount_tiers() {
        let today = "2026-10-03";

        // Expiring today / tomorrow (<= 1 day): 50% discount
        let (_, disc_50, promo_50) = evaluate_dynamic_expiry_discount(10.0, "2026-10-04", today);
        assert_eq!(disc_50, Some(5.00));
        assert!(promo_50.unwrap().contains("-50%"));

        // Expiring in 3 days: 30% discount
        let (_, disc_30, promo_30) = evaluate_dynamic_expiry_discount(20.0, "2026-10-06", today);
        assert_eq!(disc_30, Some(14.00));
        assert!(promo_30.unwrap().contains("-30%"));

        // Expiring in 6 days: 15% discount
        let (_, disc_15, promo_15) = evaluate_dynamic_expiry_discount(100.0, "2026-10-09", today);
        assert_eq!(disc_15, Some(85.00));
        assert!(promo_15.unwrap().contains("-15%"));

        // Expiring in 30 days: No discount
        let (_, no_disc, no_promo) = evaluate_dynamic_expiry_discount(50.0, "2026-11-03", today);
        assert_eq!(no_disc, None);
        assert_eq!(no_promo, None);
    }

    #[test]
    fn test_esl_radio_packet_and_sqlite_queue() {
        let conn = Connection::open_in_memory().unwrap();
        init_esl_schema(&conn).unwrap();

        let tag = EslTag {
            tag_mac: "AA:BB:CC:11:22:33".to_string(),
            shelf_id: "S-102".to_string(),
            sku: "MILK-FRESH-1L".to_string(),
            product_name: "Φρέσκο Γάλα 1L".to_string(),
            current_price_eur: 1.80,
            discount_price_eur: Some(0.90),
            unit_of_measure: "lit".to_string(),
            battery_percentage: 92,
            signal_rssi: -58,
            last_sync_utc: 1727956800000,
            pending_refresh: false,
        };
        register_esl_tag(&conn, &tag).unwrap();

        let packet = generate_esl_radio_packet(
            &tag.tag_mac,
            tag.current_price_eur,
            tag.discount_price_eur,
            Some("-50%"),
            "5201234567890",
        );

        assert_eq!(&packet[0..4], b"PRTS");
        assert!(packet.len() >= 20);

        let broadcast_id = enqueue_esl_update(&conn, &tag.tag_mac, &packet).unwrap();

        let pending_count: i64 = conn.query_row(
            "SELECT count(*) FROM esl_broadcast_queue WHERE status = 'PENDING'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(pending_count, 1);

        mark_packet_transmitted(&conn, &broadcast_id).unwrap();

        let transmitted_count: i64 = conn.query_row(
            "SELECT count(*) FROM esl_broadcast_queue WHERE status = 'TRANSMITTED'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(transmitted_count, 1);
    }

    #[test]
    fn test_batch_broadcast_and_sku_sync() {
        let conn = Connection::open_in_memory().unwrap();
        init_esl_schema(&conn).unwrap();

        let tag = EslTag {
            tag_mac: "11:22:33:44:55:66".to_string(),
            shelf_id: "SHELF-A1".to_string(),
            sku: "YOGURT-GREEK".to_string(),
            product_name: "Γιαούρτι Στραγγιστό".to_string(),
            current_price_eur: 3.20,
            discount_price_eur: None,
            unit_of_measure: "τεμ".to_string(),
            battery_percentage: 100,
            signal_rssi: -60,
            last_sync_utc: 0,
            pending_refresh: false,
        };
        register_esl_tag(&conn, &tag).unwrap();

        // Sync new price
        let enqueued = sync_esl_tags_for_sku(&conn, "YOGURT-GREEK", 2.99).unwrap();
        assert_eq!(enqueued, 1);

        let cfg = EslGatewayConfig::default();
        let res = broadcast_all_pending(&conn, &cfg).unwrap();
        assert_eq!(res.total_enqueued, 1);
        assert_eq!(res.transmitted_success, 1);
        assert_eq!(res.failed_count, 0);
    }
}

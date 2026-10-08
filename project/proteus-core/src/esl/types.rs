//! ESL (Electronic Shelf Labels) Data Types & Gateway Configuration.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EslDisplaySize {
    Epd2_9Inch, // 296x128 standard shelf edge
    Epd4_2Inch, // 400x300 promo endcap / pallet tag
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EslTag {
    pub tag_mac: String,
    pub shelf_id: String,
    pub sku: String,
    pub product_name: String,
    pub current_price_eur: f64,
    pub discount_price_eur: Option<f64>,
    pub unit_of_measure: String,
    pub battery_percentage: u8,
    pub signal_rssi: i8,
    pub last_sync_utc: i64,
    pub pending_refresh: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EslBroadcastPacket {
    pub broadcast_id: String,
    pub tag_mac: String,
    pub payload_hex: String,
    pub status: String, // PENDING, TRANSMITTED, FAILED
    pub created_at: i64,
    pub transmitted_at: Option<i64>,
}

/// RF Gateway Base Station Settings (Sub-1GHz 433/868MHz or 2.4GHz BLE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EslGatewayConfig {
    pub gateway_host: String,
    pub gateway_port: u16,
    pub rf_channel: u8,
    pub tx_power_dbm: i8,
    pub auto_sync_on_price_change: bool,
}

impl Default for EslGatewayConfig {
    fn default() -> Self {
        Self {
            gateway_host: "127.0.0.1".to_string(),
            gateway_port: 9100,
            rf_channel: 7, // 868.3 MHz channel
            tx_power_dbm: 14,
            auto_sync_on_price_change: true,
        }
    }
}

/// Financial & operational summary of a batch RF broadcast update.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EslBatchBroadcastResult {
    pub total_enqueued: usize,
    pub transmitted_success: usize,
    pub failed_count: usize,
    pub duration_ms: u64,
}

//! Federated B2B Dealer Networks & Marketplace Settlement Platform for Proteus.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Connects autonomous merchants, synchronizes cross-store dealer inventories,
//! enforces time-locked anti-race stock reservations, and calculates EU P2B / DAC7 settlements.

pub mod reservation;
pub mod settlement;

pub use reservation::{
    confirm_reservation, init_b2b_reservation_tables, reap_expired_reservations,
    register_dealer, release_reservation, reserve_stock, upsert_shared_inventory_item,
    B2bDealer, ReservationStatus, SharedItem, StockReservation,
};
pub use settlement::{
    audit_dac7_thresholds, calculate_commission_split, generate_dac7_json_report,
    init_b2b_settlement_tables, insert_commission_rule, record_order_settlement,
    CommissionRule, Dac7VendorSummary, SplitPayout,
};

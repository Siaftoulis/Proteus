//! Customer & Supplier Financial Cardex & Balance Aging Engine for Proteus BOS.
//! Implements 100% original, native Greek commercial cardex ledger routines:
//! - Rolling balances (Χρέωση / Πίστωση / Υπόλοιπο - Debit / Credit / Balance)
//! - Real-time debt and credit tracking for B2B/B2C Customers and Suppliers
//! - 5-Tier Statutory Balance Aging Analysis (0-30d, 31-60d, 61-90d, 91-120d, 120+ days)
//! - Credit Limit Enforcement preventing unauthorized over-limit debt
//! - Official Payment Collection Receipts ("Απόδειξη Είσπραξης / Πληρωμής")
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

pub mod types;
pub mod aging;
pub mod db;

pub use types::*;
pub use aging::*;
pub use db::*;

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_cardex_workflow_and_aging() {
        let conn = Connection::open_in_memory().unwrap();
        init_cardex_schema(&conn).unwrap();

        let customer = CardexEntity {
            entity_id: "099887766".to_string(),
            entity_type: EntityType::Customer,
            name: "Hellenic Tech A.E.".to_string(),
            phone: Some("2101234567".to_string()),
            email: Some("info@hellenictech.gr".to_string()),
            credit_limit_eur: 1500.0,
            payment_terms_days: 30,
            current_balance_eur: 0.0,
            created_at: 1710000000,
        };
        save_cardex_entity(&conn, &customer).unwrap();

        // 1. Post Invoice 1 (Debit +1000€)
        post_cardex_movement(
            &conn,
            "099887766",
            CardexMovementType::Debit,
            "ΤΙΜΟΛΟΓΙΟ",
            "A",
            101,
            "IT Equipment Sale",
            1000.0,
            "2026-08-01",
            "2026-08-31",
        ).unwrap();

        let ent = get_cardex_entity(&conn, "099887766").unwrap().unwrap();
        assert_eq!(ent.current_balance_eur, 1000.0);

        // 2. Check Credit Limit
        assert!(check_credit_limit_ok(&conn, "099887766", 400.0).unwrap());
        assert!(!check_credit_limit_ok(&conn, "099887766", 600.0).unwrap());

        // 3. Post Payment Receipt (Credit -400€)
        record_payment_receipt(
            &conn,
            "099887766",
            "E",
            1,
            400.0,
            "Τραπεζική Κατάθεση",
            "Alpha Bank IBAN-GR12",
            "2026-09-01",
        ).unwrap();

        let ent_after = get_cardex_entity(&conn, "099887766").unwrap().unwrap();
        assert_eq!(ent_after.current_balance_eur, 600.0);

        // 4. Verify aging analysis as of 2026-10-03 (63 days after Aug 1)
        let aging = calculate_balance_aging(&conn, "099887766", "2026-10-03").unwrap();
        assert_eq!(aging.total_balance_eur, 600.0);
        assert_eq!(aging.overdue_61_90_eur, 1000.0);

        let entries = list_cardex_entries(&conn, "099887766").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].movement_type, CardexMovementType::Debit);
        assert_eq!(entries[1].movement_type, CardexMovementType::Credit);

        // 5. GDPR Art. 17 & Greek Law 4624/2019: Customer Anonymization
        let ok = anonymize_cardex_customer(&conn, "099887766", "DPO_Admin").unwrap();
        assert!(ok);
        let anon = get_cardex_entity(&conn, "099887766").unwrap().unwrap();
        assert_eq!(anon.name, "[GDPR ΑΝΩΝΥΜΟΠΟΙΗΜΕΝΟ]");
        assert_eq!(anon.phone.as_deref(), Some("[ERASED]"));
        assert_eq!(anon.email.as_deref(), Some("[ERASED]"));
        assert_eq!(anon.current_balance_eur, 600.0); // Financial ledger balance preserved
    }
}

// Proteus Core — Fiscal Mechanisms (ΦΗΜ) & EFT-POS Protocol Bridge (AADE A.1155/2023)
// 100% Original Implementation. Zero third-party boilerplate.
// Complies with Greek Fiscal Legislation & Mandatory POS Interconnection Standards.

pub mod a1155;
pub mod db;
pub mod driver;
pub mod offline_saf;
pub mod risk_quota;
pub mod tcp_protocol;
pub mod types;

pub use offline_saf::{
    SafCaptureRequest, SafEngine, SafError, SafRiskConfig, SafStatus, SafSummary, SafVoucher,
};
pub use risk_quota::{
    IrisQrRequest, IrisQrResponse, RiskDecision, RiskMetrics, RiskPolicy, RiskQuotaError,
    RiskQuotaGuard,
};



pub use a1155::{enforce_a1155_interlock, format_a1155_receipt_slip, generate_a1155_signature, A1155Error};
pub use db::{
    get_last_daily_z, get_pos_card_transaction, init_fiscal_pos_schema, load_fiscal_pos_config,
    record_pos_card_transaction, save_daily_z_report, save_fiscal_pos_config,
};
pub use driver::{
    create_fiscal_driver, CashRegisterDriver, EftPosDriver, FiscalDeviceDriver,
    TypeBFiscalSignatureDriver,
};
pub use tcp_protocol::{
    close_terminal_batch, decode_a1155_frame, encode_a1155_frame, execute_pos_lan_handshake,
    ping_eft_pos_terminal, TerminalBatchSummary, ACK, CMD_CLOSE_BATCH, CMD_PING, CMD_REFUND,
    CMD_SALE, CMD_VOID, ETX, NAK, STX,
};
pub use types::{
    ConnectionInterface, DailyZReport, FiscalDeviceType, FiscalExecutionResult,
    FiscalHardwareConfig, FiscalReceiptItem, FiscalReceiptPayload, FiscalVatBreakdown,
    PosInitiateRequest, PosTerminalResponse, PosTransactionStatus, PosTransactionType,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn sample_receipt(method: &str, card: Option<PosTerminalResponse>) -> FiscalReceiptPayload {
        FiscalReceiptPayload {
            receipt_number: 1042,
            issue_date: "2026-10-04T12:00:00Z".to_string(),
            items: vec![FiscalReceiptItem {
                name: "Αλλαγή Οθόνης OLED".to_string(),
                quantity: 1.0,
                unit_price: 100.0,
                vat_rate: 24.0,
                department: 1,
            }],
            vat_breakdowns: vec![FiscalVatBreakdown {
                vat_rate: 24.0,
                net_amount: 80.65,
                vat_amount: 19.35,
            }],
            total_net: 80.65,
            total_vat: 19.35,
            total_gross: 100.0,
            payment_method: method.to_string(),
            card_payment: card,
        }
    }

    #[test]
    fn test_a1155_interlock_cash_bypasses_terminal() {
        let receipt = sample_receipt("CASH", None);
        let res = enforce_a1155_interlock(
            &receipt.payment_method,
            receipt.card_payment.as_ref(),
            "SECRET_KEY",
            10000,
            1042,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_a1155_interlock_card_without_terminal_fails() {
        let receipt = sample_receipt("CARD", None);
        let res = enforce_a1155_interlock(
            &receipt.payment_method,
            receipt.card_payment.as_ref(),
            "SECRET_KEY",
            10000,
            1042,
        );
        assert_eq!(res, Err(A1155Error::MissingCardAuthorization));
    }

    #[test]
    fn test_a1155_interlock_declined_card_fails() {
        let ts = 1728043200000;
        let card = PosTerminalResponse {
            transaction_id: "TX-99".to_string(),
            status: PosTransactionStatus::Declined,
            rrn: "410415123456".to_string(),
            auth_code: "".to_string(),
            terminal_id: "POS01".to_string(),
            merchant_id: "MERCH01".to_string(),
            card_brand: "VISA".to_string(),
            card_masked: "**** 1111".to_string(),
            stan: "000001".to_string(),
            interconnection_signature: "DUMMY".to_string(),
            timestamp: ts,
            raw_response_code: "51".to_string(),
            response_message: "INSUFFICIENT FUNDS".to_string(),
        };

        let receipt = sample_receipt("CARD", Some(card));
        let res = enforce_a1155_interlock(
            &receipt.payment_method,
            receipt.card_payment.as_ref(),
            "SECRET_KEY",
            10000,
            1042,
        );
        assert_eq!(res, Err(A1155Error::DeclinedCardTransaction(PosTransactionStatus::Declined)));
    }

    #[test]
    fn test_a1155_signature_verification_and_tamper_detection() {
        let secret = "GREEK_AADE_TEST_SECRET";
        let tid = "TID9910";
        let rrn = "410415888999";
        let amount = 10000;
        let rcp = 1042;
        let ts = 1728043200000;

        let sig = generate_a1155_signature(secret, tid, rrn, amount, rcp, ts);
        assert!(sig.starts_with("A1155:"));

        let card = PosTerminalResponse {
            transaction_id: "TX-100".to_string(),
            status: PosTransactionStatus::Approved,
            rrn: rrn.to_string(),
            auth_code: "099182".to_string(),
            terminal_id: tid.to_string(),
            merchant_id: "MERCH01".to_string(),
            card_brand: "MASTERCARD".to_string(),
            card_masked: "**** 4412".to_string(),
            stan: "001042".to_string(),
            interconnection_signature: sig,
            timestamp: ts,
            raw_response_code: "00".to_string(),
            response_message: "OK".to_string(),
        };

        // Legitimate execution passes
        let ok = enforce_a1155_interlock("CARD", Some(&card), secret, amount, rcp);
        assert!(ok.is_ok());

        // Tampering with amount must fail
        let tampered = enforce_a1155_interlock("CARD", Some(&card), secret, 9999, rcp);
        assert_eq!(tampered, Err(A1155Error::SignatureMismatch));
    }

    #[test]
    fn test_type_b_signature_mechanism_receipt_flow() {
        let cfg = FiscalHardwareConfig::default();
        let driver = TypeBFiscalSignatureDriver::new(cfg);
        let receipt = sample_receipt("CASH", None);

        let res = driver.process_receipt(&receipt).unwrap();
        assert!(res.success);
        assert!(res.fiscal_signature.starts_with("_#_"));
        assert!(res.fiscal_signature.ends_with("_#_"));
        assert_eq!(res.fiscal_signature.len(), 46); // _#_ (3) + 40 chars + _#_ (3) = 46
    }

    #[test]
    fn test_cash_register_ascii_stream() {
        let cfg = FiscalHardwareConfig::default();
        let driver = CashRegisterDriver::new(cfg);
        let receipt = sample_receipt("CASH", None);

        let stream = driver.format_cash_register_stream(&receipt);
        assert!(stream.contains("H;1042;2026-10-04T12:00:00Z\n"));
        assert!(stream.contains("P;Αλλαγή Οθόνης OLED;1.000;100.00;D1\n"));
        assert!(stream.contains("T;CASH;100.00\n"));
        assert!(stream.ends_with("C\n"));
    }

    #[test]
    fn test_sqlite_pos_settings_and_card_transaction_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        init_fiscal_pos_schema(&conn).unwrap();

        let mut cfg = load_fiscal_pos_config(&conn);
        assert_eq!(cfg.device_type, FiscalDeviceType::CashRegisterTypeA);
        assert!(cfg.eft_pos_interlock_enabled);

        cfg.device_type = FiscalDeviceType::SignatureMechanismTypeB;
        cfg.eft_pos_terminal_id = "TERMINAL-77".to_string();
        save_fiscal_pos_config(&conn, &cfg).unwrap();

        let loaded = load_fiscal_pos_config(&conn);
        assert_eq!(loaded.device_type, FiscalDeviceType::SignatureMechanismTypeB);
        assert_eq!(loaded.eft_pos_terminal_id, "TERMINAL-77");

        let req = PosInitiateRequest {
            transaction_id: "TX-SQL-1".to_string(),
            operation: PosTransactionType::Sale,
            amount_cents: 2550,
            currency_code: 978,
            receipt_number: 501,
            invoice_type: "11.1".to_string(),
            cashier_id: "OPERATOR-01".to_string(),
        };

        let pos_driver = EftPosDriver::new(loaded);
        let resp = pos_driver.initiate_card_transaction(&req).unwrap();
        assert_eq!(resp.status, PosTransactionStatus::Approved);

        record_pos_card_transaction(&conn, &req, &resp).unwrap();
        let fetched = get_pos_card_transaction(&conn, "TX-SQL-1").unwrap();
        assert_eq!(fetched.transaction_id, "TX-SQL-1");
        assert_eq!(fetched.terminal_id, "TERMINAL-77");
        assert_eq!(fetched.status, PosTransactionStatus::Approved);
    }

    #[test]
    fn test_a1155_receipt_slip_formatting() {
        let card = PosTerminalResponse {
            transaction_id: "TX-SLIP".to_string(),
            status: PosTransactionStatus::Approved,
            rrn: "410415999000".to_string(),
            auth_code: "883311".to_string(),
            terminal_id: "TID01".to_string(),
            merchant_id: "MID01".to_string(),
            card_brand: "VISA".to_string(),
            card_masked: "**** 9999".to_string(),
            stan: "000501".to_string(),
            interconnection_signature: "A1155:ABCDEF1234567890".to_string(),
            timestamp: 1728043200000,
            raw_response_code: "00".to_string(),
            response_message: "OK".to_string(),
        };

        let slip_bytes = format_a1155_receipt_slip(&card, 48);
        assert!(!slip_bytes.is_empty());
        let text = String::from_utf8_lossy(&slip_bytes);
        assert!(text.contains("A.1155/2023"));
        assert!(text.contains("TID01"));
        assert!(text.contains("410415999000"));
        assert!(text.contains("883311"));
    }
}

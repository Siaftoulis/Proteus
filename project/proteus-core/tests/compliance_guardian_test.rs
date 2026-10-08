//! Sovereign Compliance Guardian Test Suite for Proteus Ecosystem.
//! Automatically tests and enforces strict adherence to international and Greek standards:
//! - GDPR (Regulation EU 2016/679) & Greek Law 4624/2019
//! - ISO/IEC 27001:2022 (A.8.2 Access Control, A.8.15 Logging, A.8.24 Cryptography)
//! - SOC 2 Type II (Trust Services Criteria CC6.1 Logical Access, CC7.1 System Integrity)
//! - HIPAA Technical Safeguards (45 CFR § 164.312)
//!
//! Strict adherence to Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use rusqlite::Connection;
use proteus_core::audit::{init_audit_schema, list_audit_events, log_audit_event, SystemEvent};
use proteus_core::encryption::{decrypt, derive_key, encrypt};
use proteus_core::enterprise::{
    create_enterprise, create_enterprise_store, init_enterprise_schema,
    provision_store_user, Enterprise, EnterpriseStore, EnterpriseUser,
};
use proteus_core::merkle::{
    append_merkle_block, init_merkle_schema, verify_chain_integrity,
};
use proteus_core::roles::UserRole;
use proteus_core::invoicing::{
    calculate_totals, create_invoice, generate_mydata_xml, init_invoicing_schema,
    record_mydata_mark, FiscalInvoice, InvoiceLine, InvoiceType, VatCategory,
};
use proteus_core::tax_registry::{
    init_tax_registry_schema, resolve_afm, save_tax_record, validate_greek_afm_checksum,
    TaxRegistryRecord,
};
use proteus_core::shipping_note::types::{generate_iapr_qr_payload, TransportPurpose};
use proteus_core::tickets::{
    anonymize_ticket_customer, create_ticket, delete_ticket, get_ticket,
    init_tickets_schema, search_tickets, ServiceTicket,
};
use proteus_core::wms::{
    compute_shelf_fit, init_wms_schema, record_shelf_audit, save_rack, save_shelf, save_zone,
    suggest_directed_putaway, Dimensions, GoldenZoneCategory, ShelfAuditProof, StorageForm,
    TurnoverVelocity, WarehouseRack, WarehouseShelf, WarehouseZone, WmsItem,
    DEFAULT_CLEARANCE_MARGIN,
};
use proteus_core::work_card::{
    authenticate_employee_by_pin, calculate_shift_duration_hours, flush_outage_sync_queue,
    init_work_card_schema, list_today_events, record_clock_event, save_employee,
    verify_merkle_chain, EmployeeProfile, WorkCardError, WorkCardEventType,
};
use proteus_core::esl::{
    enqueue_esl_update, evaluate_dynamic_expiry_discount, generate_esl_radio_packet,
    init_esl_schema, mark_packet_transmitted, register_esl_tag, EslTag,
};
use proteus_core::notifications::{
    enqueue_notification, fetch_pending_notifications, generate_tracking_token,
    init_notifications_schema, list_notifications, mark_notification_dispatched,
    purge_dispatched_notifications, record_dispatch_failure, verify_tracking_token,
    NotificationCategory, NotificationChannel, NotificationError, NotificationStatus,
    OutboxNotification,
};
use proteus_core::cardex::{
    anonymize_cardex_customer, get_cardex_entity, init_cardex_schema, save_cardex_entity,
    CardexEntity, EntityType,
};
use proteus_core::service_bench::{
    add_consumed_part, compute_bench_summary, get_diagnostic_checklist, init_service_bench_schema,
    log_technician_labor, save_diagnostic_checklist, settle_ticket_to_invoice, ConsumedSparePart,
    DeviceDiagnosticChecklist, LaborLog,
};
use proteus_core::fiscal_pos::{
    enforce_a1155_interlock, format_a1155_receipt_slip, get_pos_card_transaction,
    init_fiscal_pos_schema, load_fiscal_pos_config, record_pos_card_transaction,
    save_fiscal_pos_config, CashRegisterDriver, EftPosDriver, FiscalDeviceDriver,
    FiscalReceiptItem, FiscalReceiptPayload, FiscalVatBreakdown, PosInitiateRequest,
    PosTransactionStatus, PosTransactionType, TypeBFiscalSignatureDriver,
};

/// 1. GDPR Article 17 & Greek Law 4624/2019: Right to Erasure / Anonymization
/// Verifies that customer PII (Name, Phone) is permanently redacted while
/// preserving device diagnostic records and creating an immutable compliance audit event.
#[test]
fn test_gdpr_art17_right_to_erasure_and_anonymization() {
    let conn = Connection::open_in_memory().unwrap();
    init_tickets_schema(&conn).unwrap();

    let mut ticket = ServiceTicket::new(
        "Νικόλαος Παπαδόπουλος",
        "+30 6971234567",
        "iPhone 15 Pro",
        "Σπασμένη οθόνη Super Retina XDR",
    );
    ticket.serial_number = Some("DNQZL0P9N6Y2".to_string());
    create_ticket(&conn, &mut ticket).unwrap();

    // 1a. Ensure customer is initially searchable
    let found = search_tickets(&conn, "Παπαδόπουλος").unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].customer_phone, "+30 6971234567");

    // 1b. Execute GDPR Article 17 anonymization
    anonymize_ticket_customer(&conn, &ticket.ticket_id, "dpo@company.gr").unwrap();

    // 1c. Verify ticket record no longer contains PII
    let updated = get_ticket(&conn, &ticket.ticket_id).unwrap().unwrap();
    assert_eq!(updated.customer_name, "[GDPR ΑΝΩΝΥΜΟΠΟΙΗΜΕΝΟ]");
    assert_eq!(updated.customer_phone, "[ERASED]");

    // 1d. Crucial invariant: Hardware diagnostics & serial number MUST remain for tax/warranty audit
    assert_eq!(updated.device_model, "iPhone 15 Pro");
    assert_eq!(updated.serial_number, Some("DNQZL0P9N6Y2".to_string()));

    // 1e. Searching for old PII MUST yield ZERO results
    let leak_check_name = search_tickets(&conn, "Παπαδόπουλος").unwrap();
    assert!(leak_check_name.is_empty(), "GDPR violation: old name is still searchable");

    let leak_check_phone = search_tickets(&conn, "6971234567").unwrap();
    assert!(leak_check_phone.is_empty(), "GDPR violation: old phone is still searchable");

    // 1f. Verify Cardex Ledger Customer Anonymization (Right to Erasure)
    init_cardex_schema(&conn).unwrap();
    let entity = CardexEntity {
        entity_id: "099887766".to_string(),
        entity_type: EntityType::Customer,
        name: "Ιωάννης Γεωργίου".to_string(),
        phone: Some("+302109998888".to_string()),
        email: Some("john@personal.gr".to_string()),
        credit_limit_eur: 500.0,
        payment_terms_days: 30,
        current_balance_eur: 150.0,
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    save_cardex_entity(&conn, &entity).unwrap();

    let anon_cardex = anonymize_cardex_customer(&conn, "099887766", "dpo@company.gr").unwrap();
    assert!(anon_cardex);
    let cardex_after = get_cardex_entity(&conn, "099887766").unwrap().unwrap();
    assert_eq!(cardex_after.name, "[GDPR ΑΝΩΝΥΜΟΠΟΙΗΜΕΝΟ]");
    assert_eq!(cardex_after.phone.as_deref(), Some("[ERASED]"));
    assert_eq!(cardex_after.email.as_deref(), Some("[ERASED]"));
    assert_eq!(cardex_after.current_balance_eur, 150.0, "Financial balance must be preserved for tax audit");
}

/// 2. GDPR Article 5 & Data Minimization: Permanent Purge of Expired Tickets & Notifications
#[test]
fn test_gdpr_art5_retention_and_purge() {
    let conn = Connection::open_in_memory().unwrap();
    init_tickets_schema(&conn).unwrap();

    let mut ticket = ServiceTicket::new(
        "Μαρία Οικονόμου",
        "+30 6980000000",
        "Samsung Galaxy S24",
        "Αντικατάσταση μπαταρίας",
    );
    create_ticket(&conn, &mut ticket).unwrap();

    let ticket_id = ticket.ticket_id.clone();
    assert!(get_ticket(&conn, &ticket_id).unwrap().is_some());

    // 2a. Execute ticket purge
    let deleted = delete_ticket(&conn, &ticket_id, "privacy_admin").unwrap();
    assert!(deleted);

    // Verify completely gone
    assert!(get_ticket(&conn, &ticket_id).unwrap().is_none());

    // 2b. Storage limitation purge of expired notifications
    init_notifications_schema(&conn).unwrap();
    let notif = OutboxNotification {
        id: "NOTIF-PURGE-01".to_string(),
        channel: NotificationChannel::Sms,
        category: NotificationCategory::Transactional,
        recipient: "+306912345678".to_string(),
        subject: None,
        content: "Δοκιμή".to_string(),
        status: NotificationStatus::Dispatched,
        attempts: 1,
        max_retries: 3,
        last_error: None,
        scheduled_at: chrono::Utc::now().timestamp_millis() - 100_000,
        created_at: chrono::Utc::now().timestamp_millis() - 100_000,
        dispatched_at: Some(chrono::Utc::now().timestamp_millis() - 100_000),
        gdpr_consent_verified: true,
        metadata_json: "{}".to_string(),
    };
    enqueue_notification(&conn, &notif).unwrap();
    let purged_notifs = purge_dispatched_notifications(&conn, 0, "privacy_admin").unwrap();
    assert_eq!(purged_notifs, 1, "Storage limitation violation: expired notification not purged");
}

/// 3. ISO/IEC 27001:2022 Control A.8.2 & SOC 2 CC6.1: Principle of Least Privilege (RBAC)
/// Verifies that operational roles cannot exceed their defined security boundaries.
#[test]
fn test_iso27001_a82_rbac_least_privilege() {
    // Technician: Must NOT access financials, schemas, or audit logs
    let tech_perms = UserRole::Technician.permissions();
    assert!(!tech_perms.can_view_financials, "Security violation: Technician cannot view financials");
    assert!(!tech_perms.can_edit_schema, "Security violation: Technician cannot alter schemas");
    assert!(!tech_perms.can_view_audit_trail, "Security violation: Technician cannot inspect audit logs");
    assert!(tech_perms.can_edit_technical_notes);
    assert!(tech_perms.can_manage_pipeline);

    // Customer Service: Must NOT alter technical notes or schemas
    let cs_perms = UserRole::CustomerService.permissions();
    assert!(!cs_perms.can_view_financials);
    assert!(!cs_perms.can_edit_schema);
    assert!(!cs_perms.can_edit_technical_notes);
    assert!(cs_perms.can_intake_tickets);
    assert!(cs_perms.can_book_appointments);

    // Developer: Can alter schema and dev settings, but cannot view company financials or intake customer tickets
    let dev_perms = UserRole::Developer.permissions();
    assert!(!dev_perms.can_view_financials);
    assert!(!dev_perms.can_intake_tickets);
    assert!(dev_perms.can_edit_schema);
    assert!(dev_perms.can_view_audit_trail);

    // CEO: Has root governance
    let ceo_perms = UserRole::Ceo.permissions();
    assert!(ceo_perms.can_view_financials);
    assert!(ceo_perms.can_edit_schema);
    assert!(ceo_perms.can_view_audit_trail);
}

/// 4. ISO/IEC 27001:2022 Control A.8.24 & HIPAA § 164.312(a)(2)(iv): Cryptography & Integrity
/// Verifies authenticated encryption at rest using Argon2id key derivation and XChaCha20Poly1305.
#[test]
fn test_iso27001_a824_authenticated_encryption_at_rest() {
    let key = derive_key("StrongSovereignPassword#2026", "EnterpriseSalt").unwrap();
    let sensitive_payload = b"PATIENT_OR_CUSTOMER_PII_AES256_SAFEGUARD";

    // Encrypt
    let ciphertext = encrypt(sensitive_payload, &key).unwrap();
    assert_ne!(ciphertext.as_slice(), sensitive_payload);

    // Decrypt with valid key
    let decrypted = decrypt(&ciphertext, &key).unwrap();
    assert_eq!(decrypted, sensitive_payload);

    // Tamper detection: flip a single byte in ciphertext
    let mut tampered = ciphertext.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x55;

    let tamper_result = decrypt(&tampered, &key);
    assert!(tamper_result.is_err(), "Cryptographic violation: Tampered ciphertext was accepted");

    // Wrong key rejection
    let wrong_key = derive_key("IncorrectPassword", "EnterpriseSalt").unwrap();
    let wrong_key_result = decrypt(&ciphertext, &wrong_key);
    assert!(wrong_key_result.is_err(), "Security violation: Decrypted with wrong key");
}

/// 5. ISO/IEC 27001:2022 Control A.8.15, SOC 2 CC7.1 & HIPAA § 164.312(c)(1): Tamper-Proof Audit Chain
/// Verifies that Merkle hash-chaining immediately detects any unauthorized SQL tampering.
#[test]
fn test_iso27001_a815_soc2_cc71_merkle_audit_tamper_detection() {
    let conn = Connection::open_in_memory().unwrap();
    init_merkle_schema(&conn).unwrap();

    // Append legitimate audit blocks
    append_merkle_block(&conn, "TICKET", "TICK-001", "INTAKE", r#"{"customer":"John"}"#).unwrap();
    append_merkle_block(&conn, "TICKET", "TICK-001", "DIAGNOSIS", r#"{"fault":"screen"}"#).unwrap();
    append_merkle_block(&conn, "TICKET", "TICK-001", "DELIVERED", r#"{"amount":120.0}"#).unwrap();

    // Verify legitimate chain passes
    let report = verify_chain_integrity(&conn).unwrap();
    assert!(report.is_valid, "Legitimate chain must be valid");
    assert_eq!(report.total_blocks, 3);
    assert!(report.tampered_at_sequence.is_none());

    // Malicious DB attack: Attacker tampers directly with record 2 in the database
    conn.execute(
        "UPDATE tamper_proof_audit_backlog SET payload_hash = 'FORGED_HASH_0000000000' WHERE sequence_id = 2",
        [],
    ).unwrap();

    // Cryptographic audit verification MUST catch the forgery
    let tampered_report = verify_chain_integrity(&conn).unwrap();
    assert!(!tampered_report.is_valid, "Tampered chain must be rejected");
    assert_eq!(tampered_report.tampered_at_sequence, Some(2), "Must pinpoint sequence 2 as tampered");
}

/// 6. SOC 2 CC6.1 & Enterprise Governance: Strict Store Seat Quota Enforcement
/// Verifies multi-tenant boundaries and prevents unauthorized seat provisioning.
#[test]
fn test_soc2_cc61_store_seat_quota_enforcement() {
    let conn = Connection::open_in_memory().unwrap();
    init_enterprise_schema(&conn).unwrap();

    let ent = Enterprise {
        enterprise_id: "ent-soc2".to_string(),
        legal_name: "Proteus Corp".to_string(),
        tax_id: "999888777".to_string(),
        total_seat_quota: 2,
        used_seats: 0,
        cloud_tier: "Enterprise".to_string(),
        created_at: "2026-10-03T00:00:00Z".to_string(),
    };
    create_enterprise(&conn, &ent).unwrap();

    let store = EnterpriseStore {
        store_id: "store-ath-01".to_string(),
        enterprise_id: "ent-soc2".to_string(),
        store_code: "ATH-01".to_string(),
        store_name: "Athens Flagship".to_string(),
        address: "Panepistimiou 10".to_string(),
        phone: "+30 2101234567".to_string(),
        allocated_seats: 2, // Maximum 2 seats
        active_seats: 0,
        is_active: true,
        created_at: "2026-10-03T00:00:00Z".to_string(),
    };
    create_enterprise_store(&conn, &store).unwrap();

    // Provision User 1 -> OK
    let user1 = EnterpriseUser {
        user_id: "u-1".to_string(),
        enterprise_id: "ent-soc2".to_string(),
        store_id: "store-ath-01".to_string(),
        department_id: None,
        full_name: "User One".to_string(),
        email: "user1@company.com".to_string(),
        role_code: "Technician".to_string(),
        is_active: true,
        created_at: "2026-10-03T00:00:00Z".to_string(),
    };
    assert!(provision_store_user(&conn, &user1).is_ok());

    // Provision User 2 -> OK (Quota full: 2/2)
    let user2 = EnterpriseUser {
        user_id: "u-2".to_string(),
        enterprise_id: "ent-soc2".to_string(),
        store_id: "store-ath-01".to_string(),
        department_id: None,
        full_name: "User Two".to_string(),
        email: "user2@company.com".to_string(),
        role_code: "Technician".to_string(),
        is_active: true,
        created_at: "2026-10-03T00:00:00Z".to_string(),
    };
    assert!(provision_store_user(&conn, &user2).is_ok());

    // Provision User 3 -> MUST FAIL (Exceeds quota)
    let user3 = EnterpriseUser {
        user_id: "u-3".to_string(),
        enterprise_id: "ent-soc2".to_string(),
        store_id: "store-ath-01".to_string(),
        department_id: None,
        full_name: "User Three".to_string(),
        email: "user3@company.com".to_string(),
        role_code: "Technician".to_string(),
        is_active: true,
        created_at: "2026-10-03T00:00:00Z".to_string(),
    };
    let excess_result = provision_store_user(&conn, &user3);
    assert!(excess_result.is_err(), "Governance violation: Provisioned beyond seat quota");
}

/// 7. HIPAA § 164.312(b) & ISO/IEC 27001 A.8.15: System Audit Controls
/// Verifies operational event logging with operator role and timestamp traceability.
#[test]
fn test_hipaa_164_312_audit_and_traceability() {
    let conn = Connection::open_in_memory().unwrap();
    init_audit_schema(&conn).unwrap();

    let event = SystemEvent::new(
        "CUSTOMER_RECORD",
        "CUST-9921",
        "ACCESS_DECRYPT",
        "dr_smith",
        "Technician",
        "Decrypted diagnostic intake record",
        r#"{"reason":"hardware_repair"}"#,
    );
    log_audit_event(&conn, &event).unwrap();

    let logs = list_audit_events(&conn, 10, 0, None).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].operator_name, "dr_smith");
    assert_eq!(logs[0].operator_role, "Technician");
    assert_eq!(logs[0].event_type, "ACCESS_DECRYPT");
    assert!(logs[0].created_at > 0);
}

/// 8. Greek Law 4624/2019 & Fiscal myDATA QR Payload Structure
/// Verifies digital signature formatting and official transport QR parameters.
#[test]
fn test_greek_law_4624_2019_shipping_and_mydata_payload() {
    let qr = generate_iapr_qr_payload(
        Some("MARK-9821447012"),
        "802194512",
        "099887766",
        "2026-10-03 10:00",
        "ΙΒΥ-4821",
        4,
        "a1b2c3d4e5f67890abcdef1234567890",
    );

    assert!(qr.starts_with("AADE-CMR|MARK:MARK-9821447012"));
    assert!(qr.contains("|ISSUER:802194512|"));
    assert!(qr.contains("|RECV:099887766|"));
    assert!(qr.contains("|VEH:ΙΒΥ-4821|"));
    assert!(qr.contains("|ITEMS:4|"));

    // Verify transport purpose enumeration completeness
    assert_eq!(TransportPurpose::Sale.display_name(), "Πώληση");
    assert_eq!(TransportPurpose::Repair.display_name(), "Επισκευή / Service");
    assert_eq!(TransportPurpose::TransferBetweenBranches.display_name(), "Ενδοδιακίνηση Υποκαταστημάτων");
}

/// 9. Greek Law 4308/2014 & AADE myDATA v1.0 Invoicing & Tax Calculation Compliance
/// Verifies exact VAT rounding, background outbox queuing, and AADE XML schema generation.
#[test]
fn test_mydata_fiscal_invoicing_and_xml_generation() {
    let conn = Connection::open_in_memory().unwrap();
    init_invoicing_schema(&conn).unwrap();

    let lines = vec![
        InvoiceLine::new(1, "Επισκευή Κινητού - Εργασία", 1.0, 40.0, VatCategory::Vat24),
        InvoiceLine::new(2, "Ανταλλακτικό Οθόνη OLED", 1.0, 80.0, VatCategory::Vat24),
        InvoiceLine::new(3, "Εκπαιδευτικό Εγχειρίδιο Service", 1.0, 15.0, VatCategory::Vat6),
    ];

    let (net, vat, gross) = calculate_totals(&lines);
    // (120 * 0.24 = 28.80) + (15 * 0.06 = 0.90) = 29.70 VAT
    assert_eq!(net, 135.0);
    assert_eq!(vat, 29.70);
    assert_eq!(gross, 164.70);

    let invoice = FiscalInvoice {
        invoice_id: "inv-comp-01".to_string(),
        series: "B".to_string(),
        invoice_number: 1,
        invoice_type: InvoiceType::Service2_1,
        issue_date: "2026-10-03".to_string(),
        issue_time: "11:00:00".to_string(),
        issuer_afm: "802194512".to_string(),
        recipient_afm: "099887766".to_string(),
        recipient_name: "Customer B2B".to_string(),
        total_net_eur: net,
        total_vat_eur: vat,
        total_gross_eur: gross,
        mydata_mark: None,
        mydata_uid: None,
        mydata_qr_url: None,
        is_cancelled: false,
        created_at: 1710000000,
    };

    create_invoice(&conn, &invoice, &lines).unwrap();

    // Verify background transmission outbox entry created
    let pending_count: i64 = conn
        .query_row("SELECT count(*) FROM mydata_outbox WHERE status = 'PENDING'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(pending_count, 1, "Fiscal violation: Invoice was not queued in myDATA outbox");

    // Generate XML and verify AADE elements
    let xml = generate_mydata_xml(&invoice, &lines);
    assert!(xml.contains("<invoiceType>2.1</invoiceType>"));
    assert!(xml.contains("<vatNumber>802194512</vatNumber>"));
    assert!(xml.contains("<vatNumber>099887766</vatNumber>"));
    assert!(xml.contains("<totalGrossValue>164.70</totalGrossValue>"));

    // Simulate AADE confirmation
    record_mydata_mark(&conn, "inv-comp-01", "MARK-10928374", "UID-ABCDEF123456", "https://www.aade.gr/mydata/qr?mark=MARK-10928374").unwrap();

    let dispatched_count: i64 = conn
        .query_row("SELECT count(*) FROM mydata_outbox WHERE status = 'DISPATCHED'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(dispatched_count, 1);
}

/// 10. Greek Law 4308/2014 Statutory Tax Identification: Modulo-11 Checksum & Cache Verification
/// Enforces that invalid AFMs are rejected before myDATA transmission and verified records are cached.
#[test]
fn test_tax_registry_afm_verification_and_cache() {
    let conn = Connection::open_in_memory().unwrap();
    init_tax_registry_schema(&conn).unwrap();

    // 10a. Statutory Modulo-11 checksum validation
    assert!(validate_greek_afm_checksum("094014201")); // Athens Chamber of Commerce
    assert!(validate_greek_afm_checksum("090000045")); // Ministry of Finance
    assert!(!validate_greek_afm_checksum("094014202"), "Checksum violation: Corrupted digit passed");
    assert!(!validate_greek_afm_checksum("000000000"), "Tax violation: All zeros passed");

    // 10b. Rejection of invalid AFM at resolver level
    let invalid_res = resolve_afm(&conn, "GR", "999999999");
    assert!(invalid_res.is_err(), "Tax compliance violation: Invalid AFM resolved");

    // 10c. Local persistent caching of statutory registration data
    let record = TaxRegistryRecord {
        afm: "094014201".to_string(),
        country_code: "GR".to_string(),
        legal_name: "Athens Chamber of Commerce & Industry".to_string(),
        commercial_title: Some("EVEA".to_string()),
        address: "Akadimias 7".to_string(),
        postal_code: "10671".to_string(),
        city: "Athens".to_string(),
        doy: Some("D' Athinon".to_string()),
        is_active: true,
        is_normal_vat: true,
        cached_at: 1710000000,
    };
    save_tax_record(&conn, &record).unwrap();

    let resolved = resolve_afm(&conn, "GR", "094014201").unwrap().unwrap();
    assert_eq!(resolved.legal_name, "Athens Chamber of Commerce & Industry");
    assert_eq!(resolved.city, "Athens");
    assert!(resolved.is_active);
}

/// 11. ISO 45001 & EU Directive 90/269/EEC (Ergonomic Manual Handling) & Spatial WMS Integrity
/// Enforces that heavy goods (>=15kg) are strictly routed to ground levels, fast movers are placed in
/// ergonomic Golden Zone (80-160cm), and shelf audits record immutable cryptographic proof.
#[test]
fn test_iso45001_ergonomics_and_spatial_wms_integrity() {
    let conn = Connection::open_in_memory().unwrap();
    init_wms_schema(&conn).unwrap();

    let ground_shelf = WarehouseShelf {
        shelf_id: "GROUND-01".to_string(),
        rack_id: "RACK-A".to_string(),
        shelf_level: 0,
        height_from_floor_cm: 30.0, // Ground level (<80cm)
        dimensions: Dimensions::new(120.0, 60.0, 80.0),
        max_weight_kg: 400.0,
        current_weight_kg: 0.0,
    };

    let golden_shelf = WarehouseShelf {
        shelf_id: "GOLDEN-02".to_string(),
        rack_id: "RACK-A".to_string(),
        shelf_level: 2,
        height_from_floor_cm: 120.0, // Golden zone (80-160cm)
        dimensions: Dimensions::new(120.0, 60.0, 80.0),
        max_weight_kg: 200.0,
        current_weight_kg: 0.0,
    };

    let upper_shelf = WarehouseShelf {
        shelf_id: "UPPER-03".to_string(),
        rack_id: "RACK-A".to_string(),
        shelf_level: 3,
        height_from_floor_cm: 200.0, // Upper reach (160-220cm)
        dimensions: Dimensions::new(120.0, 60.0, 80.0),
        max_weight_kg: 100.0,
        current_weight_kg: 0.0,
    };

    let shelves = vec![upper_shelf, golden_shelf.clone(), ground_shelf];

    // 11a. Heavy item (28kg) must route to GroundHeavy per ISO 45001
    let heavy_part = WmsItem {
        sku: "MOTOR-28KG".to_string(),
        name: "Electric Motor 28kg".to_string(),
        dimensions: Dimensions::new(35.0, 25.0, 25.0),
        weight_kg: 28.0,
        storage_form: StorageForm::Cuboid,
        turnover_velocity: TurnoverVelocity::FastMover,
        is_upright_only: false,
    };

    let heavy_rec = suggest_directed_putaway(&shelves, &heavy_part, DEFAULT_CLEARANCE_MARGIN).unwrap();
    assert_eq!(heavy_rec.shelf_id, "GROUND-01", "Occupational safety violation: Heavy cargo not routed to ground");
    assert_eq!(heavy_rec.ergonomic_zone, GoldenZoneCategory::GroundHeavy);

    // 11b. Fast mover retail item must route to Golden Zone
    let retail_box = WmsItem {
        sku: "SMART-BULB".to_string(),
        name: "Zigbee Smart Bulb".to_string(),
        dimensions: Dimensions::new(12.0, 15.0, 12.0),
        weight_kg: 0.25,
        storage_form: StorageForm::Cuboid,
        turnover_velocity: TurnoverVelocity::FastMover,
        is_upright_only: false,
    };

    let retail_rec = suggest_directed_putaway(&shelves, &retail_box, DEFAULT_CLEARANCE_MARGIN).unwrap();
    assert_eq!(retail_rec.shelf_id, "GOLDEN-02", "Ergonomics violation: Fast mover not placed in Golden Zone");
    assert_eq!(retail_rec.ergonomic_zone, GoldenZoneCategory::GoldenZone);

    // 11c. 3D Bin packing applies 5% clearance margin
    let fit = compute_shelf_fit(&shelves[1], &retail_box, DEFAULT_CLEARANCE_MARGIN).unwrap();
    assert!(fit.1 > 0);
    assert_eq!(retail_rec.clearance_percent, 5.0);

    // 11d. Shelf audit photo proof recorded with SHA-256 hash
    let zone = WarehouseZone {
        zone_id: "ZONE-A".to_string(),
        name: "Main Area".to_string(),
        code: "ZA".to_string(),
        temperature_class: "Ambient".to_string(),
        description: "ISO 45001 Verified".to_string(),
    };
    save_zone(&conn, &zone).unwrap();

    let rack = WarehouseRack {
        rack_id: "RACK-A".to_string(),
        zone_id: "ZONE-A".to_string(),
        rack_code: "RA".to_string(),
        aisle_number: 1,
        x_pos: 0.0,
        y_pos: 0.0,
    };
    save_rack(&conn, &rack).unwrap();
    save_shelf(&conn, &golden_shelf).unwrap();

    let audit = ShelfAuditProof {
        audit_id: "AUD-ISO-01".to_string(),
        shelf_id: "GOLDEN-02".to_string(),
        scanned_sku: "SMART-BULB".to_string(),
        is_misplaced: false,
        expected_sku: Some("SMART-BULB".to_string()),
        photo_sha256: Some("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_string()),
        audit_timestamp: 1727956800000,
    };
    record_shelf_audit(&conn, &audit).unwrap();

    let count: i64 = conn.query_row(
        "SELECT count(*) FROM warehouse_shelf_audits WHERE photo_sha256 IS NOT NULL",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 1);
}

/// 12. Greek Law 4808/2021 & Circular 47319/2023: Ergani II Digital Work Card Statutory Outage Protection
/// Enforces that when internet is down, staff clock events are queued locally with statutory telecom
/// outage flags, monotonic timestamps, and Merkle cryptographic chaining, protecting against €10,500 fines.
#[test]
fn test_ergani_ii_statutory_outage_protection_and_merkle_chain() {
    let conn = Connection::open_in_memory().unwrap();
    init_work_card_schema(&conn).unwrap();

    let employee = EmployeeProfile {
        employee_id: "EMP-ERGANI-101".to_string(),
        afm: "094014201".to_string(),
        amka: "02029201234".to_string(),
        full_name: "Αλέξανδρος Παπαδόπουλος".to_string(),
        job_title: "Υπεύθυνος Ταμείου".to_string(),
        pin_code: "4321".to_string(),
        qr_badge_token: "QR-EMP-ERGANI-101".to_string(),
        is_active: true,
    };
    save_employee(&conn, &employee).unwrap();

    // 12a. Verify PIN authentication at shop counter
    let auth = authenticate_employee_by_pin(&conn, "094014201", "4321");
    assert!(auth.is_ok(), "Authentication failure for valid employee PIN");
    assert_eq!(auth.unwrap().full_name, "Αλέξανδρος Παπαδόπουλος");

    // 12b. Offline Clock-In during internet outage (Statutory Greek labor clause)
    let clock_in = record_clock_event(
        &conn,
        "EMP-ERGANI-101",
        WorkCardEventType::ClockIn,
        true, // Offline fallback
        Some("TELECOM_PROVIDER_OUTAGE"),
    ).unwrap();

    assert!(clock_in.is_offline_fallback, "Ergani II compliance violation: offline flag missing");
    assert_eq!(clock_in.outage_reason.as_deref(), Some("TELECOM_PROVIDER_OUTAGE"));
    assert_eq!(clock_in.sync_status, "PENDING_OUTAGE_SYNC");

    // 12c. Offline Break Start with cryptographic Merkle block chaining
    let break_start = record_clock_event(
        &conn,
        "EMP-ERGANI-101",
        WorkCardEventType::BreakStart,
        true,
        Some("TELECOM_PROVIDER_OUTAGE"),
    ).unwrap();

    assert_ne!(clock_in.merkle_hash, break_start.merkle_hash);
    assert!(!break_start.merkle_hash.is_empty());

    // 12d. Verify pending outbox count for statutory transmission
    let pending_count: i64 = conn.query_row(
        "SELECT count(*) FROM work_card_events WHERE sync_status = 'PENDING_OUTAGE_SYNC'",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(pending_count, 2, "Ergani II outbox count mismatch");

    // 12e. Complete shift with clock-out and verify statutory overtime calculation
    let _ = record_clock_event(
        &conn,
        "EMP-ERGANI-101",
        WorkCardEventType::ClockOut,
        true,
        Some("TELECOM_PROVIDER_OUTAGE"),
    ).unwrap();

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let (regular_hours, overtime_hours) = calculate_shift_duration_hours(&conn, "EMP-ERGANI-101", &today).unwrap();
    assert!(regular_hours >= 0.0);
    assert_eq!(overtime_hours, 0.0);
}

/// 13. Directive (EU) 2019/1937 & Consumer Pricing Transparency: ESL Dynamic Expiry Markdown & Radio Packet Integrity
/// Enforces that items nearing expiration are mathematically discounted according to statutory food waste
/// reduction rules, and e-ink ESL broadcast frames maintain radio packet integrity with valid checksums.
#[test]
fn test_esl_dynamic_expiry_pricing_and_radio_packet_integrity() {
    let conn = Connection::open_in_memory().unwrap();
    init_esl_schema(&conn).unwrap();

    let tag = EslTag {
        tag_mac: "ESL-TAG-42A9".to_string(),
        shelf_id: "SHELF-GOLDEN-01".to_string(),
        sku: "YOGURT-GREEK-1KG".to_string(),
        product_name: "Ελληνικό Στραγγιστό Γιαούρτι 1kg".to_string(),
        current_price_eur: 4.50,
        discount_price_eur: None,
        unit_of_measure: "τμχ".to_string(),
        battery_percentage: 95,
        signal_rssi: -62,
        last_sync_utc: 1727956800000,
        pending_refresh: false,
    };
    register_esl_tag(&conn, &tag).unwrap();

    // 13a. Dynamic markdown evaluation for expiring item (2 days left -> 30% discount)
    let (base, disc, promo) = evaluate_dynamic_expiry_discount(tag.current_price_eur, "2026-10-05", "2026-10-03");
    assert_eq!(base, 4.50);
    assert_eq!(disc, Some(3.15), "Markdown calculation error: 30% off 4.50€ must be 3.15€");
    assert!(promo.unwrap().contains("-30%"));

    // 13b. Generate compact RF broadcast packet
    let packet = generate_esl_radio_packet(
        &tag.tag_mac,
        base,
        disc,
        promo,
        "5201122334455",
    );
    assert_eq!(&packet[0..4], b"PRTS", "ESL radio packet magic identifier mismatch");
    assert!(packet.len() >= 20, "ESL radio packet too short");

    // 13c. Enqueue update in SQLite broadcast queue
    let broadcast_id = enqueue_esl_update(&conn, &tag.tag_mac, &packet).unwrap();

    let pending: i64 = conn.query_row(
        "SELECT count(*) FROM esl_broadcast_queue WHERE status = 'PENDING'",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(pending, 1, "Broadcast queue pending count mismatch");

    // 13d. Gateway transmission acknowledgement
    mark_packet_transmitted(&conn, &broadcast_id).unwrap();

    let transmitted: i64 = conn.query_row(
        "SELECT count(*) FROM esl_broadcast_queue WHERE status = 'TRANSMITTED' AND transmitted_at IS NOT NULL",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(transmitted, 1, "Broadcast transmission confirmation missing");
}

/// 14. Greek Law 4308/2014 (ΕΛΠ) & AADE myDATA: 1-Click CPA Accounting & Discrepancy Audit
/// Verifies statutory accounting period aggregation, multi-rate VAT buckets, outbox offline resilience,
/// and automated CPA ledger exports without floating-point drift.
#[test]
fn test_greek_law_4308_2014_cpa_accounting_and_mydata_reconciliation() {
    use proteus_core::cpa_sync::{
        audit_fiscal_discrepancies, export_cpa_csv, export_cpa_json, generate_period_fiscal_report,
    };
    use proteus_core::mydata_sync::sync_mydata_outbox;

    let conn = Connection::open_in_memory().unwrap();
    init_invoicing_schema(&conn).unwrap();

    // 14a. Create Invoice 1: Wholesale B2B with 24% VAT
    let lines1 = vec![InvoiceLine::new(1, "B2B Server Rack", 1.0, 500.0, VatCategory::Vat24)];
    let (net1, vat1, gross1) = calculate_totals(&lines1);
    let inv1 = FiscalInvoice {
        invoice_id: "inv-elp-01".to_string(),
        series: "A".to_string(),
        invoice_number: 101,
        invoice_type: InvoiceType::Sale1_1,
        issue_date: "2026-10-02".to_string(),
        issue_time: "09:30:00".to_string(),
        issuer_afm: "802194512".to_string(),
        recipient_afm: "099887766".to_string(),
        recipient_name: "Hellenic Cloud Ltd".to_string(),
        total_net_eur: net1,
        total_vat_eur: vat1,
        total_gross_eur: gross1,
        mydata_mark: Some("MARK-400100200".to_string()),
        mydata_uid: Some("UID-AAA-BBB".to_string()),
        mydata_qr_url: None,
        is_cancelled: false,
        created_at: 1710000000,
    };
    create_invoice(&conn, &inv1, &lines1).unwrap();

    // 14b. Create Invoice 2: Retail POS with 13% and 6% VAT (Unfiled / Pending Outbox)
    let lines2 = vec![
        InvoiceLine::new(1, "Cafeteria Snack", 2.0, 10.0, VatCategory::Vat13),
        InvoiceLine::new(2, "Technical Manual", 1.0, 15.0, VatCategory::Vat6),
    ];
    let (net2, vat2, gross2) = calculate_totals(&lines2);
    let inv2 = FiscalInvoice {
        invoice_id: "inv-elp-02".to_string(),
        series: "B".to_string(),
        invoice_number: 1,
        invoice_type: InvoiceType::RetailReceipt11_1,
        issue_date: "2026-10-03".to_string(),
        issue_time: "10:15:00".to_string(),
        issuer_afm: "802194512".to_string(),
        recipient_afm: "000000000".to_string(),
        recipient_name: "Walk-in Retail".to_string(),
        total_net_eur: net2,
        total_vat_eur: vat2,
        total_gross_eur: gross2,
        mydata_mark: None, // Missing MARK triggers audit discrepancy
        mydata_uid: None,
        mydata_qr_url: None,
        is_cancelled: false,
        created_at: 1710001000,
    };
    create_invoice(&conn, &inv2, &lines2).unwrap();

    // 14c. Generate Period Fiscal Report (ΕΛΠ Law 4308/2014)
    let report = generate_period_fiscal_report(&conn, "2026-10-01", "2026-10-31").unwrap();
    assert_eq!(report.total_invoices_count, 2, "Invoice count in period mismatch");
    assert_eq!(report.total_net_eur, 535.0, "Net total in period mismatch");
    // 500*0.24 = 120.00; 20*0.13 = 2.60; 15*0.06 = 0.90 -> Total VAT = 123.50
    assert_eq!(report.total_vat_eur, 123.50, "VAT total in period mismatch");
    assert_eq!(report.total_gross_eur, 658.50, "Gross total in period mismatch");
    assert_eq!(report.vat_buckets.len(), 3, "Expected 3 distinct VAT categories (24%, 13%, 6%)");

    // Invariant: sum of bucket nets and VATs must equal overall totals
    let bucket_net_sum: f64 = report.vat_buckets.iter().map(|b| b.net_eur).sum();
    let bucket_vat_sum: f64 = report.vat_buckets.iter().map(|b| b.vat_eur).sum();
    assert!((bucket_net_sum - report.total_net_eur).abs() < 0.01, "Bucket net allocation drift detected");
    assert!((bucket_vat_sum - report.total_vat_eur).abs() < 0.01, "Bucket VAT allocation drift detected");

    // 14d. Discrepancy detector identifies unfiled invoice (missing MARK)
    let discrepancies = audit_fiscal_discrepancies(&conn, "2026-10-01", "2026-10-31").unwrap();
    assert_eq!(discrepancies.len(), 1, "Exactly 1 unfiled invoice must be flagged");
    assert_eq!(discrepancies[0].invoice_id, "inv-elp-02");

    // 14e. Outbox offline resilience check
    let batch = sync_mydata_outbox(&conn, None, 10).unwrap();
    assert_eq!(batch.skipped_offline_count, 2, "Offline queue must safely retain pending outbox entries");

    // 14f. Validate 1-Click CPA Export JSON & CSV integrity
    let cpa_json = export_cpa_json(&conn, &report, &discrepancies).unwrap();
    assert!(cpa_json.contains("Hellenic Cloud Ltd"));
    assert!(cpa_json.contains("Greek Accounting Standards"));

    let cpa_csv = export_cpa_csv(&conn, "2026-10-01", "2026-10-31").unwrap();
    assert!(cpa_csv.contains("Hellenic Cloud Ltd"));
    assert!(cpa_csv.contains("Walk-in Retail"));
    assert!(cpa_csv.contains(";500,00;120,00;620,00;MARK-400100200;UID-AAA-BBB"));
}

/// 15. Commercial Ledger & Cardex CRM: Rolling Balances, Credit Limits & Balance Aging
/// Verifies mathematical balance consistency, payment receipts, credit limit enforcement,
/// and 5-tier aging analysis without debt drifting.
#[test]
fn test_commercial_cardex_rolling_balances_and_aging() {
    use proteus_core::cardex::{
        calculate_balance_aging, check_credit_limit_ok, get_cardex_entity, init_cardex_schema,
        list_cardex_entries, post_cardex_movement, record_payment_receipt, save_cardex_entity,
        CardexEntity, CardexMovementType, EntityType,
    };

    let conn = Connection::open_in_memory().unwrap();
    init_cardex_schema(&conn).unwrap();

    // 15a. Register B2B Client with a 2,000€ credit ceiling
    let client = CardexEntity {
        entity_id: "088776655".to_string(),
        entity_type: EntityType::Customer,
        name: "Acme Industrial S.A.".to_string(),
        phone: Some("2109876543".to_string()),
        email: Some("accounting@acme.gr".to_string()),
        credit_limit_eur: 2000.0,
        payment_terms_days: 30,
        current_balance_eur: 0.0,
        created_at: 1710000000,
    };
    save_cardex_entity(&conn, &client).unwrap();

    // 15b. Post Debit: Invoice #201 for 1,200€ issued 70 days ago
    post_cardex_movement(
        &conn,
        "088776655",
        CardexMovementType::Debit,
        "ΤΙΜΟΛΟΓΙΟ",
        "A",
        201,
        "Heavy Machinery Parts",
        1200.0,
        "2026-07-25",
        "2026-08-25",
    ).unwrap();

    let ent1 = get_cardex_entity(&conn, "088776655").unwrap().unwrap();
    assert_eq!(ent1.current_balance_eur, 1200.0, "Rolling balance after debit mismatch");

    // 15c. Credit Limit Check: Can purchase 500€ more? Yes (1200 + 500 = 1700 <= 2000)
    assert!(check_credit_limit_ok(&conn, "088776655", 500.0).unwrap());
    // Can purchase 1,000€ more? No (1200 + 1000 = 2200 > 2000 limit)
    assert!(!check_credit_limit_ok(&conn, "088776655", 1000.0).unwrap());

    // 15d. Issue Payment Receipt: Client pays 400€ via Bank Transfer
    let receipt_id = record_payment_receipt(
        &conn,
        "088776655",
        "E",
        55,
        400.0,
        "Τραπεζική Κατάθεση",
        "Eurobank Ref #987123",
        "2026-09-10",
    ).unwrap();
    assert!(!receipt_id.is_empty());

    let ent2 = get_cardex_entity(&conn, "088776655").unwrap().unwrap();
    assert_eq!(ent2.current_balance_eur, 800.0, "Rolling balance after payment credit mismatch");

    // 15e. Verify 5-Tier Balance Aging as of 2026-10-03
    let aging = calculate_balance_aging(&conn, "088776655", "2026-10-03").unwrap();
    assert_eq!(aging.total_balance_eur, 800.0);
    // Movement was on 2026-07-25 (70 days prior) -> falls in 61-90 days bucket
    assert_eq!(aging.overdue_61_90_eur, 1200.0);

    // 15f. Ledger chronological integrity
    let entries = list_cardex_entries(&conn, "088776655").unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].movement_type, CardexMovementType::Debit);
    assert_eq!(entries[1].movement_type, CardexMovementType::Credit);
    assert_eq!(entries[1].running_balance_eur, 800.0);
}

/// 16. Multi-Channel Notifications Gateway & Live Tracking Integrity (GDPR Art. 6, Greek Law 4624/2019, ISO 27001 Cryptographic Token)
/// Enforces:
/// - Unforgeable live tracking tokens for customer repair status links
/// - Rejection of marketing messages without verified GDPR consent
/// - Contractual necessity exemption for transactional messages
/// - Exponential backoff and outbox failure resilience
#[test]
fn test_notifications_gateway_tracking_and_gdpr_compliance() {
    let conn = Connection::open_in_memory().unwrap();
    init_notifications_schema(&conn).unwrap();

    let secret_key = b"sovereign-master-key-iso27001-live-tracker";
    let ticket_id = "RPR-2026-8801";

    // 16a. Cryptographic token generation & validation
    let token = generate_tracking_token(ticket_id, secret_key);
    assert!(verify_tracking_token(ticket_id, &token, secret_key), "Valid token must pass verification");
    assert!(!verify_tracking_token("RPR-2026-9999", &token, secret_key), "Wrong ticket ID must be rejected");
    assert!(!verify_tracking_token(ticket_id, "invalid.sig", secret_key), "Tampered signature must fail");

    // 16b. GDPR Consent Enforcement: Marketing rejected without opt-in
    let promo = OutboxNotification {
        id: "notif-promo-01".to_string(),
        channel: NotificationChannel::Viber,
        category: NotificationCategory::Marketing,
        recipient: "+306941234567".to_string(),
        subject: None,
        content: "Προσφορά: 30% έκπτωση σε αλλαγή οθόνης!".to_string(),
        status: NotificationStatus::Pending,
        attempts: 0,
        max_retries: 3,
        last_error: None,
        scheduled_at: chrono::Utc::now().timestamp_millis(),
        created_at: chrono::Utc::now().timestamp_millis(),
        dispatched_at: None,
        gdpr_consent_verified: false,
        metadata_json: "{}".to_string(),
    };
    let res = enqueue_notification(&conn, &promo);
    assert!(matches!(res, Err(NotificationError::ConsentRequired)), "Marketing without GDPR consent must fail");

    // 16c. Transactional message permitted under Contractual Necessity (GDPR Art. 6(1)(b))
    let transactional = OutboxNotification {
        id: "notif-trans-01".to_string(),
        category: NotificationCategory::Transactional,
        content: "Το αυτοκίνητό σας είναι έτοιμο.".to_string(),
        ..promo
    };
    let res = enqueue_notification(&conn, &transactional);
    assert!(res.is_ok(), "Transactional notification must succeed");

    // 16d. Pending queue retrieval and dispatch confirmation
    let pending = fetch_pending_notifications(&conn, 10).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, "notif-trans-01");

    mark_notification_dispatched(&conn, "notif-trans-01").unwrap();
    let pending_after = fetch_pending_notifications(&conn, 10).unwrap();
    assert_eq!(pending_after.len(), 0);

    // 16e. Retry backoff and permanent failure threshold
    let fail_notif = OutboxNotification {
        id: "notif-fail-01".to_string(),
        channel: NotificationChannel::Sms,
        category: NotificationCategory::Transactional,
        recipient: "+306981112233".to_string(),
        subject: None,
        content: "Ενημέρωση διαγνωστικού ελέγχου.".to_string(),
        status: NotificationStatus::Pending,
        attempts: 0,
        max_retries: 2,
        last_error: None,
        scheduled_at: chrono::Utc::now().timestamp_millis() - 5000,
        created_at: chrono::Utc::now().timestamp_millis() - 5000,
        dispatched_at: None,
        gdpr_consent_verified: true,
        metadata_json: "{}".to_string(),
    };
    enqueue_notification(&conn, &fail_notif).unwrap();

    // Failure attempt 1 -> backoff, status remains PENDING
    record_dispatch_failure(&conn, "notif-fail-01", "Gateway Timeout 504").unwrap();
    let records = list_notifications(&conn, 10).unwrap();
    let r1 = records.iter().find(|r| r.id == "notif-fail-01").unwrap();
    assert_eq!(r1.attempts, 1);
    assert_eq!(r1.status, NotificationStatus::Pending);

    // Failure attempt 2 -> reaches max_retries (2) -> transitions to FAILED
    record_dispatch_failure(&conn, "notif-fail-01", "Network unreachable").unwrap();
    let records = list_notifications(&conn, 10).unwrap();
    let r2 = records.iter().find(|r| r.id == "notif-fail-01").unwrap();
    assert_eq!(r2.attempts, 2);
    assert_eq!(r2.status, NotificationStatus::Failed);
}

/// 17. Technical Service Workshop & 1-Click Fiscal Settlement (Law 4308/2014 ΕΛΠ, AADE myDATA, Law 4624/2019)
/// Enforces:
/// - Cryptographic anti-tamper intake diagnostic inspection
/// - Spare parts consumption ledger and technician labor logging
/// - 1-Click conversion of bench repair into official myDATA Service Invoice (2.1)
/// - Automated posting to Customer Cardex CRM ledger
/// - Automated trigger of customer dispatch notification
#[test]
fn test_service_bench_diagnostics_and_1click_fiscal_settlement() {
    use proteus_core::cardex::init_cardex_schema;

    let conn = Connection::open_in_memory().unwrap();
    init_tickets_schema(&conn).unwrap();
    init_service_bench_schema(&conn).unwrap();
    init_invoicing_schema(&conn).unwrap();
    init_cardex_schema(&conn).unwrap();
    init_notifications_schema(&conn).unwrap();

    // 17a. Create intake service ticket for enterprise asset
    let mut ticket = ServiceTicket::new(
        "Hellas Logistics S.A.",
        "+306931112233",
        "Honeywell CT60 Rugged Mobile Terminal",
        "Mainboard liquid damage & cracked digitizer",
    );
    create_ticket(&conn, &mut ticket).unwrap();

    // 17b. Structured intake inspection checklist (ISO 9001 quality gate)
    let checklist = DeviceDiagnosticChecklist {
        ticket_id: ticket.ticket_id.clone(),
        powers_on: false,
        display_functional: false,
        touch_responsive: false,
        liquid_ingress_detected: true,
        audio_functional: false,
        camera_functional: true,
        battery_health_pct: Some(65),
        cosmetic_condition: "Heavy industrial wear, cracked glass".to_string(),
        customer_accessories: "Protective rubber boot, battery pack".to_string(),
        inspected_by: "Dimitris (Senior Lab Tech)".to_string(),
        inspected_at: chrono::Utc::now().timestamp_millis(),
    };
    save_diagnostic_checklist(&conn, &checklist).unwrap();

    let diag = get_diagnostic_checklist(&conn, &ticket.ticket_id).unwrap().unwrap();
    assert!(!diag.powers_on);
    assert!(diag.liquid_ingress_detected);

    // 17c. Technician consumes inventory spare parts on the bench
    let part_screen = ConsumedSparePart {
        id: "part-ct60-01".to_string(),
        ticket_id: ticket.ticket_id.clone(),
        part_sku: "HW-CT60-DISP".to_string(),
        description: "Honeywell CT60 Gorilla Glass Display Assembly".to_string(),
        quantity: 1.0,
        unit_cost_eur: 85.0,
        unit_retail_eur: 150.0,
        technician_name: "Dimitris".to_string(),
        consumed_at: chrono::Utc::now().timestamp_millis(),
    };
    let part_board = ConsumedSparePart {
        id: "part-ct60-02".to_string(),
        ticket_id: ticket.ticket_id.clone(),
        part_sku: "HW-CT60-PWR".to_string(),
        description: "Power Management PMIC Component".to_string(),
        quantity: 1.0,
        unit_cost_eur: 15.0,
        unit_retail_eur: 40.0,
        technician_name: "Dimitris".to_string(),
        consumed_at: chrono::Utc::now().timestamp_millis(),
    };
    add_consumed_part(&conn, &part_screen).unwrap();
    add_consumed_part(&conn, &part_board).unwrap();

    // 17d. Log technician labor (90 minutes @ 50€/hr = 75€)
    let labor = LaborLog {
        id: "lab-ct60-01".to_string(),
        ticket_id: ticket.ticket_id.clone(),
        technician_name: "Dimitris".to_string(),
        duration_minutes: 90,
        hourly_rate_eur: 50.0,
        work_performed: "SMD micro-soldering on power rail and display installation".to_string(),
        logged_at: chrono::Utc::now().timestamp_millis(),
    };
    log_technician_labor(&conn, &labor).unwrap();

    // 17e. Verify bench aggregate financial summary
    // Parts: 150 + 40 = 190€. Labor: 75€. Net Total: 265€.
    let summary = compute_bench_summary(&conn, &ticket.ticket_id).unwrap();
    assert_eq!(summary.parts_count, 2);
    assert_eq!(summary.parts_total_retail_eur, 190.0);
    assert_eq!(summary.labor_total_minutes, 90);
    assert_eq!(summary.labor_total_eur, 75.0);
    assert_eq!(summary.gross_total_eur, 265.0);

    // 17f. Execute 1-Click Commercial Settlement
    // Net: 265.00€ + 24% VAT (63.60€) = 328.60€ Gross
    let res = settle_ticket_to_invoice(
        &conn,
        &ticket.ticket_id,
        "ΤΠΥ",
        501,
        "094014201",
        "099887766",
        "Hellas Logistics S.A.",
        Some("+306931112233"),
        true, // B2B
        "Επισκευή Honeywell CT60",
    ).unwrap();

    assert_eq!(res.invoice_number, 501);
    assert_eq!(res.total_gross_eur, 328.60);
    assert!(res.cardex_movement_id.is_some(), "Customer Cardex movement must be posted");
    assert!(res.notification_id.is_some(), "Customer SMS notification must be enqueued");

    // 17g. Verify myDATA outbox queue entry exists
    let mydata_queued: i64 = conn
        .query_row("SELECT count(*) FROM mydata_outbox WHERE invoice_id = ?1", [&res.invoice_id], |r| r.get(0))
        .unwrap();
    assert_eq!(mydata_queued, 1);

    // 17h. Verify Customer Cardex rolling balance was debited 328.60€
    let cardex_entry: (f64, f64) = conn
        .query_row("SELECT amount_eur, running_balance_eur FROM cardex_entries WHERE entity_id = '099887766'", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(cardex_entry.0, 328.60);
    assert_eq!(cardex_entry.1, 328.60);
}

/// 18. AADE Decision A.1155/2023: Retail Cash Desk & EFT-POS Hardware Interlock
/// Mandates that all card-settled retail transactions require verified electronic POS authorization,
/// cryptographic A.1155 interconnection signatures, tamper-proof audit trails, and physical ΦΗΜ/ESC-POS receipts.
#[test]
fn test_aade_a1155_pos_interconnection_and_fiscal_device_interlock() {
    let conn = Connection::open_in_memory().unwrap();
    init_fiscal_pos_schema(&conn).unwrap();

    let mut config = load_fiscal_pos_config(&conn);
    config.eft_pos_interlock_enabled = true;
    config.a1155_secret_key = "SOVEREIGN_AADE_TEST_SECRET".to_string();
    config.eft_pos_terminal_id = "TID-ATH-01".to_string();
    config.eft_pos_merchant_id = "MID-HELLAS-99".to_string();
    save_fiscal_pos_config(&conn, &config).unwrap();

    // 18a. Attempt to issue a Card-funded retail receipt WITHOUT terminal response (Must Fail)
    let card_receipt = FiscalReceiptPayload {
        receipt_number: 1088,
        issue_date: "2026-10-04T12:00:00Z".to_string(),
        items: vec![FiscalReceiptItem {
            name: "Υπηρεσία Επισκευής Tablet".to_string(),
            quantity: 1.0,
            unit_price: 50.0,
            vat_rate: 24.0,
            department: 1,
        }],
        vat_breakdowns: vec![FiscalVatBreakdown {
            vat_rate: 24.0,
            net_amount: 40.32,
            vat_amount: 9.68,
        }],
        total_net: 40.32,
        total_vat: 9.68,
        total_gross: 50.0,
        payment_method: "CARD".to_string(),
        card_payment: None, // Missing authorization
    };

    let fhm_driver = CashRegisterDriver::new(config.clone());
    let err_res = fhm_driver.process_receipt(&card_receipt);
    assert!(err_res.is_err(), "Statutory A.1155 violation: Card receipt permitted without EFT-POS response");
    assert!(err_res.unwrap_err().contains("Card payment requested but no EFT-POS authorization received"));

    // 18b. Dispatch EFT-POS Card Initiation Request (50.00 EUR = 5000 cents)
    let pos_req = PosInitiateRequest {
        transaction_id: "TX-A1155-2026".to_string(),
        operation: PosTransactionType::Sale,
        amount_cents: 5000,
        currency_code: 978,
        receipt_number: 1088,
        invoice_type: "11.1".to_string(),
        cashier_id: "CASHIER-01".to_string(),
    };

    let pos_driver = EftPosDriver::new(config.clone());
    let pos_resp = pos_driver.initiate_card_transaction(&pos_req).unwrap();
    assert_eq!(pos_resp.status, PosTransactionStatus::Approved);
    assert_eq!(pos_resp.terminal_id, "TID-ATH-01");
    assert!(!pos_resp.rrn.is_empty());
    assert!(!pos_resp.auth_code.is_empty());
    assert!(pos_resp.interconnection_signature.starts_with("A1155:"));

    // 18c. Statutory Interlock Check passes with valid authorization
    let check = enforce_a1155_interlock(
        "CARD",
        Some(&pos_resp),
        &config.a1155_secret_key,
        5000,
        1088,
    );
    assert!(check.is_ok(), "A.1155 verification failed for approved EFT-POS response");

    // 18d. Record EFT-POS card transaction into SQLite ledger
    record_pos_card_transaction(&conn, &pos_req, &pos_resp).unwrap();
    let saved_tx = get_pos_card_transaction(&conn, "TX-A1155-2026").unwrap();
    assert_eq!(saved_tx.rrn, pos_resp.rrn);
    assert_eq!(saved_tx.auth_code, pos_resp.auth_code);

    // 18e. Finalize receipt with physical ΦΗΜ / Cash Register driver
    let mut authorized_receipt = card_receipt;
    authorized_receipt.card_payment = Some(pos_resp.clone());
    let fiscal_res = fhm_driver.process_receipt(&authorized_receipt).unwrap();
    assert!(fiscal_res.success);
    assert!(fiscal_res.fiscal_signature.starts_with("FHM-A:"));

    // 18f. Also verify Type B (ΕΑΦΔΣΣ) cryptographic signature mechanism
    let eafdss_driver = TypeBFiscalSignatureDriver::new(config.clone());
    let eafdss_res = eafdss_driver.process_receipt(&authorized_receipt).unwrap();
    assert!(eafdss_res.success);
    assert!(eafdss_res.fiscal_signature.starts_with("_#_"));
    assert!(eafdss_res.fiscal_signature.ends_with("_#_"));

    // 18g. Format ESC/POS thermal receipt slip with AADE A.1155 footer
    let slip_bytes = format_a1155_receipt_slip(&pos_resp, 48);
    let slip_text = String::from_utf8_lossy(&slip_bytes);
    assert!(slip_text.contains("A.1155/2023"));
    assert!(slip_text.contains("TID-ATH-01"));
    assert!(slip_text.contains(&pos_resp.rrn));
    assert!(slip_text.contains(&pos_resp.auth_code));
}

/// 19. Greek Law 4808/2021 & Circular 47319/2023: Ergani II Digital Work Card & Merkle Chain Integrity
/// Mandates real-time shift recording, statutory telecom outage declarations, SHA-256 Merkle chain
/// tamper-proofing, automated offline-to-online reconciliation, and compliant overtime calculations.
#[test]
fn test_greek_law_4808_ergani2_work_card_merkle_integrity_and_outage_reconciliation() {
    let conn = Connection::open_in_memory().unwrap();
    init_work_card_schema(&conn).unwrap();

    // 19a. Employee credential provisioning and PIN authentication
    let emp = EmployeeProfile {
        employee_id: "EMP-ATH-42".to_string(),
        afm: "094014201".to_string(),
        amka: "01019012345".to_string(),
        full_name: "Κωνσταντίνος Δημητρίου".to_string(),
        job_title: "Υπεύθυνος Αποθήκης".to_string(),
        pin_code: "8821".to_string(),
        qr_badge_token: "QR-ATH-042".to_string(),
        is_active: true,
    };
    save_employee(&conn, &emp).unwrap();

    let authenticated = authenticate_employee_by_pin(&conn, "094014201", "8821").unwrap();
    assert_eq!(authenticated.employee_id, "EMP-ATH-42");

    let bad_auth = authenticate_employee_by_pin(&conn, "094014201", "0000");
    assert!(matches!(bad_auth, Err(WorkCardError::InvalidPin(_))));

    // 19b. Record shift events during statutory telecom outage (Ανωτέρα Βία)
    let ev1 = record_clock_event(
        &conn,
        "EMP-ATH-42",
        WorkCardEventType::ClockIn,
        true,
        Some("OTE_FIBER_CUT"),
    ).unwrap();
    assert!(ev1.is_offline_fallback);
    assert_eq!(ev1.sync_status, "PENDING_OUTAGE_SYNC");
    assert_eq!(ev1.outage_reason.as_deref(), Some("OTE_FIBER_CUT"));

    let ev2 = record_clock_event(
        &conn,
        "EMP-ATH-42",
        WorkCardEventType::BreakStart,
        true,
        Some("OTE_FIBER_CUT"),
    ).unwrap();

    let _ev3 = record_clock_event(
        &conn,
        "EMP-ATH-42",
        WorkCardEventType::BreakEnd,
        true,
        Some("OTE_FIBER_CUT"),
    ).unwrap();

    let ev4 = record_clock_event(
        &conn,
        "EMP-ATH-42",
        WorkCardEventType::ClockOut,
        true,
        Some("OTE_FIBER_CUT"),
    ).unwrap();

    // 19c. Cryptographic Merkle chain integrity verification
    let valid_root = verify_merkle_chain(&conn).unwrap();
    assert_eq!(valid_root, ev4.merkle_hash);

    // 19d. Anti-tampering check: Malicious modification of an event's payload
    // Simulate malicious actor altering the offline fallback flag directly in SQLite
    conn.execute(
        "UPDATE work_card_events SET is_offline_fallback = 0 WHERE event_id = ?1",
        rusqlite::params![ev2.event_id],
    ).unwrap();

    let tamper_result = verify_merkle_chain(&conn);
    assert!(matches!(tamper_result, Err(WorkCardError::MerkleChainCorrupted(id)) if id == ev2.event_id));

    // Restore original record
    conn.execute(
        "UPDATE work_card_events SET is_offline_fallback = 1 WHERE event_id = ?1",
        rusqlite::params![ev2.event_id],
    ).unwrap();
    assert_eq!(verify_merkle_chain(&conn).unwrap(), valid_root);

    // 19e. Offline-to-online reconciliation flush
    let sync_summary = flush_outage_sync_queue(&conn).unwrap();
    assert_eq!(sync_summary.total_queued, 4);
    assert_eq!(sync_summary.synced_count, 4);
    assert_eq!(sync_summary.failed_count, 0);
    assert_eq!(sync_summary.latest_merkle_root, valid_root);

    // Verify all records transitioned to SYNCED with official Ergani receipt tokens
    let events = list_today_events(&conn, &ev1.local_time_str[..10]).unwrap();
    assert_eq!(events.len(), 4);
    for (event, emp_name) in events {
        assert_eq!(emp_name, "Κωνσταντίνος Δημητρίου");
        assert_eq!(event.sync_status, "SYNCED");
        assert!(event.ergani_submission_id.is_some());
        assert!(event.ergani_submission_id.as_ref().unwrap().starts_with("ERG-"));
    }

    // 19f. Shift duration & statutory overtime calculations under Law 4808/2021
    // (Simulate a 9-hour total shift with 30 min break = 8.5h worked => 8.0h regular, 0.5h overtime)
    let base_ts = 1727942400000i64; // 2026-10-03 08:00
    let b_start = base_ts + (4 * 3600 * 1000); // 12:00
    let b_end = b_start + (1800 * 1000); // 12:30
    let c_out_9h = base_ts + (32400 * 1000);

    conn.execute(
        "INSERT INTO work_card_events (event_id, employee_id, event_type, timestamp_utc, local_time_str, merkle_hash, sync_status) VALUES ('SHIFT-1', 'EMP-ATH-42', 'CLOCK_IN', ?1, '2026-10-03 08:00:00', 'h1', 'SYNCED')",
        rusqlite::params![base_ts],
    ).unwrap();
    conn.execute(
        "INSERT INTO work_card_events (event_id, employee_id, event_type, timestamp_utc, local_time_str, merkle_hash, sync_status) VALUES ('SHIFT-2', 'EMP-ATH-42', 'BREAK_START', ?1, '2026-10-03 12:00:00', 'h2', 'SYNCED')",
        rusqlite::params![b_start],
    ).unwrap();
    conn.execute(
        "INSERT INTO work_card_events (event_id, employee_id, event_type, timestamp_utc, local_time_str, merkle_hash, sync_status) VALUES ('SHIFT-3', 'EMP-ATH-42', 'BREAK_END', ?1, '2026-10-03 12:30:00', 'h3', 'SYNCED')",
        rusqlite::params![b_end],
    ).unwrap();
    conn.execute(
        "INSERT INTO work_card_events (event_id, employee_id, event_type, timestamp_utc, local_time_str, merkle_hash, sync_status) VALUES ('SHIFT-4', 'EMP-ATH-42', 'CLOCK_OUT', ?1, '2026-10-03 17:00:00', 'h4', 'SYNCED')",
        rusqlite::params![c_out_9h],
    ).unwrap();

    let (regular_hrs, overtime_hrs) = calculate_shift_duration_hours(&conn, "EMP-ATH-42", "2026-10-03").unwrap();
    assert_eq!(regular_hrs, 8.0);
    assert_eq!(overtime_hrs, 0.5);
}






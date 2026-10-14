//! Proteus Sovereign Business OS — End-to-End Omnichannel Integration Harness
//!
//! Connects every subsystem into an authentic operational pipeline:
//! 1. Master DB Boot & Sovereign Pre-flight Integrity Check
//! 2. Staff Clock-In & Merkle Chain (Ergani II / Law 4808/2021)
//! 3. Calibrated RS-232 Scale Reading & Metrology Interlock
//! 4. Touch POS Retail Checkout with EFT-POS TID & myDATA (A.1138 & A.1155)
//! 5. AADE myDATA Outbox Staging & QR-code Verification
//! 6. Federated B2B Dealer Stock Reservation with 15-min Time-Lock
//! 7. Inbound WebStore Order Ingestion via Cryptographic Relay
//! 8. Dropshipping Automated Routing & Fulfillment Dispatch Slip
//! 9. Sovereign B2G Peppol BIS Billing 3.0 / EN 16931 UBL 2.1 Invoice
//! 10. Post-Execution Database Integrity & Audit Verification

use chrono::Utc;
use rusqlite::Connection;
use std::fs;

use proteus_core::b2b_network::{
    confirm_reservation, init_b2b_reservation_tables, register_dealer, reserve_stock,
    upsert_shared_inventory_item, B2bDealer, SharedItem,
};
use proteus_core::bootstrap::bootstrap_store_database;
use proteus_core::dropship::{
    generate_fulfillment_slip, init_dropship_dispatch_tables, init_dropship_pricing_tables,
    insert_supplier, route_inbound_order_to_suppliers, upsert_dropship_item, DropshipSupplier,
};
use proteus_core::hardware::scales::{
    init_scale_metrology_schema, record_weighed_cart_item, ScaleFrameParser,
    ScaleVendorProtocol, WeighedCartItem,
};
use proteus_core::invoicing::{
    calculate_totals, create_invoice, format_mydata_qr_url, FiscalInvoice, InvoiceLine,
    InvoiceType, VatCategory,
};
use proteus_core::peppol::{
    init_peppol_tables, register_contract, register_public_buyer, PeppolBillingDocument,
    PeppolLineItem, PeppolParty, PeppolUblGenerator, PublicBuyer, PublicContract,
};
use proteus_core::preflight::run_preflight_diagnostics;
use proteus_core::ui_dsl::deploy::{
    init_inbound_tables, ingest_order_envelope, OrderRelayEnvelope, RelayOrderItem,
};
use proteus_core::work_card::{
    init_work_card_schema, record_clock_event, save_employee, EmployeeProfile,
    WorkCardEventType,
};

#[test]
fn test_end_to_end_omnichannel_business_simulation() {
    // =========================================================================
    // STEP 1: Master DB Boot & Pre-Flight Diagnostic Integrity Check
    // =========================================================================
    let mut conn = Connection::open_in_memory().expect("Failed to create in-memory SQLite store");
    proteus_core::apply_storage_tuning(&conn).expect("Storage tuning must succeed");

    // Initialize all domain database schemas
    bootstrap_store_database(&conn).expect("Bootstrap must succeed");
    init_b2b_reservation_tables(&conn).expect("B2B tables must initialize");
    init_dropship_pricing_tables(&conn).expect("Dropship pricing tables must initialize");
    init_dropship_dispatch_tables(&conn).expect("Dropship dispatch tables must initialize");
    init_peppol_tables(&conn).expect("Peppol tables must initialize");
    init_inbound_tables(&conn).expect("Inbound webstore tables must initialize");
    init_scale_metrology_schema(&conn).expect("Scale metrology tables must initialize");

    let temp_dir = std::env::temp_dir().join(format!("pilot_sim_{}", uuid::Uuid::new_v4()));
    let config_path = temp_dir.join("proteus.toml");
    let preflight_report = run_preflight_diagnostics(&conn, &config_path);

    assert!(preflight_report.overall_pass, "Pre-flight overall pass must be true");
    assert!(preflight_report.sqlite.integrity_ok, "SQLite integrity must report ok");
    assert!(config_path.exists(), "Self-healing must generate missing config");

    // =========================================================================
    // STEP 2: Staff Clock-In & Merkle Chain (Ergani II Law 4808/2021)
    // =========================================================================
    init_work_card_schema(&conn).unwrap();
    let employee = EmployeeProfile {
        employee_id: "EMP-GR-001".to_string(),
        afm: "094014201".to_string(),
        amka: "01019012345".to_string(),
        full_name: "Ιωάννης Παπαδόπουλος".to_string(),
        job_title: "Υπεύθυνος Ταμείου & Αποθήκης".to_string(),
        pin_code: "1234".to_string(),
        qr_badge_token: "QR-SEC-094014201".to_string(),
        is_active: true,
    };
    save_employee(&conn, &employee).expect("Failed to register employee");

    let clock_event = record_clock_event(
        &conn,
        &employee.employee_id,
        WorkCardEventType::ClockIn,
        false,
        None,
    )
    .expect("Failed to clock in");

    assert_eq!(clock_event.employee_id, "EMP-GR-001");
    assert!(!clock_event.merkle_hash.is_empty(), "Merkle hash must be non-empty");

    // =========================================================================
    // STEP 3: Calibrated RS-232 Scale Reading & Metrology Interlock
    // =========================================================================
    let scale_raw_frame = b"\x02ST,GS,+001.500kg\r\n";
    let parsed_weight = ScaleFrameParser::parse_frame(ScaleVendorProtocol::Dibal, scale_raw_frame)
        .expect("Valid scale frame");

    assert!(parsed_weight.is_stable, "Scale must be stable");
    assert_eq!(parsed_weight.net_weight_grams, 1500);

    let weight_kg = parsed_weight.net_weight_grams as f64 / 1000.0;
    let line_total = ((weight_kg * 20.00) * 100.0_f64).round() / 100.0_f64;
    let weighed_item = WeighedCartItem {
        description: "Βιολογικός Καφές Αιθιοπίας".to_string(),
        quantity_kg: weight_kg,
        unit_price_per_kg: 20.00,
        line_total_eur: line_total,
        net_weight_grams: parsed_weight.net_weight_grams,
        tare_weight_grams: parsed_weight.tare_weight_grams,
        is_certified_stable: true,
    };
    assert!((weighed_item.line_total_eur - 30.00).abs() < 0.01);

    let metrology_log = record_weighed_cart_item(&conn, &weighed_item, Some(1001), "Dibal")
        .expect("Failed to record scale metrology log");
    assert!(!metrology_log.tamper_hash.is_empty());

    // =========================================================================
    // STEP 4: Touch POS Retail Checkout with EFT-POS Interlock (Law A.1155/2023)
    // =========================================================================
    let invoice_id = format!("INV-{}", uuid::Uuid::new_v4());
    let line1 = InvoiceLine::new(
        1,
        weighed_item.description.clone(),
        weighed_item.quantity_kg,
        16.13,
        VatCategory::Vat24,
    );

    let line2 = InvoiceLine::new(
        2,
        "Καλώδιο USB-C 100W",
        1.0,
        10.00,
        VatCategory::Vat24,
    );

    let (net_calc, vat_calc, gross_calc) = calculate_totals(&[line1.clone(), line2.clone()]);
    assert!(gross_calc > 0.0);

    let fiscal_invoice = FiscalInvoice {
        invoice_id: invoice_id.clone(),
        series: "POS-A".to_string(),
        invoice_number: 1001,
        invoice_type: InvoiceType::RetailReceipt11_1,
        issue_date: "2026-10-14".to_string(),
        issue_time: "18:30:00".to_string(),
        issuer_afm: "094014201".to_string(),
        recipient_afm: "000000000".to_string(), // Retail end-consumer
        recipient_name: "Πελάτης Λιανικής".to_string(),
        total_net_eur: net_calc,
        total_vat_eur: vat_calc,
        total_gross_eur: gross_calc,
        mydata_mark: None,
        mydata_uid: None,
        mydata_qr_url: None,
        is_cancelled: false,
        created_at: Utc::now().timestamp(),
    };

    create_invoice(&conn, &fiscal_invoice, &[line1, line2])
        .expect("Failed to save fiscal invoice to store");

    // Verify official myDATA QR generation
    let simulated_mark = "400008899112233";
    let qr_url = format_mydata_qr_url(
        simulated_mark,
        &fiscal_invoice.issuer_afm,
        &fiscal_invoice.issue_date,
        fiscal_invoice.total_gross_eur,
    );
    assert!(qr_url.contains("https://www.aade.gr/mydata/qr?mark=400008899112233"));

    // =========================================================================
    // STEP 5: AADE myDATA Outbox Queue Verification
    // =========================================================================
    let outbox_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM mydata_outbox WHERE status = 'PENDING'",
            [],
            |r| r.get(0),
        )
        .expect("Query outbox count");
    assert_eq!(outbox_count, 1, "Exactly one invoice must be pending myDATA transmission");

    // =========================================================================
    // STEP 6: Federated B2B Dealer Network Stock Reservation
    // =========================================================================
    let dealer = B2bDealer {
        dealer_id: "DEALER-THESS-01".to_string(),
        legal_name: "Βόρεια Δίκτυα Ε.Π.Ε.".to_string(),
        afm: "999888777".to_string(),
        contact_email: "dealer@thess-networks.gr".to_string(),
        endpoint_url: Some("https://thess.proteus.internal".to_string()),
        p2b_terms_version: "2026.1".to_string(),
        status: "ACTIVE".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    register_dealer(&conn, &dealer).expect("Register B2B dealer");

    let shared_item = SharedItem {
        sku: "PROT-ROUTER-5G".to_string(),
        dealer_id: dealer.dealer_id.clone(),
        title: "Sovereign 5G Mesh Gateway".to_string(),
        wholesale_price: 150.00,
        available_quantity: 20,
        reserved_quantity: 0,
        updated_at: Utc::now().to_rfc3339(),
    };
    upsert_shared_inventory_item(&conn, &shared_item).expect("Upsert shared inventory");

    // Atomically reserve 4 units with 15-min time-lock
    let reservation = reserve_stock(
        &mut conn,
        &dealer.dealer_id,
        &shared_item.sku,
        4,
        "CART-SES-8821",
        900,
    )
    .expect("Reserve 4 units");
    assert_eq!(reservation.quantity, 4);

    // Confirm after POS settlement
    confirm_reservation(&mut conn, &reservation.reservation_id).expect("Confirm reservation");

    // =========================================================================
    // STEP 7: Inbound WebStore Order Ingestion via Cryptographic Relay
    // =========================================================================
    let item1 = RelayOrderItem {
        sku: "DROPSHIP-SOLAR-01".to_string(),
        title: "Ηλιακό Πάνελ 100W Portable".to_string(),
        quantity: 2,
        unit_price: 85.00,
        total: 170.00,
    };
    let web_order = OrderRelayEnvelope {
        order_id: format!("WEB-{}", uuid::Uuid::new_v4()),
        store_id: "STORE-ATHENS-01".to_string(),
        created_at: Utc::now().to_rfc3339(),
        customer_name: "Μαρία Γεωργίου".to_string(),
        customer_email: "maria.g@example.com".to_string(),
        customer_phone: "+30 690 1234567".to_string(),
        shipping_address: "Τσιμισκή 50, Θεσσαλονίκη".to_string(),
        customer_afm: None,
        items: vec![item1],
        subtotal: 170.00,
        shipping_cost: 0.00,
        total_amount: 170.00,
        currency: "EUR".to_string(),
        payment_method: "CARD".to_string(),
        notes: Some("Παράδοση πρωινές ώρες".to_string()),
        signature: None,
    };
    ingest_order_envelope(&mut conn, &web_order, None).expect("Ingest webstore order envelope");

    // =========================================================================
    // STEP 8: Dropshipping Automated Routing & Fulfillment Dispatch Slip
    // =========================================================================
    let supplier = DropshipSupplier {
        id: "SUPPLIER-ENERGY-01".to_string(),
        name: "SolarTech Hellas Wholesale".to_string(),
        contact_email: "orders@solartech-hellas.gr".to_string(),
        feed_url: Some("https://api.solartech-hellas.gr/catalog.json".to_string()),
        feed_format: "JSON".to_string(),
        active: true,
        created_at: Utc::now().to_rfc3339(),
    };
    insert_supplier(&conn, &supplier).expect("Register dropship supplier");

    upsert_dropship_item(
        &conn,
        "DROPSHIP-SOLAR-01",
        &supplier.id,
        "SOLAR-100W-WH",
        Some("5201234567890"),
        "Ηλιακό Πάνελ 100W Portable",
        Some("Solar"),
        50.00,
        100,
    )
    .expect("Upsert dropship catalog item");

    let dispatch_records = route_inbound_order_to_suppliers(&mut conn, &web_order)
        .expect("Route items to supplier fulfillment queue");

    assert_eq!(dispatch_records.len(), 1);
    let fulfillment_slip = generate_fulfillment_slip(&dispatch_records[0], "Proteus Central Store");
    assert!(fulfillment_slip.contains("Proteus Central Store"));
    assert!(fulfillment_slip.contains("Τσιμισκή 50, Θεσσαλονίκη"));
    assert!(fulfillment_slip.contains("SOLAR-100W-WH"));
    assert!(fulfillment_slip.contains("Ηλιακό Πάνελ 100W Portable"));

    // =========================================================================
    // STEP 9: Sovereign B2G Peppol BIS Billing 3.0 / EN 16931 UBL 2.1 Invoice
    // =========================================================================
    let buyer = PublicBuyer {
        buyer_id: "BUYER-ATHENS-MUN".to_string(),
        endpoint_id: "9934:090000001".to_string(),
        scheme_id: "9934".to_string(),
        authority_code: "1001".to_string(),
        legal_name: "Δήμος Αθηναίων".to_string(),
        afm: "090000001".to_string(),
        address_line: "Αθηνάς 63".to_string(),
        city: "Αθήνα".to_string(),
        postal_zone: "10552".to_string(),
        country_code: "GR".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    register_public_buyer(&conn, &buyer).expect("Register public sector buyer");

    let contract = PublicContract {
        contract_id: "CTR-2026-IT-004".to_string(),
        buyer_id: buyer.buyer_id.clone(),
        tender_contract_number: "ΔΗΜ-ΑΘ-8842/2026".to_string(),
        buyer_accounting_code: Some("ACC-2026-99".to_string()),
        cpv_code: Some("30200000-1".to_string()),
        valid_from: "2026-01-01".to_string(),
        valid_to: "2026-12-31".to_string(),
    };
    register_contract(&conn, &contract).expect("Register public procurement contract");

    let peppol_doc = PeppolBillingDocument {
        invoice_number: "PEP-2026-0001".to_string(),
        issue_date: "2026-10-14".to_string(),
        due_date: "2026-11-14".to_string(),
        currency: "EUR".to_string(),
        supplier: PeppolParty {
            endpoint_id: "9934:094014201".to_string(),
            scheme_id: "9934".to_string(),
            authority_code: None,
            legal_name: "Proteus Sovereign Systems".to_string(),
            afm: "094014201".to_string(),
            address_line: "Ερμού 1".to_string(),
            city: "Αθήνα".to_string(),
            postal_zone: "10563".to_string(),
            country_code: "GR".to_string(),
        },
        buyer: PeppolParty {
            endpoint_id: buyer.endpoint_id.clone(),
            scheme_id: buyer.scheme_id.clone(),
            authority_code: Some(buyer.authority_code.clone()),
            legal_name: buyer.legal_name.clone(),
            afm: buyer.afm.clone(),
            address_line: buyer.address_line.clone(),
            city: buyer.city.clone(),
            postal_zone: buyer.postal_zone.clone(),
            country_code: buyer.country_code.clone(),
        },
        contract_reference: Some(contract.tender_contract_number.clone()),
        tender_order_reference: Some("PO-ATH-9912".to_string()),
        buyer_accounting_code: contract.buyer_accounting_code.clone(),
        mydata_mark: Some("400009988776655".to_string()),
        iban: Some("GR1201100400000001234567890".to_string()),
        bic: Some("ETHNGRAA".to_string()),
        lines: vec![PeppolLineItem {
            line_id: "1".to_string(),
            name: "Τερματικό Αυτόνομης Μηχανογράφησης".to_string(),
            cpv_code: Some("30200000-1".to_string()),
            quantity: 2.0,
            unit_code: "C62".to_string(),
            unit_price_eur: 500.00,
            vat_rate_percent: 24.0,
        }],
    };

    let ubl_xml = PeppolUblGenerator::generate_invoice_xml(&peppol_doc)
        .expect("Generate Peppol UBL XML");
    assert!(ubl_xml.contains("urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0"));
    assert!(ubl_xml.contains("9934:090000001"));
    assert!(ubl_xml.contains("GR1201100400000001234567890"));
    assert!(ubl_xml.contains("400009988776655"));
    assert!(ubl_xml.contains("ΔΗΜ-ΑΘ-8842/2026"));

    // =========================================================================
    // STEP 10: Complete Database Integrity & Audit Verification
    // =========================================================================
    let integrity_status: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .expect("PRAGMA integrity check");
    assert_eq!(integrity_status, "ok", "Database must remain 100% integral and corruption-free");

    // Clean up temporary preflight directory
    let _ = fs::remove_dir_all(&temp_dir);
}

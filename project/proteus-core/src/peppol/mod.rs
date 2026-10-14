//! Sovereign B2G Electronic Invoicing & Peppol BIS Billing 3.0 Engine.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Provides European Standard EN 16931 UBL 2.1 electronic invoice generation for Greek
//! public sector procurement (ΓΓΠΣΔΔ / ΕΣΗΔΗΣ / ΚΗΜΔΗΣ) without recurring intermediary provider fees.

pub mod schemas;
pub mod ubl;

pub use schemas::{
    init_peppol_tables, query_public_buyer, register_contract, register_public_buyer,
    validate_and_format_participant_id, validate_cpv_code, validate_greek_afm, PeppolInvoiceRecord,
    PublicBuyer, PublicContract,
};
pub use ubl::{
    PeppolBillingDocument, PeppolLineItem, PeppolParty, PeppolUblGenerator,
    PEPPOL_CUSTOMIZATION_ID, PEPPOL_PROFILE_ID,
};

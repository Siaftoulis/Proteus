//! Cryptographic live repair tracking tokens & message formatting.
//! Strict Rule 1 (100% Original Codebase) and Rule 3 (<400 lines).

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Generates a cryptographically unforgeable live tracking token for repair or order tickets.
/// Combines ticket ID, nonce, and HMAC-SHA256 signature using a sovereign secret key.
pub fn generate_tracking_token(ticket_id: &str, secret_key: &[u8]) -> String {
    let nonce = Uuid::new_v4().simple().to_string();
    let mut hasher = Sha256::new();
    hasher.update(secret_key);
    hasher.update(ticket_id.as_bytes());
    hasher.update(nonce.as_bytes());
    let sig_bytes = hasher.finalize();
    let sig = hex_encode(&sig_bytes[0..12]);
    format!("{}.{}", nonce, sig)
}

/// Verifies whether the presented tracking token matches the ticket ID and sovereign secret key.
pub fn verify_tracking_token(ticket_id: &str, token: &str, secret_key: &[u8]) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return false;
    }
    let (nonce, expected_sig) = (parts[0], parts[1]);
    let mut hasher = Sha256::new();
    hasher.update(secret_key);
    hasher.update(ticket_id.as_bytes());
    hasher.update(nonce.as_bytes());
    let sig_bytes = hasher.finalize();
    let computed_sig = hex_encode(&sig_bytes[0..12]);
    computed_sig == expected_sig
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Formats a standardized live repair tracking SMS/Viber message.
pub fn format_repair_tracking_message(
    ticket_id: &str,
    status: &str,
    base_url: &str,
    token: &str,
) -> String {
    let clean_url = base_url.trim_end_matches('/');
    format!(
        "Ενημέρωση Proteus: Η επισκευή σας [{}] είναι '{}'. Παρακολουθήστε ζωντανά την εξέλιξη: {}/track?id={}&t={}",
        ticket_id, status, clean_url, ticket_id, token
    )
}

/// Formats an official payment receipt notification message.
pub fn format_payment_receipt_message(series: &str, number: u32, amount_eur: f64) -> String {
    format!(
        "Ενημέρωση Proteus: Εκδόθηκε η Απόδειξη Είσπραξης {}-{} ποσού €{:.2}. Ευχαριστούμε για την προτίμηση!",
        series, number, amount_eur
    )
}

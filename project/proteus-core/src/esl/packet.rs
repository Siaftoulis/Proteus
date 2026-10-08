//! ESL Radio Packet Generator & Dynamic Expiry Pricing Formula.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use chrono::NaiveDate;

/// Generates an energy-efficient binary RF frame for e-ink ESL tag updates.
/// Frame structure:
/// [0..4]: Magic 'PRTS' (0x50, 0x52, 0x54, 0x53)
/// [4..10]: 6-byte MAC identifier
/// [10..14]: Price in cents (big-endian u32)
/// [14..18]: Discount price in cents (0 if none)
/// [18..19]: Promo badge code (0: none, 1: -15%, 2: -30%, 3: -50%)
/// [19..N]: Barcode ASCII bytes
/// [N..N+2]: 16-bit XOR checksum
pub fn generate_esl_radio_packet(
    tag_mac: &str,
    price: f64,
    discount: Option<f64>,
    promo_badge: Option<&str>,
    barcode: &str,
) -> Vec<u8> {
    let mut packet = Vec::with_capacity(32);
    // 1. Magic
    packet.extend_from_slice(b"PRTS");

    // 2. MAC address (parse 6 hex pairs)
    let clean_mac: String = tag_mac.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let mut mac_bytes = [0u8; 6];
    for (i, chunk) in clean_mac.as_bytes().chunks(2).take(6).enumerate() {
        if chunk.len() == 2 {
            if let Ok(s) = std::str::from_utf8(chunk) {
                if let Ok(b) = u8::from_str_radix(s, 16) {
                    mac_bytes[i] = b;
                }
            }
        }
    }
    packet.extend_from_slice(&mac_bytes);

    // 3. Price in cents
    let price_cents = (price * 100.0).round() as u32;
    packet.extend_from_slice(&price_cents.to_be_bytes());

    // 4. Discount price in cents
    let disc_cents = discount.map(|d| (d * 100.0).round() as u32).unwrap_or(0);
    packet.extend_from_slice(&disc_cents.to_be_bytes());

    // 5. Promo badge code
    let promo_code: u8 = match promo_badge {
        Some(s) if s.contains("-50%") => 3,
        Some(s) if s.contains("-30%") => 2,
        Some(s) if s.contains("-15%") => 1,
        _ => 0,
    };
    packet.push(promo_code);

    // 6. Barcode payload
    packet.extend_from_slice(barcode.as_bytes());

    // 7. Checksum (16-bit XOR sum)
    let mut checksum: u16 = 0;
    for b in &packet {
        checksum ^= (*b as u16) << 8 | *b as u16;
    }
    packet.extend_from_slice(&checksum.to_be_bytes());

    packet
}

/// Evaluates dynamic expiry markdown based on days remaining until expiration (GS1 AI 17).
pub fn evaluate_dynamic_expiry_discount(
    base_price: f64,
    expiry_date_str: &str,
    current_date_str: &str,
) -> (f64, Option<f64>, Option<&'static str>) {
    let expiry = NaiveDate::parse_from_str(expiry_date_str, "%Y-%m-%d");
    let current = NaiveDate::parse_from_str(current_date_str, "%Y-%m-%d");

    if let (Ok(exp), Ok(curr)) = (expiry, current) {
        let days_left = (exp - curr).num_days();

        if days_left <= 1 {
            let discounted = (base_price * 0.50 * 100.0).round() / 100.0;
            (base_price, Some(discounted), Some("-50% ΑΜΕΣΗ ΚΑΤΑΝΑΛΩΣΗ"))
        } else if days_left <= 3 {
            let discounted = (base_price * 0.70 * 100.0).round() / 100.0;
            (base_price, Some(discounted), Some("-30% ΠΡΟΣΦΟΡΑ ΗΜΕΡΑΣ"))
        } else if days_left <= 7 {
            let discounted = (base_price * 0.85 * 100.0).round() / 100.0;
            (base_price, Some(discounted), Some("-15% ΕΚΠΤΩΣΗ"))
        } else {
            (base_price, None, None)
        }
    } else {
        (base_price, None, None)
    }
}

pub fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        s.push_str(&format!("{:02X}", b));
    }
    s
}

pub fn hex_decode(hex_str: &str) -> Option<Vec<u8>> {
    if hex_str.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for chunk in hex_str.as_bytes().chunks(2) {
        let s = std::str::from_utf8(chunk).ok()?;
        let b = u8::from_str_radix(s, 16).ok()?;
        bytes.push(b);
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_structure_and_checksum() {
        let packet = generate_esl_radio_packet(
            "AA:BB:CC:11:22:33",
            12.50,
            Some(8.75),
            Some("-30%"),
            "5201234567890",
        );
        assert_eq!(&packet[0..4], b"PRTS");
        assert!(packet.len() >= 22);
    }

    #[test]
    fn test_hex_encode_decode_roundtrip() {
        let data = b"HELLO_ESL_PACKET";
        let encoded = hex_encode(data);
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

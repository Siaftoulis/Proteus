//! Multi-Vendor Weigh Scale Protocol Engine (CAS, Dibal, Mettler-Toledo, Avery Weigh-Tronix).
//! Parses streaming RS-232 / USB serial ASCII frames from commercial retail scales
//! with auto-protocol detection, motion/stability flags, and tare/overload verification.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use serde::{Deserialize, Serialize};
use crate::hardware::types::{HardwareBusError, ScaleReading};

/// Supported industrial and retail weigh scale protocol dialects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleVendorProtocol {
    /// Mettler-Toledo MT-SICS (Standard Interface Command Set) e.g. "S S 1.250 kg\r\n".
    MettlerToledoSics,
    /// CAS PD-II / ER / SW Series Protocol e.g. "ST,GS,+ 1.250kg\r\n".
    CasStandard,
    /// Dibal POS / Wind / Series 500 Protocol e.g. "$01.250kg\r\n" or "01.250 kg S\r\n".
    Dibal,
    /// Avery Weigh-Tronix 6700/6720 Protocol e.g. "\nS+001.250kg\r".
    AveryWeighTronix,
    /// Universal auto-detection scanning frame signatures.
    AutoDetect,
}

impl ScaleVendorProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MettlerToledoSics => "METTLER_TOLEDO_SICS",
            Self::CasStandard => "CAS_STANDARD",
            Self::Dibal => "DIBAL",
            Self::AveryWeighTronix => "AVERY_WEIGH_TRONIX",
            Self::AutoDetect => "AUTO_DETECT",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "METTLER_TOLEDO_SICS" | "METTLER" | "TOLEDO" => Self::MettlerToledoSics,
            "CAS_STANDARD" | "CAS" => Self::CasStandard,
            "DIBAL" => Self::Dibal,
            "AVERY_WEIGH_TRONIX" | "AVERY" => Self::AveryWeighTronix,
            _ => Self::AutoDetect,
        }
    }

    /// Serial command bytes to request immediate weight read (polling).
    pub fn poll_command(&self) -> &'static [u8] {
        match self {
            Self::MettlerToledoSics => b"SI\r\n",
            Self::CasStandard => b"W\r",
            Self::Dibal => b"P\r\n",
            Self::AveryWeighTronix => b"W\r",
            Self::AutoDetect => b"W\r",
        }
    }

    /// Serial command bytes to zero the scale.
    pub fn zero_command(&self) -> &'static [u8] {
        match self {
            Self::MettlerToledoSics => b"Z\r\n",
            Self::CasStandard => b"Z\r",
            Self::Dibal => b"Z\r\n",
            Self::AveryWeighTronix => b"Z\r",
            Self::AutoDetect => b"Z\r",
        }
    }

    /// Serial command bytes to tare the current load.
    pub fn tare_command(&self) -> &'static [u8] {
        match self {
            Self::MettlerToledoSics => b"T\r\n",
            Self::CasStandard => b"T\r",
            Self::Dibal => b"T\r\n",
            Self::AveryWeighTronix => b"T\r",
            Self::AutoDetect => b"T\r",
        }
    }
}

/// Parses multi-vendor weigh scale byte streams.
pub struct ScaleFrameParser;

impl ScaleFrameParser {
    /// Parses a raw serial frame using the specified protocol dialect.
    pub fn parse_frame(
        protocol: ScaleVendorProtocol,
        raw_bytes: &[u8],
    ) -> Result<ScaleReading, HardwareBusError> {
        match protocol {
            ScaleVendorProtocol::MettlerToledoSics => Self::parse_sics(raw_bytes),
            ScaleVendorProtocol::CasStandard => Self::parse_cas(raw_bytes),
            ScaleVendorProtocol::Dibal => Self::parse_dibal(raw_bytes),
            ScaleVendorProtocol::AveryWeighTronix => Self::parse_avery(raw_bytes),
            ScaleVendorProtocol::AutoDetect => {
                let (_, reading) = Self::detect_and_parse(raw_bytes)?;
                Ok(reading)
            }
        }
    }

    /// Auto-detects protocol dialect by inspecting frame signatures and parses the reading.
    pub fn detect_and_parse(
        raw_bytes: &[u8],
    ) -> Result<(ScaleVendorProtocol, ScaleReading), HardwareBusError> {
        let text = String::from_utf8_lossy(raw_bytes);
        let trimmed = text.trim();

        if trimmed.starts_with("S S") || trimmed.starts_with("S D") || trimmed.starts_with("S I") {
            let r = Self::parse_sics(raw_bytes)?;
            return Ok((ScaleVendorProtocol::MettlerToledoSics, r));
        }

        if trimmed.starts_with("ST,") || trimmed.starts_with("US,") || trimmed.starts_with("OL,") {
            let r = Self::parse_cas(raw_bytes)?;
            return Ok((ScaleVendorProtocol::CasStandard, r));
        }

        if trimmed.starts_with('$') || trimmed.starts_with('P') {
            let r = Self::parse_dibal(raw_bytes)?;
            return Ok((ScaleVendorProtocol::Dibal, r));
        }

        // Try Avery Weigh-Tronix or fallback CAS
        if let Ok(r) = Self::parse_avery(raw_bytes) {
            return Ok((ScaleVendorProtocol::AveryWeighTronix, r));
        }

        if let Ok(r) = Self::parse_cas(raw_bytes) {
            return Ok((ScaleVendorProtocol::CasStandard, r));
        }

        Err(HardwareBusError::InvalidFraming)
    }

    /// Mettler-Toledo MT-SICS Parser:
    /// Examples:
    /// "S S   1.250 kg\r\n" -> Stable 1.250 kg
    /// "S D   1.250 kg\r\n" -> Dynamic / Unstable 1.250 kg
    /// "S I\r\n" -> Overload or invalid command
    /// "S +\r\n" -> Overload
    pub fn parse_sics(raw: &[u8]) -> Result<ScaleReading, HardwareBusError> {
        let text = String::from_utf8_lossy(raw);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(HardwareBusError::InvalidFraming);
        }

        if trimmed == "S +" || trimmed == "S I" {
            return Ok(ScaleReading {
                net_weight_grams: 0,
                tare_weight_grams: 0,
                unit: "kg".to_string(),
                is_stable: false,
                is_overweight: true,
            });
        }

        let is_stable = trimmed.starts_with("S S");
        let (val_grams, unit) = extract_weight_and_unit(trimmed)?;

        Ok(ScaleReading {
            net_weight_grams: val_grams,
            tare_weight_grams: 0,
            unit,
            is_stable,
            is_overweight: false,
        })
    }

    /// CAS Protocol Parser:
    /// Examples:
    /// "ST,GS,+  1.250kg\r\n" -> Stable gross 1.250 kg
    /// "US,NT,+  0.500kg\r\n" -> Unstable net 0.500 kg
    /// "OL,GS,+  0.000kg\r\n" -> Overload
    pub fn parse_cas(raw: &[u8]) -> Result<ScaleReading, HardwareBusError> {
        let text = String::from_utf8_lossy(raw);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(HardwareBusError::InvalidFraming);
        }

        let is_overweight = trimmed.starts_with("OL");
        let is_stable = trimmed.starts_with("ST") && !is_overweight;

        let (val_grams, unit) = extract_weight_and_unit(trimmed)?;

        Ok(ScaleReading {
            net_weight_grams: if is_overweight { 0 } else { val_grams },
            tare_weight_grams: 0,
            unit,
            is_stable,
            is_overweight,
        })
    }

    /// Dibal Protocol Parser:
    /// Examples:
    /// "$01.250kg\r\n" -> Stable 1.250 kg
    /// "01.250 kg S\r\n" -> Stable 1.250 kg
    /// "01.250 kg M\r\n" -> Moving (Unstable) 1.250 kg
    pub fn parse_dibal(raw: &[u8]) -> Result<ScaleReading, HardwareBusError> {
        let text = String::from_utf8_lossy(raw);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(HardwareBusError::InvalidFraming);
        }

        let is_stable = !trimmed.ends_with('M') && !trimmed.contains("UNSTABLE");
        let is_overweight = trimmed.contains("OVER") || trimmed.contains("OL");

        let (val_grams, unit) = extract_weight_and_unit(trimmed)?;

        Ok(ScaleReading {
            net_weight_grams: if is_overweight { 0 } else { val_grams },
            tare_weight_grams: 0,
            unit,
            is_stable,
            is_overweight,
        })
    }

    /// Avery Weigh-Tronix Protocol Parser:
    /// Examples:
    /// "\nS+001.250kg\r" -> Stable 1.250 kg
    /// "\nM+001.250kg\r" -> Motion / Unstable 1.250 kg
    /// "\nO+000.000kg\r" -> Overload
    pub fn parse_avery(raw: &[u8]) -> Result<ScaleReading, HardwareBusError> {
        let text = String::from_utf8_lossy(raw);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(HardwareBusError::InvalidFraming);
        }

        let is_overweight = trimmed.starts_with('O') || trimmed.contains("OVER");
        let is_stable = trimmed.starts_with('S') && !is_overweight;

        let (val_grams, unit) = extract_weight_and_unit(trimmed)?;

        Ok(ScaleReading {
            net_weight_grams: if is_overweight { 0 } else { val_grams },
            tare_weight_grams: 0,
            unit,
            is_stable,
            is_overweight,
        })
    }
}

/// Helper that extracts numeric float weight, sign, and unit ("kg" or "g"), converting to integer grams.
fn extract_weight_and_unit(text: &str) -> Result<(i64, String), HardwareBusError> {
    let lower = text.to_lowercase();
    let unit = if lower.contains("kg") {
        "kg".to_string()
    } else if lower.contains('g') {
        "g".to_string()
    } else if lower.contains("lb") {
        "lb".to_string()
    } else {
        "kg".to_string()
    };

    let sign: i64 = if text.contains('-') { -1 } else { 1 };

    let mut num_str = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num_str.push(ch);
        }
    }

    if num_str.is_empty() {
        return Err(HardwareBusError::InvalidFraming);
    }

    let val_f64: f64 = num_str.parse().map_err(|_| HardwareBusError::InvalidFraming)?;
    let multiplier = match unit.as_str() {
        "kg" => 1000.0,
        "g" => 1.0,
        "lb" => 453.592, // convert pounds to grams
        _ => 1000.0,
    };

    let grams = (val_f64 * multiplier).round() as i64 * sign;
    Ok((grams, unit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mettler_toledo_sics_parser() {
        let stable_frame = b"S S   1.250 kg\r\n";
        let reading = ScaleFrameParser::parse_frame(ScaleVendorProtocol::MettlerToledoSics, stable_frame).unwrap();
        assert_eq!(reading.net_weight_grams, 1250);
        assert!(reading.is_stable);
        assert!(!reading.is_overweight);

        let dynamic_frame = b"S D   0.450 kg\r\n";
        let reading_dyn = ScaleFrameParser::parse_frame(ScaleVendorProtocol::MettlerToledoSics, dynamic_frame).unwrap();
        assert_eq!(reading_dyn.net_weight_grams, 450);
        assert!(!reading_dyn.is_stable);

        let overload_frame = b"S +\r\n";
        let reading_ol = ScaleFrameParser::parse_frame(ScaleVendorProtocol::MettlerToledoSics, overload_frame).unwrap();
        assert!(reading_ol.is_overweight);
    }

    #[test]
    fn test_cas_protocol_parser() {
        let cas_frame = b"ST,GS,+  2.500kg\r\n";
        let reading = ScaleFrameParser::parse_frame(ScaleVendorProtocol::CasStandard, cas_frame).unwrap();
        assert_eq!(reading.net_weight_grams, 2500);
        assert!(reading.is_stable);
        assert!(!reading.is_overweight);

        let cas_unstable = b"US,GS,+  0.800kg\r\n";
        let r_unstable = ScaleFrameParser::parse_frame(ScaleVendorProtocol::CasStandard, cas_unstable).unwrap();
        assert_eq!(r_unstable.net_weight_grams, 800);
        assert!(!r_unstable.is_stable);
    }

    #[test]
    fn test_dibal_and_avery_parsers() {
        let dibal_frame = b"$01.500kg\r\n";
        let r_dibal = ScaleFrameParser::parse_frame(ScaleVendorProtocol::Dibal, dibal_frame).unwrap();
        assert_eq!(r_dibal.net_weight_grams, 1500);
        assert!(r_dibal.is_stable);

        let avery_frame = b"\nS+003.200kg\r";
        let r_avery = ScaleFrameParser::parse_frame(ScaleVendorProtocol::AveryWeighTronix, avery_frame).unwrap();
        assert_eq!(r_avery.net_weight_grams, 3200);
        assert!(r_avery.is_stable);
    }

    #[test]
    fn test_auto_detect_scale_protocols() {
        let (proto1, r1) = ScaleFrameParser::detect_and_parse(b"S S   1.100 kg\r\n").unwrap();
        assert_eq!(proto1, ScaleVendorProtocol::MettlerToledoSics);
        assert_eq!(r1.net_weight_grams, 1100);

        let (proto2, r2) = ScaleFrameParser::detect_and_parse(b"ST,GS,+  0.950kg\r\n").unwrap();
        assert_eq!(proto2, ScaleVendorProtocol::CasStandard);
        assert_eq!(r2.net_weight_grams, 950);
    }
}

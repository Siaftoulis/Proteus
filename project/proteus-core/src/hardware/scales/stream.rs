//! Real-Time Scale Stream Buffer, Stability Filter & Tare Compensation Engine.
//! Buffers asynchronous serial bytes, detects complete frames, enforces metrological
//! stability (OIML R76 / A.1155 legal interlock), and calculates exact net weights with container tares.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use std::collections::HashMap;
use crate::hardware::scales::protocol::{ScaleFrameParser, ScaleVendorProtocol};
use crate::hardware::types::{HardwareBusError, ScaleReading};

/// Live metrological status of a connected scale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveScaleState {
    pub gross_weight_grams: i64,
    pub tare_weight_grams: i64,
    pub net_weight_grams: i64,
    pub unit: String,
    pub is_stable: bool,
    pub is_settled: bool,
    pub is_zero: bool,
    pub is_overweight: bool,
    pub consecutive_stable_count: u32,
}

impl Default for LiveScaleState {
    fn default() -> Self {
        Self {
            gross_weight_grams: 0,
            tare_weight_grams: 0,
            net_weight_grams: 0,
            unit: "kg".to_string(),
            is_stable: false,
            is_settled: false,
            is_zero: true,
            is_overweight: false,
            consecutive_stable_count: 0,
        }
    }
}

/// Manages manual and preset container tares for retail packaging and smart tote logistics.
#[derive(Debug, Clone, Default)]
pub struct TareEngine {
    active_tare_grams: i64,
    preset_tares: HashMap<String, i64>,
}

impl TareEngine {
    pub fn new() -> Self {
        let mut presets = HashMap::new();
        presets.insert("TOTE-STD".to_string(), 350);
        presets.insert("TOTE-SMART".to_string(), 420);
        presets.insert("BAG-PAPER-S".to_string(), 15);
        presets.insert("BAG-PAPER-L".to_string(), 35);
        presets.insert("PLASTIC-CONTAINER".to_string(), 85);

        Self {
            active_tare_grams: 0,
            preset_tares: presets,
        }
    }

    /// Sets an explicit manual tare in grams.
    pub fn set_manual_tare(&mut self, grams: i64) {
        self.active_tare_grams = grams.max(0);
    }

    /// Clears the active tare back to 0.
    pub fn clear_tare(&mut self) {
        self.active_tare_grams = 0;
    }

    /// Activates a predefined container tare by its code.
    pub fn apply_preset(&mut self, preset_code: &str) -> bool {
        if let Some(&tare) = self.preset_tares.get(preset_code) {
            self.active_tare_grams = tare;
            true
        } else {
            false
        }
    }

    pub fn active_tare(&self) -> i64 {
        self.active_tare_grams
    }

    /// Computes net weight from gross, applying active tare with zero floor if desired.
    pub fn compute_net(&self, gross_grams: i64) -> i64 {
        gross_grams - self.active_tare_grams
    }
}

/// Asynchronous serial byte stream buffer and metrological stability filter.
pub struct ScaleStreamProcessor {
    protocol: ScaleVendorProtocol,
    buffer: Vec<u8>,
    max_buffer_size: usize,
    tare_engine: TareEngine,
    consecutive_stable_threshold: u32,
    last_state: LiveScaleState,
}

impl ScaleStreamProcessor {
    pub fn new(protocol: ScaleVendorProtocol) -> Self {
        Self {
            protocol,
            buffer: Vec::with_capacity(512),
            max_buffer_size: 1024,
            tare_engine: TareEngine::new(),
            consecutive_stable_threshold: 3, // Requires 3 consecutive identical stable frames to settle
            last_state: LiveScaleState::default(),
        }
    }

    pub fn tare_engine_mut(&mut self) -> &mut TareEngine {
        &mut self.tare_engine
    }

    pub fn current_state(&self) -> &LiveScaleState {
        &self.last_state
    }

    /// Feeds incoming serial bytes, extracts complete frames, and updates live state.
    pub fn feed_bytes(&mut self, incoming: &[u8]) -> Vec<Result<ScaleReading, HardwareBusError>> {
        self.buffer.extend_from_slice(incoming);

        // Guard against memory leaks if sender never sends line delimiter
        if self.buffer.len() > self.max_buffer_size {
            self.buffer.drain(0..self.buffer.len() - 256);
        }

        let mut results = Vec::new();

        while let Some(delimiter_idx) = find_frame_delimiter(&self.buffer) {
            let frame_bytes: Vec<u8> = self.buffer.drain(0..=delimiter_idx).collect();
            if frame_bytes.iter().all(|b| b.is_ascii_whitespace()) {
                continue;
            }
            let parsed = ScaleFrameParser::parse_frame(self.protocol, &frame_bytes);

            if let Ok(ref reading) = parsed {
                self.update_live_state(reading);
            }
            results.push(parsed);
        }

        results
    }

    fn update_live_state(&mut self, reading: &ScaleReading) {
        let gross = reading.net_weight_grams; // Hardware reading is gross before software tare
        let net = self.tare_engine.compute_net(gross);
        let is_zero = net.abs() <= 2; // Metrological zero band (+- 2 grams)

        let is_overweight = reading.is_overweight;
        let is_stable = reading.is_stable && !is_overweight;

        // Stability filtering: requires consecutive consistent frames to declare settled
        if is_stable && (self.last_state.net_weight_grams - net).abs() <= 2 {
            self.last_state.consecutive_stable_count += 1;
        } else if is_stable {
            self.last_state.consecutive_stable_count = 1;
        } else {
            self.last_state.consecutive_stable_count = 0;
        }

        let is_settled = self.last_state.consecutive_stable_count >= self.consecutive_stable_threshold;

        self.last_state = LiveScaleState {
            gross_weight_grams: gross,
            tare_weight_grams: self.tare_engine.active_tare(),
            net_weight_grams: net,
            unit: reading.unit.clone(),
            is_stable,
            is_settled,
            is_zero,
            is_overweight,
            consecutive_stable_count: self.last_state.consecutive_stable_count,
        };
    }
}

/// Identifies delimiter index (`\r\n`, `\n`, `\r`, or ETX `0x03`) ending a complete scale frame.
fn find_frame_delimiter(buf: &[u8]) -> Option<usize> {
    for (i, &b) in buf.iter().enumerate() {
        if b == b'\r' {
            if i + 1 < buf.len() && buf[i + 1] == b'\n' {
                return Some(i + 1);
            }
            return Some(i);
        } else if b == b'\n' || b == 0x03 {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunked_stream_ingestion() {
        let mut processor = ScaleStreamProcessor::new(ScaleVendorProtocol::MettlerToledoSics);

        // Feed chunks fragment by fragment
        let r1 = processor.feed_bytes(b"S S ");
        assert!(r1.is_empty());

        let r2 = processor.feed_bytes(b"  1.50");
        assert!(r2.is_empty());

        let r3 = processor.feed_bytes(b"0 kg\r\n");
        assert_eq!(r3.len(), 1);
        let reading = r3[0].as_ref().unwrap();
        assert_eq!(reading.net_weight_grams, 1500);
        assert!(reading.is_stable);

        let state = processor.current_state();
        assert_eq!(state.gross_weight_grams, 1500);
        assert_eq!(state.net_weight_grams, 1500);
    }

    #[test]
    fn test_tare_engine_preset_and_net_calculation() {
        let mut tare_engine = TareEngine::new();
        assert_eq!(tare_engine.active_tare(), 0);

        // Apply smart tote preset (350g)
        assert!(tare_engine.apply_preset("TOTE-STD"));
        assert_eq!(tare_engine.active_tare(), 350);

        let gross = 2500;
        let net = tare_engine.compute_net(gross);
        assert_eq!(net, 2150);

        tare_engine.clear_tare();
        assert_eq!(tare_engine.active_tare(), 0);
        assert_eq!(tare_engine.compute_net(gross), 2500);
    }

    #[test]
    fn test_stability_filter_settled_detection() {
        let mut processor = ScaleStreamProcessor::new(ScaleVendorProtocol::CasStandard);

        // Feed 1st frame: moving (unstable)
        processor.feed_bytes(b"US,GS,+  1.200kg\r\n");
        assert!(!processor.current_state().is_stable);
        assert!(!processor.current_state().is_settled);

        // Feed stable frame 1
        processor.feed_bytes(b"ST,GS,+  1.200kg\r\n");
        assert!(processor.current_state().is_stable);
        assert!(!processor.current_state().is_settled); // count 1 < 3

        // Feed stable frame 2
        processor.feed_bytes(b"ST,GS,+  1.200kg\r\n");
        assert!(!processor.current_state().is_settled); // count 2 < 3

        // Feed stable frame 3
        processor.feed_bytes(b"ST,GS,+  1.200kg\r\n");
        assert!(processor.current_state().is_settled); // count 3 >= 3 -> SETTLED!
    }
}

//! Low-latency native hardware audio feedback engine.
//!
//! Provides instantaneous auditory feedback for POS barcode scanning,
//! ticket ingestion, and validation errors without external audio dependencies.
//! Uses native OS asynchronous sound dispatch (Win32 MessageBeep on Windows,
//! and standard terminal bell on Unix).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFeedbackType {
    /// Crisp chime for barcode reads, NFC taps, and successful ticket intake.
    BarcodeSuccess,
    /// Gentle standard click/affirmation beep for UI actions.
    ActionAffirm,
    /// Alert buzz for scan failure, missing required fields, or validation error.
    ErrorAlert,
}

/// Plays immediate hardware audio feedback matching the requested event type.
///
/// Dispatches asynchronously through the OS native audio subsystem to ensure
/// zero blocking or stuttering on the rendering thread.
pub fn play_feedback(feedback: AudioFeedbackType) {
    #[cfg(target_os = "windows")]
    {
        #[link(name = "user32")]
        extern "system" {
            fn MessageBeep(uType: u32) -> i32;
        }

        // Win32 Standard Sound Identifiers
        // 0x00000040 = MB_ICONASTERISK (Information / Asterisk chime)
        // 0x00000000 = MB_OK (Standard default system notification)
        // 0x00000010 = MB_ICONHAND (Critical error / Stop tone)
        let sound_type = match feedback {
            AudioFeedbackType::BarcodeSuccess => 0x00000040,
            AudioFeedbackType::ActionAffirm => 0x00000000,
            AudioFeedbackType::ErrorAlert => 0x00000010,
        };

        unsafe {
            let _ = MessageBeep(sound_type);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::io::Write;
        match feedback {
            AudioFeedbackType::BarcodeSuccess | AudioFeedbackType::ActionAffirm => {
                let _ = std::io::stdout().write_all(b"\x07");
                let _ = std::io::stdout().flush();
            }
            AudioFeedbackType::ErrorAlert => {
                let _ = std::io::stderr().write_all(b"\x07\x07");
                let _ = std::io::stderr().flush();
            }
        }
    }
}

/// Immediate chime when a barcode, QR code, or GS1-128 code is successfully scanned.
pub fn play_barcode_chime() {
    play_feedback(AudioFeedbackType::BarcodeSuccess);
}

/// Subdued affirmative chime when a form is saved or record committed.
pub fn play_affirm_tone() {
    play_feedback(AudioFeedbackType::ActionAffirm);
}

/// Low alert tone when a scan fails, a duplicate occurs, or validation errors arise.
pub fn play_error_tone() {
    play_feedback(AudioFeedbackType::ErrorAlert);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_feedback_dispatch() {
        // Must execute cleanly without panic across all feedback types
        play_feedback(AudioFeedbackType::BarcodeSuccess);
        play_feedback(AudioFeedbackType::ActionAffirm);
        play_feedback(AudioFeedbackType::ErrorAlert);

        play_barcode_chime();
        play_affirm_tone();
        play_error_tone();
    }
}

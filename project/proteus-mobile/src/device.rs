//! Mobile Device Bridge & Platform Lifecycle Engine.
//!
//! Provides unified abstractions for iOS and Android hardware capabilities,
//! safe area insets (Notch, Dynamic Island, Navigation bar), haptic tactile feedback,
//! biometric auth detection, and lifecycle background state management.

use serde::{Deserialize, Serialize};

/// Target mobile runtime platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobilePlatform {
    Android,
    Ios,
    DesktopSimulator,
}

impl MobilePlatform {
    pub fn current() -> Self {
        #[cfg(target_os = "android")]
        {
            Self::Android
        }
        #[cfg(target_os = "ios")]
        {
            Self::Ios
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            Self::DesktopSimulator
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Android => "Android OS (NDK/JVM)",
            Self::Ios => "Apple iOS (UIKit/Metal)",
            Self::DesktopSimulator => "Mobile Desktop Simulator",
        }
    }
}

/// Screen safe area insets in logical points to avoid hardware cutouts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SafeAreaInsets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Default for SafeAreaInsets {
    fn default() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

impl SafeAreaInsets {
    /// Standard preset for modern iPhone with Dynamic Island / FaceID notch.
    pub fn modern_iphone() -> Self {
        Self {
            top: 47.0,
            bottom: 34.0,
            left: 0.0,
            right: 0.0,
        }
    }

    /// Standard preset for modern Android device with punch-hole camera and gesture navigation bar.
    pub fn modern_android() -> Self {
        Self {
            top: 36.0,
            bottom: 24.0,
            left: 0.0,
            right: 0.0,
        }
    }

    /// Applies safe area margins to an egui Ui frame.
    pub fn apply_to_margin(&self, base_margin: f32) -> egui::Margin {
        egui::Margin {
            top: (base_margin + self.top).round() as i8,
            bottom: (base_margin + self.bottom).round() as i8,
            left: (base_margin + self.left).round() as i8,
            right: (base_margin + self.right).round() as i8,
        }
    }
}

/// Tactile haptic feedback types for handheld barcode scanning and fiscal button presses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HapticPattern {
    LightTap,
    MediumImpact,
    HeavyImpact,
    SuccessNotification,
    WarningNotification,
    ErrorNotification,
}

/// Biometric hardware authentication capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiometricCapability {
    None,
    Fingerprint,
    TouchId,
    FaceId,
    BiometricWeak,
}

impl BiometricCapability {
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn prompt_title(&self) -> &'static str {
        match self {
            Self::FaceId => "Επαλήθευση μέσω Face ID",
            Self::TouchId | Self::Fingerprint => "Επαλήθευση μέσω Δακτυλικού Αποτυπώματος",
            Self::BiometricWeak => "Βιομετρική Επαλήθευση",
            Self::None => "Είσοδος με PIN",
        }
    }
}

/// Handheld device state and sensory telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceContext {
    pub platform: MobilePlatform,
    pub safe_area: SafeAreaInsets,
    pub battery_level: Option<f32>,
    pub is_charging: bool,
    pub is_low_power_mode: bool,
    pub biometric_type: BiometricCapability,
    pub device_model: String,
    pub app_version: String,
}

impl Default for DeviceContext {
    fn default() -> Self {
        Self {
            platform: MobilePlatform::current(),
            safe_area: SafeAreaInsets::default(),
            battery_level: Some(1.0),
            is_charging: false,
            is_low_power_mode: false,
            biometric_type: BiometricCapability::None,
            device_model: "Proteus Generic Mobile".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl DeviceContext {
    /// Triggers sensory haptic feedback vibration on supported mobile platforms.
    pub fn trigger_haptic(&self, _pattern: HapticPattern) {
        // Native NDK / iOS CoreHaptics bridge hook:
        // On simulator/desktop, performs no-op.
    }

    /// Checks if device is in a restricted power or background state.
    pub fn should_throttle_sync(&self) -> bool {
        self.is_low_power_mode || self.battery_level.map(|l| l < 0.15).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_platform_detection() {
        let platform = MobilePlatform::current();
        assert_eq!(platform, MobilePlatform::DesktopSimulator);
        assert!(platform.display_name().contains("Simulator"));
    }

    #[test]
    fn test_safe_area_insets_presets() {
        let iphone = SafeAreaInsets::modern_iphone();
        assert_eq!(iphone.top, 47.0);
        assert_eq!(iphone.bottom, 34.0);

        let android = SafeAreaInsets::modern_android();
        assert_eq!(android.top, 36.0);
        assert_eq!(android.bottom, 24.0);

        let margin = iphone.apply_to_margin(8.0);
        assert_eq!(margin.top, 55);
        assert_eq!(margin.bottom, 42);
    }

    #[test]
    fn test_biometric_capabilities() {
        assert!(!BiometricCapability::None.is_available());
        assert!(BiometricCapability::TouchId.is_available());
        assert!(BiometricCapability::FaceId.is_available());
        assert_eq!(BiometricCapability::FaceId.prompt_title(), "Επαλήθευση μέσω Face ID");
    }

    #[test]
    fn test_device_context_throttling_logic() {
        let mut ctx = DeviceContext::default();
        assert!(!ctx.should_throttle_sync());

        ctx.is_low_power_mode = true;
        assert!(ctx.should_throttle_sync());

        ctx.is_low_power_mode = false;
        ctx.battery_level = Some(0.10);
        assert!(ctx.should_throttle_sync());
    }
}

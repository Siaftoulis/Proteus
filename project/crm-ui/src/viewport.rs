//! Mobile Viewport & Runtime Touch Adaptation.
//! Provides dynamic metrics context for desktop vs mobile/tablet touch targets,
//! safe area insets, and responsive interaction scaling.

use eframe::egui::{Margin, Vec2};

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportProfile {
    pub min_touch_target: f32, // 44.0 points for Mobile/Tablet, 24.0 points for Desktop
    pub font_scale_multiplier: f32,
    pub is_touch_device: bool,
    pub safe_area_insets: Margin, // Notch & soft navigation bar insets
}

impl Default for ViewportProfile {
    fn default() -> Self {
        Self::desktop()
    }
}

impl ViewportProfile {
    pub fn desktop() -> Self {
        Self {
            min_touch_target: 24.0,
            font_scale_multiplier: 1.0,
            is_touch_device: false,
            safe_area_insets: Margin::ZERO,
        }
    }

    pub fn mobile_touch() -> Self {
        Self {
            min_touch_target: 44.0, // Apple HIG & Android Material touch target minimum
            font_scale_multiplier: 1.15,
            is_touch_device: true,
            safe_area_insets: Margin::same(12),
        }
    }

    pub fn tablet_touch() -> Self {
        Self {
            min_touch_target: 40.0,
            font_scale_multiplier: 1.10,
            is_touch_device: true,
            safe_area_insets: Margin::symmetric(16, 12),
        }
    }

    /// Enforces the minimum touch target dimensions for interactive controls.
    pub fn touch_size(&self, natural_size: Vec2) -> Vec2 {
        Vec2::new(
            natural_size.x.max(self.min_touch_target),
            natural_size.y.max(self.min_touch_target),
        )
    }

    /// Applies touch padding around interactive areas when on touch devices.
    pub fn touch_margin(&self) -> Margin {
        if self.is_touch_device {
            Margin::symmetric(6, 6)
        } else {
            Margin::symmetric(2, 2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_profiles() {
        let desk = ViewportProfile::desktop();
        assert_eq!(desk.min_touch_target, 24.0);
        assert!(!desk.is_touch_device);
        assert_eq!(desk.touch_size(Vec2::new(10.0, 10.0)), Vec2::new(24.0, 24.0));

        let mob = ViewportProfile::mobile_touch();
        assert_eq!(mob.min_touch_target, 44.0);
        assert!(mob.is_touch_device);
        assert_eq!(mob.touch_size(Vec2::new(30.0, 20.0)), Vec2::new(44.0, 44.0));
        assert_eq!(mob.touch_size(Vec2::new(100.0, 50.0)), Vec2::new(100.0, 50.0));
    }
}

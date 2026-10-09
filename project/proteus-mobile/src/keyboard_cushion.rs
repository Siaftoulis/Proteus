//! Virtual Soft Keyboard Viewport Adjuster & Dynamic Scroll Cushion (Micro-task 29.1.1).
//! Solves Problem P28: Touch screen virtual keyboard obscuring input fields
//! on mobile and tablet touch POS devices. Provides focus-driven scroll-to-visible
//! and bottom cushion expansion for uninterrupted intake workflows.

use egui::{Align, Context, Id, Response, ScrollArea, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Configuration parameters for the soft keyboard cushion engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CushionConfig {
    /// Nominal height of the on-screen soft keyboard in points when active.
    pub nominal_keyboard_height: f32,
    /// Extra margin reserved above the keyboard to keep controls clearly visible.
    pub cushion_margin: f32,
    /// Smooth interpolation animation speed (0.1 to 1.0, 1.0 = instant).
    pub animation_speed: f32,
    /// When true, centers focused controls in the visible area.
    pub auto_center: bool,
}

impl Default for CushionConfig {
    fn default() -> Self {
        Self {
            nominal_keyboard_height: 290.0,
            cushion_margin: 32.0,
            animation_speed: 0.35,
            auto_center: true,
        }
    }
}

/// Dynamic viewport adjuster and focus tracker for mobile soft keyboards.
#[derive(Debug, Clone)]
pub struct KeyboardCushion {
    pub config: CushionConfig,
    pub is_keyboard_active: bool,
    pub current_cushion_height: f32,
    pub target_cushion_height: f32,
    pub last_focused_id: Option<Id>,
}

impl Default for KeyboardCushion {
    fn default() -> Self {
        Self::new(CushionConfig::default())
    }
}

impl KeyboardCushion {
    pub fn new(config: CushionConfig) -> Self {
        Self {
            config,
            is_keyboard_active: false,
            current_cushion_height: 0.0,
            target_cushion_height: 0.0,
            last_focused_id: None,
        }
    }

    /// Updates internal animation height towards target. Returns true if animating.
    pub fn update_animation(&mut self, dt: f32) -> bool {
        let diff = self.target_cushion_height - self.current_cushion_height;
        if diff.abs() < 0.5 {
            self.current_cushion_height = self.target_cushion_height;
            false
        } else {
            let step = diff * (self.config.animation_speed * (dt * 60.0).clamp(0.5, 2.0));
            self.current_cushion_height += step;
            true
        }
    }

    /// Evaluates an input response. If focused, expands cushion and triggers scroll.
    pub fn register_input_focus(&mut self, response: &Response) {
        if response.has_focus() {
            self.is_keyboard_active = true;
            self.target_cushion_height = self.config.nominal_keyboard_height + self.config.cushion_margin;
            self.last_focused_id = Some(response.id);

            // Trigger egui automatic scroll to place control in view
            let align = if self.config.auto_center {
                Some(Align::Center)
            } else {
                Some(Align::Min)
            };
            response.scroll_to_me(align);
        }
    }

    /// Manually dismisses the soft keyboard and collapses the bottom cushion.
    pub fn dismiss_keyboard(&mut self, ctx: &Context) {
        self.is_keyboard_active = false;
        self.target_cushion_height = 0.0;
        self.last_focused_id = None;
        ctx.memory_mut(|mem| {
            mem.stop_text_input();
        });
        ctx.request_repaint();
    }

    /// Toggles virtual keyboard state for touch emulation testing on desktop.
    pub fn toggle_emulated_keyboard(&mut self) {
        if self.is_keyboard_active {
            self.is_keyboard_active = false;
            self.target_cushion_height = 0.0;
        } else {
            self.is_keyboard_active = true;
            self.target_cushion_height = self.config.nominal_keyboard_height + self.config.cushion_margin;
        }
    }

    /// Wraps a scrollable form in a managed scroll container with dynamic bottom cushion.
    pub fn show_cushioned_scroll<R>(
        &mut self,
        ui: &mut Ui,
        id_salt: &str,
        content: impl FnOnce(&mut Ui, &mut Self) -> R,
    ) -> R {
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        let is_animating = self.update_animation(dt);
        if is_animating {
            ui.ctx().request_repaint();
        }

        let cushion_h = self.current_cushion_height;

        ScrollArea::vertical()
            .id_salt(id_salt)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let result = content(ui, self);

                // Reserve bottom cushion space so lower fields can scroll above keyboard
                if cushion_h > 0.0 {
                    ui.add_space(cushion_h);
                }

                result
            })
            .inner
    }

    /// Helper to create a cushioned single-line text input with 44pt touch comfort.
    pub fn cushioned_text_edit(
        &mut self,
        ui: &mut Ui,
        text: &mut String,
        hint: &str,
    ) -> Response {
        let desired_size = Vec2::new(ui.available_width(), 44.0);
        let resp = ui.add_sized(
            desired_size,
            egui::TextEdit::singleline(text).hint_text(hint),
        );
        self.register_input_focus(&resp);
        resp
    }

    /// Helper to create a cushioned multi-line text input with focus protection.
    pub fn cushioned_multiline_edit(
        &mut self,
        ui: &mut Ui,
        text: &mut String,
        min_height: f32,
    ) -> Response {
        let desired_size = Vec2::new(ui.available_width(), min_height.max(44.0));
        let resp = ui.add_sized(
            desired_size,
            egui::TextEdit::multiline(text),
        );
        self.register_input_focus(&resp);
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_cushion_initial_state() {
        let cushion = KeyboardCushion::default();
        assert!(!cushion.is_keyboard_active);
        assert_eq!(cushion.current_cushion_height, 0.0);
        assert_eq!(cushion.target_cushion_height, 0.0);
    }

    #[test]
    fn test_toggle_emulated_keyboard() {
        let mut cushion = KeyboardCushion::default();
        cushion.toggle_emulated_keyboard();
        assert!(cushion.is_keyboard_active);
        let expected_target = cushion.config.nominal_keyboard_height + cushion.config.cushion_margin;
        assert_eq!(cushion.target_cushion_height, expected_target);

        // Toggle back off
        cushion.toggle_emulated_keyboard();
        assert!(!cushion.is_keyboard_active);
        assert_eq!(cushion.target_cushion_height, 0.0);
    }

    #[test]
    fn test_animation_progress_towards_target() {
        let mut cushion = KeyboardCushion::default();
        cushion.target_cushion_height = 300.0;

        // Step animation forward
        let animating = cushion.update_animation(0.016);
        assert!(animating);
        assert!(cushion.current_cushion_height > 0.0);
        assert!(cushion.current_cushion_height <= 300.0);

        // Run until completion
        for _ in 0..50 {
            cushion.update_animation(0.016);
        }
        assert_eq!(cushion.current_cushion_height, 300.0);
        assert!(!cushion.update_animation(0.016));
    }

    #[test]
    fn test_cushion_config_custom_values() {
        let config = CushionConfig {
            nominal_keyboard_height: 350.0,
            cushion_margin: 40.0,
            animation_speed: 0.5,
            auto_center: false,
        };
        let mut cushion = KeyboardCushion::new(config);
        cushion.toggle_emulated_keyboard();
        assert_eq!(cushion.target_cushion_height, 390.0);
    }
}

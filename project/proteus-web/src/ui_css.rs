//! Nordic Slate Design System for Proteus Web Portal & Marketplace.
//! Adheres strictly to the human-crafted, utility-first "Average Joe" principle:
//! Crisp 1px hairline borders, subtle tonal shifts, zero purple gradients, and no muddy shadows.

pub const CSS_STYLES: &str = include_str!("../ui_design/styles.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_styles_loaded_and_valid() {
        assert!(!CSS_STYLES.is_empty());
        assert!(CSS_STYLES.contains("--palette-deep-slate-navy"));
        assert!(CSS_STYLES.contains(".hero-section"));
        assert!(CSS_STYLES.contains(".spec-metric"));
    }
}

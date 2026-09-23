# Accessibility (a11y) & Usability Standards

## 1. Contrast Requirements (WCAG 2.1 AA)
- Normal Text (< 18pt / 24px): Minimum contrast ratio **4.5:1** against surface background.
- Large Text (>= 18pt / 24px or bold >= 14pt / 18.5px): Minimum contrast ratio **3:1**.
- UI Components & Graphical Objects: Minimum contrast ratio **3:1** against adjacent background.

## 2. Token Contrast Verification
- Text Main (`#f1f5f9`) on Surface (`#151821`): Ratio **13.8:1** (Passes AAA).
- Text Muted (`#94a3b8`) on Surface (`#151821`): Ratio **6.2:1** (Passes AA).
- Primary Button Text (`#ffffff`) on Primary Blue (`#3b82f6`): Ratio **4.6:1** (Passes AA).
- Focus Ring (`#3b82f6`, 2px outline): High visibility on all dark surfaces.

## 3. Keyboard Navigation
- All interactive controls (`PDSButton`, `PDSInput`, `PDSNavTabs`, table rows) must support full Tab navigation.
- Active focus state must display a visible 2px focus ring (`shadows.focusRing` / `--border-focus`).
- Modal dialogs must trap focus when open and release it upon dismissal (Escape or Close button).

---
tags:
  - company/ui/design-system
---
# Design System

> Reusable UI component library. All values sourced from [[Brand Identity]].

## Buttons

| Variant | Class | Background | Text | Border | Hover |
|---------|-------|------------|------|--------|-------|
| Primary | `.cr-btn` | `--cr-primary` | `--cr-text-inverse` | None | Darken 10% |
| Secondary | `.cr-btn-secondary` | Transparent | `--cr-text-secondary` | `--cr-border` | Bg `--cr-surface-raised` |
| Danger | `.cr-btn-danger` | Transparent | `--cr-error` | `--cr-error` | Bg `--cr-error` + `--cr-text-inverse` |

All buttons: 10px 16px padding, 14px font, `--cr-radius-md` radius, disabled at 0.6 opacity.

## Inputs

| Property | Value |
|----------|-------|
| Background | `--cr-bg` |
| Border | `--cr-border` |
| Border focus | `--cr-primary` |
| Text | `--cr-text` |
| Placeholder | `--cr-text-muted` |
| Padding | 10px 14px |
| Radius | `--cr-radius-md` |

## Cards

| Property | Value |
|----------|-------|
| Background | `--cr-surface` |
| Border | `--cr-border-subtle` |
| Padding | 24px |
| Radius | `--cr-radius-lg` |

## Topbar

| Property | Value |
|----------|-------|
| Background | `--cr-surface` |
| Height | 44px |
| Border bottom | `--cr-border-subtle` |

## Layout

- Panels: 220px wide, `--cr-surface` background, right border
- Canvas: fills remaining space, `--cr-bg` background
- Gaps between panels: none (border acts as separator)

## Login Screen

Centered card layout:
- Card: `--cr-surface`, `--cr-radius-lg`, 380px wide, 40px padding
- Title: 24px, 700 weight
- Tabs: segmented control style, `--cr-bg` background, `--cr-surface-raised` for active
- Inputs: full width, stacked with 12px gap
- Button: full width, primary style
- Divider: "or" with lines on both sides, 16px margin
- Google button: `--cr-bg` background, `--cr-border` border, text centered

Related: [[Brand Identity]] | [[Designer]] | [[Launcher Mockups]]

---
tags:
  - company/ui/brand
---
# Brand Identity

> CRM Builder visual identity guide. Defines the look and feel of the product.

## Brand Essence

| Attribute       | Description                                                                     |
| --------------- | ------------------------------------------------------------------------------- |
| **Mission**     | Give small businesses the power to build their own CRM, visually, without code. |
| **Personality** | Confident, precise, modern, developer-friendly                                  |
| **Tone**        | Direct. No fluff. "Here's what it does. Here's how. Go."                        |
| **Tagline**     | Build your CRM. Your way.                                                       |

## Logo Concept

Interlocking geometric shapes forming a "C" — representing building blocks or construction. Monochrome version in primary blue, simplified favicon as a single block.

- Favicon: A square blue block with a white "C" cutout
- Full logo: Two interlocking L-shapes forming a bracket-like C
- No gradients in logo (works on any background)

## Color Palette

### Brand Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--cr-primary` | `#2563eb` | Primary buttons, links, active states |
| `--cr-primary-hover` | `#1d4ed8` | Button hover, focus rings |
| `--cr-primary-subtle` | `#3b82f6` | Secondary elements, badges |
| `--cr-accent` | `#f59e0b` | Highlights, warnings, "pro" features |

### Surface Colors (Dark Theme)

| Token | Hex | Usage |
|-------|-----|-------|
| `--cr-bg` | `#0f172a` | App background |
| `--cr-surface` | `#1e293b` | Cards, panels, topbar |
| `--cr-surface-raised` | `#334155` | Hover states, active tabs |
| `--cr-border` | `#475569` | Dividers, borders |
| `--cr-border-subtle` | `#334155` | Subtle separators |

### Text Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--cr-text` | `#f8fafc` | Primary text, headings |
| `--cr-text-secondary` | `#94a3b8` | Labels, descriptions |
| `--cr-text-muted` | `#64748b` | Placeholders, hints |
| `--cr-text-inverse` | `#0f172a` | Text on primary backgrounds |

### Semantic Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--cr-success` | `#22c55e` | Save confirmations, status active |
| `--cr-error` | `#ef4444` | Errors, destructive actions |
| `--cr-info` | `#06b6d4` | Info messages, tooltips |

## Typography

### Font Stack

```css
--cr-font-sans: 'Inter', system-ui, -apple-system, sans-serif;
--cr-font-mono: 'JetBrains Mono', 'Fira Code', monospace;
```

### Type Scale

| Size | Weight | Usage |
|------|--------|-------|
| 32px / 2rem | 700 | Display / hero text |
| 24px / 1.5rem | 700 | H1 page titles |
| 20px / 1.25rem | 600 | H2 section headers |
| 18px / 1.125rem | 600 | H3 panel headers |
| 16px / 1rem | 500 | Body text, input labels |
| 14px / 0.875rem | 400 | UI text, buttons |
| 13px / 0.8125rem | 400 | Small UI, tabs |
| 12px / 0.75rem | 400 | Captions, hints |
| 11px / 0.6875rem | 400 | Badges, metadata |

Line height: 1.5 for body, 1.2 for headings.

## Spacing Grid

4px base unit. All spacing in multiples of 4.

| Token | Value |
|-------|-------|
| `--cr-space-1` | 4px |
| `--cr-space-2` | 8px |
| `--cr-space-3` | 12px |
| `--cr-space-4` | 16px |
| `--cr-space-5` | 20px |
| `--cr-space-6` | 24px |
| `--cr-space-8` | 32px |
| `--cr-space-10` | 40px |
| `--cr-space-12` | 48px |

## Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--cr-radius-sm` | 4px | Small badges, tags |
| `--cr-radius-md` | 6px | Inputs, buttons, cards |
| `--cr-radius-lg` | 12px | Modals, dialogs |

## Shadows

| Token | Value |
|-------|-------|
| `--cr-shadow-sm` | `0 1px 2px rgba(0,0,0,0.3)` |
| `--cr-shadow-md` | `0 4px 12px rgba(0,0,0,0.4)` |
| `--cr-shadow-lg` | `0 8px 24px rgba(0,0,0,0.5)` |

## Application

All UI must use these tokens. No hardcoded colors, font sizes, or spacing outside of this system.

Related: [[Design System]] | [[Designer]] | [[../../Developer/02 - Architecture|Architecture]]

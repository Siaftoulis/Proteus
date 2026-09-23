# Visual Design & Tool Synchronization Reference

## 1. Overview
The Proteus Design System bridges visual design tools (such as Penpot and Figma) with the deterministic codebase via standardized, machine-readable design tokens (W3C Design Tokens Community Group format).

## 2. Token Bridge Architecture
```
Visual Design (Penpot / Figma)
       │ (Export Design Tokens)
       ▼
/design-system/tokens/*.json
       │ (Central Registry Index)
       ▼
/design-system/registry.json
       │ (AI UI Engineer / Compiler)
       ▼
Target Implementation (Web HTML/CSS or Desktop egui)
```

## 3. Tooling Conventions
1. **Names Must Match Component Manifests**:
   - In Figma or Penpot, component frames must be named identically to the registered components (e.g. `PDSButton`, `PDSCard`, `PDSStatCard`).
2. **Variants Must Match Manifest Enums**:
   - Component variant properties (e.g. `variant=primary|secondary`, `size=sm|md|lg`) must map directly to manifest values without deviation.
3. **No Arbitrary Numbers**:
   - Spacing, padding, and corner radii must select from token scales (4px, 8px, 12px, 16px, 24px, 32px).
   - Hardcoded values like `37px` or custom hex values outside `colors.json` are rejected by the design compiler.

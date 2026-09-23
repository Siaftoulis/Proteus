# Proteus Deterministic Design System — AI Engineering Rules

## 1. Core Mandate
The AI UI Engineer must NEVER repeatedly reinvent visual design. The visual design is defined once through Design Tokens, Component Manifests, Layout Registries, and Page Patterns. All subsequent UI tasks are deterministic composition operations.

Instead of "Design a modern dashboard from scratch", the AI executes:
`Use DashboardLayout + PDSHeader + 4 x PDSStatCard + PDSDataTable + QuickActions.`

---

## 2. Strict AI Efficiency Rules

* **RULE A (No Redundant Components)**:
  Never invent a component when an existing registered component can satisfy the request.
* **RULE B (No Prop Invention)**:
  Never invent a prop name. Always consult the component manifest and use registered prop names.
* **RULE C (No Visual Token Invention)**:
  Never invent visual tokens. Only reference tokens defined in `design-system/tokens/`.
* **RULE D (No Arbitrary Values)**:
  Never invent arbitrary spacing (e.g. `17px`), colors (e.g. `#2b3345`), typography sizes, breakpoints, radii, or shadows.
* **RULE E (Configuration Over Code)**:
  Prefer configuration and composition over custom code generation.
* **RULE F (Pattern First)**:
  Prefer existing page patterns (`design-system/patterns/`) over designing a page from zero.
* **RULE G (Single Reusable Point)**:
  Prefer modifying one reusable component over duplicating similar code across multiple pages.
* **RULE H (Targeted Edits Only)**:
  Do not rewrite unrelated files.
* **RULE I (No Broad Refactors)**:
  Do not perform broad refactors unless explicitly commanded.
* **RULE J (Inspect Before Creating)**:
  Before creating anything new, inspect `registry.json` and determine whether an existing primitive satisfies the requirement.
* **RULE K (Closest Primitive Principle)**:
  When information is underspecified, use the closest existing system primitive instead of creating a speculative implementation.
* **RULE L (Low Reasoning & Conciseness)**:
  Do not generate long conversational explanations during implementation. Follow: Inspect → Decide → Patch → Validate.

---

## 3. Registered Primitives Quick Reference

| Category | Registered Entities |
|---|---|
| **Components** | `PDSButton`, `PDSCard`, `PDSInput`, `PDSBadge`, `PDSStatCard`, `PDSDataTable`, `PDSModal`, `PDSNavTabs`, `PDSHeader` |
| **Layouts** | `DashboardLayout`, `ApplicationShell`, `AuthLayout`, `SettingsLayout`, `MarketplaceLayout` |
| **Patterns** | `DashboardOverview`, `WorkspaceLauncher`, `MarketplaceCatalog`, `SettingsManager` |
| **Radius** | `xs` (4px), `sm` (6px), `md` (8px), `lg` (12px - standard card), `xl` (16px), `full` (9999px) |
| **Colors** | Canvas (`#0c0e14`), Surface (`#151821`), Border (`#262c3d`), Primary Accent (`#3b82f6`), Success (`#10b981`) |

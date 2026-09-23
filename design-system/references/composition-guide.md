# Deterministic AI Composition Guide

## 1. Principles
When assembling UI requests, the AI must NOT invent novel CSS, bespoke markup, or ad-hoc components. Instead, it translates requests into deterministic layout and component selections.

## 2. Decision Tree
```
User Request: "Add a new operations overview screen"
       │
       ▼
1. Query registry.json for matching Page Pattern:
   Found: "DashboardOverview" (patterns/manifests/DashboardOverview.json)
       │
       ▼
2. Inspect Pattern Manifest:
   - Layout: DashboardLayout
   - Components: PDSHeader, PDSStatCard, PDSCard, PDSDataTable
       │
       ▼
3. Map Data Payload into Component Props:
   - Stats -> 4 x PDSStatCard (title, value, trend)
   - Table -> PDSDataTable (columns, rows, emptyMessage)
       │
       ▼
4. Output Composition (Deterministic Assembly Complete)
```

## 3. Negative Examples (Anti-Patterns)
- **BAD**: Generating 120 lines of custom CSS with arbitrary margins (`margin: 17px; background: #232938;`).
- **GOOD**: Using registered tokens `spacing.lg` (16px) and `canvas.surface` (`#151821`).
- **BAD**: Creating a custom `MyCustomStatPill` component.
- **GOOD**: Using registered `PDSStatCard` or `PDSBadge`.

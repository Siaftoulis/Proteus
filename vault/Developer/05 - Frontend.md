---
tags:
  - developer/frontend
---
# Frontend

> React 18 + TypeScript + Vite 7 in Tauri v2 WebView.

## Key Files

| File | Purpose |
|------|---------|
| `src/App.tsx` | Main component (state, modes, widgets, import) |
| `src/App.css` | Catppuccin dark theme |
| `src/main.tsx` | Entry point |
| `src/nodes/` | Custom React Flow nodes |

## State (App.tsx)

| State | Type | Description |
|-------|------|-------------|
| widgets | Widget[] | Canvas widgets |
| selectedWidget | string | Active widget ID |
| mode | designer/flow | Current mode |
| gridMode | free/grid | Snap mode |
| projects | Project[] | Project list |

## Designer Mode Features

- react-rnd drag + resize
- Grid snap (20px)
- 9 widget types
- Z-index ordering
- Keyboard delete
- Drag-from-palette (HTML5 DnD)

## Flow Mode Features

- React Flow + 4 custom nodes
- Type-safe handle colors
- Delete nodes/edges

Related: [[02 - Architecture|Architecture]] | [[06 - Nodes System|Nodes]]

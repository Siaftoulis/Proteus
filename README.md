# Proteus (CRM Builder)

Welcome to **Proteus** — a next-generation CRM platform built from the ground up using **Rust and egui**. It operates as a fast, native, cross-platform desktop application.

## 🎯 Vision & Goal
The main goal of Proteus is to empower non-technical users to design and deploy custom CRM interfaces (forms, data tables, automation flows) through an intuitive visual editor. Think of it as **Figma or Penpot specifically tailored for CRM systems**. 

Our philosophy is simple: **No premature abstractions and a true offline-first approach**.

---

## 🏗️ What Has Been Built So Far

### Core Architecture
- **Offline-first Database**: Built on top of SQLite (`crm-core`), storing projects, custom records, contacts, deals, tasks, and flow graphs without requiring an active internet connection.
- **Desktop Application (`crm-ui`)**: A unified, single-window UI with a Windows 11 / Linear.app hybrid dark theme. It seamlessly switches between contexts: **Designer, Contacts, Pipeline, Studio, and Tasks**.
- **No-Dependency State Management**: Centralized application state without complex reducers or external state frameworks.

### Features & Capabilities
- **Visual Designer**: 
  - Infinite canvas with smooth pan & zoom (scroll / middle-mouse).
  - Robust Scene Graph handling hierarchical nodes (parent/child) with Z-order rendering.
  - Complete transform controls including 8-point resize handles, smart snapping, and grid modes.
  - Drag-and-drop widget palette and contextual menus.
- **CRM-Specific Widgets**: Native widgets like `InputField`, `DataTable`, `Dropdown`, and `Button` built specifically to bind to CRM entities.
- **Flow Builder**: A visual automation editor featuring trigger, action, condition, and gate nodes that can be connected via edges.
- **Built-in CRM Modules**: 
  - **Contacts**: List, search, inline edit, and notes.
  - **Pipeline (Deals)**: Kanban board with drag-and-drop between stages.
  - **Tasks**: Filterable task list by status and priority.

---

## 📝 Roadmap & Tasks Checklist

### Active Items (Current Sprint)
- [x] Designer — 8-point resize handles.
- [x] Designer — Right-click context menus.
- [ ] **Data Model Engine**: Standardize the `records` table for all entities (EAV pattern) and build the `DataEngine` inside `crm-core` (P0).
- [ ] **Data-bound Widgets**: Develop `DataBoundTable` and `DynamicForm` widgets that read/write directly to SQLite instead of using mock data (P0).
- [ ] **Flow Runtime Engine**: Implement a synchronous DAG (Directed Acyclic Graph) walker in Rust to actually execute the defined automation flows (P1).
- [ ] **Designer Mechanics**: Add keyboard shortcuts (Ctrl+C/V/D), Undo/Redo command stack, and multi-select.

### Mid-Term Goals (Next Steps)
- [ ] **Live Data Preview**: Visualize real database data within the Designer mode.
- [ ] **Advanced Flow Actions**: Add specific nodes for sending emails, firing webhooks, and nesting sub-flows.
- [ ] **Data Import/Export**: CSV/Excel import for Contacts and Deals.
- [ ] **Pipeline Upgrades**: Custom user-defined Kanban stages and forecasting.

### Long-Term Platform Goals
- [ ] **Cloud Sync & Auth**: Google OAuth integration and REST API cloud sync via the backend `license-server`.
- [ ] **WASM Plugin System**: Allow third-party extensibility.
- [ ] **Multi-Project Management**: Support for switching between different CRM workspaces seamlessly.

---

*Proteus is currently in active development. Architecture decisions prioritize maximum control, high performance, and an unmatched user experience over unnecessary complexity.*

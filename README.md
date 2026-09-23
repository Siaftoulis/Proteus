# Proteus (CRM Builder)

Welcome to **Proteus** — a next-generation CRM platform built from the ground up using **Rust and egui**. It operates as a fast, native, cross-platform desktop application.

## Vision & Goal
The main goal of Proteus is to empower non-technical users to design and deploy custom CRM interfaces (forms, data tables, automation flows) through an intuitive visual editor. Think of it as **Figma or Penpot specifically tailored for CRM systems**. 

Our philosophy is simple: **No premature abstractions and a true offline-first approach**.

---

## What Has Been Built So Far

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

## Ecosystem Architecture & 6-Role Specialization

The Proteus ecosystem operates across 4 interconnected tiers (`crm-core`, `crm-ui`, `proteus-client`, `proteus-web`) with 6 distinct roles:
1. **Customer Support (CS)**: Client onboarding, profile intake, and quote proposal management.
2. **Business Analyst (BA)**: Operational workflow blueprints, business logic, SLAs, and approval hierarchies.
3. **Data Analyst & Architect (DA)**: SQLite schema design, foreign keys, triggers, validation rules, and data migration.
4. **UI/UX Designer (PCD)**: Desktop & POS layout design, Penpot-standard styling, component tokens, and user experience.
5. **IT & Systems Specialist (PCSS)**: Enterprise LAN mesh networking, Merkle audit trails, outbox replication, and backups.
6. **Field Support Deployer (PCDS)**: On-site hardware calibration (ESC/POS thermal printer, cash drawer kick, barcode scanner) and activation handshake.

### Commercial Models & Anti-Scam Protection
- **No Direct Export**: PDS never exports unencrypted `.pr` bundles directly; all delivery runs through `proteus-web`.
- **Bare Minimum Floor Engine**: Algorithmic floor pricing based on project complexity prevents under-the-table evasion.
- **Project Slot Board**: Small businesses with constrained budgets post their project as an open 6-slot board with automated escrow allocation, enabling community freelancers to accept offers or bid competitively.
- **Hardware-Locked DRM**: Packages compile strictly bound to the store's unique Machine ID with SHA-256 certificate validation.

---

## Roadmap & Tasks Checklist

### Active Items (Current Sprint)
- [x] Designer — 8-point resize handles & context menus.
- [x] Client — Clean role-driven workspace navigation (`SHOP`, `BA`, `DA`, `PCSS`, `PCDS`, `HQ`).
- [x] Web — Professional company services, workshops, and talent marketplace portal.
- [x] Core — RBAC separation of Business Analyst (`BA`) and Data Analyst (`DA`).
- [ ] **Slot Board & Bidding Engine (`proteus-web`)**: Interactive project slot board UI with dynamic escrow splits and instant accept offers (P0).
- [ ] **Complexity Floor Calculator (`crm-core`)**: Automated formula calculating minimum project value based on screen, entity, trigger, and hardware counts (P0).
- [ ] **Data Model Engine**: Standardize the `records` table for all entities (EAV pattern) and build the `DataEngine` inside `crm-core` (P0).
- [ ] **Data-bound Widgets**: Develop `DataBoundTable` and `DynamicForm` widgets that read/write directly to SQLite instead of using mock data (P0).

### Mid-Term Goals (Next Steps)
- [ ] **Live Data Preview**: Visualize real database data within the Designer mode.
- [ ] **Flow Runtime Engine**: Implement a synchronous DAG walker in Rust to execute defined automation flows.
- [ ] **Hardware Calibration Suite**: Interactive visual tester in PCDS for ESC/POS baud rate, cut type, and drawer pin.

---

*Proteus is currently in active development. Architecture decisions prioritize maximum control, high performance, and an unmatched user experience over unnecessary complexity.*

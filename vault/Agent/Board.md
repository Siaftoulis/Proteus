---
kanban-plugin: basic
tags:
  - agent/kanban
---
# Project Board

## Sprint 1 (Safety Net) -- Done (July 4)

- [x] Tests for existing alpha code (crm-core + React) -- [[../Company/Engineering/Developers|Developers]]
- [x] Error handling: replace `unwrap()` with `Result` -- [[../Company/Engineering/Developers|Developers]]
- [x] Structured logging (tracing) -- [[../Company/Engineering/Developers|Developers]]
- [x] Dependency version pinning -- [[../Company/Engineering/Developers|Developers]]
- [x] TypeScript strict mode -- [[../Company/Engineering/Developers|Developers]]
- [x] CI/CD pipeline (GitHub Actions) -- [[../Company/Engineering/DevOps|DevOps]]
- [x] Toast notifications for command errors (frontend) -- [[../Company/Engineering/Developers|Developers]]
- [x] Input validation (project name, forms) -- [[../Company/Engineering/Developers|Developers]]

## Sprint 2 (Launcher) -- Done (July 4)

- [x] Auth server crate (Axum, port 3001, 6 endpoints)
- [x] Email/password register + login (argon2)
- [x] JWT access tokens (1h) + refresh tokens (30d, rotated)
- [x] Token verification endpoint
- [x] App distribution stubs (latest-version + download)
- [x] 6 integration tests
- [x] LoginScreen React component
- [x] Tauri token store commands
- [x] Auth gate in App.tsx
- [x] User badge + logout button in topbar
- [x] Google OAuth: backend + Tauri command + LoginScreen button
- [x] Brand identity + Design System + launcher mockups
- [x] CSS design tokens
- [x] License integration (user_licenses table, check on login, badge)

## Sprint 3 (Sync & Launch) -- Complete (July 5)

- [x] Custom SQLite encryption (XChaCha20-Poly1305 + argon2id KDF) -- 8 tests
- [x] Export/import system (.crmb format) -- 4 tests
- [x] Roles and permissions (role field, editor default)
- [x] P2P sync (HTTP server, pull/push, merge, last-write-wins) -- 5 tests
- [x] Health endpoints + backup script + monitoring docs
- [x] 39 crm-core + 6 auth-server + 6 license-server + 11 frontend = **62 tests, all passing, zero warnings**
- [x] Full 6-step review chain completed (10 QA BLOCKs → B1-B3 fixed, B4-B10 clean)
- [x] PM review: "Infrastructure solid, core CRM features placeholder"
- [x] CEO verdict: **No-Go for public beta. Conditional Go in 4-6 weeks**
- [x] Argon2id KDF upgrade (SHA-256 → argon2 hash_password_into)

## Sprint 4 (Core CRM) — Complete (Sept 10)

### CEO's 4 Conditions for Beta

- [x] **4.1: Kill auth gating** — App works fully offline without login
- [x] **4.2: Data-bound table widget** — Editable records, not mockups
- [x] **4.3: Execute at least one simple flow** — e.g. "on button click, create record"
- [x] **4.4: Frontend tests fixed and passing**

### Additional Sprint 4 Goals

- [x] Data model engine (entity definitions, record CRUD in Rust)
- [x] Widget property editor (table columns, form field mapping)
- [x] Project management UI (rename, list, create)
- [x] Undo/redo for designer canvas
- [x] Offline-first mode (no server required for core features)
- [x] Keyboard shortcuts (Undo/Redo, Duplicate, Copy/Paste, Deselect, Nudge)
- [x] CSV Export & Import for Contacts and Pipeline Deals
- [x] SQLite Live Sync across Contacts, Deals, and Table Widgets

## Sprint 5: Pre-Enlistment Windows MVP (Sep 14 – Oct 31, 2026) — ACTIVE

> **Goal**: Single autonomous Windows binary (`.exe`) for Service & Intake Tracking with local SQLite and ESC/POS thermal receipt printing.  
> **Reference**: [[../Developer/14 - Proteus BOS Blueprint|14 - Proteus BOS Blueprint]] | [[../Developer/15 - Master Problem Audit & Architectural Solutions|15 - Master Problem Audit (P1-P21)]]

### 5.1 Storage & Schema Layer (Atomic Step 1)
- [x] Ensure SQLite path defaults to standard `%APPDATA%\Proteus\data\store.db` (Zero-privilege)
- [x] Implement `service_tickets` table DDL with UUIDv7 IDs & LWW `updated_at`
- [x] Implement `system_events` table for audit log and sync outbox
- [x] Create indexes on `current_status`, `customer_phone`, `updated_at`

### 5.2 Service & Intake UI Views in egui (Atomic Step 2)
- [x] **Screen 1: New Intake Form (Νέα Παραλαβή)**
  - [x] Customer Name & Phone inputs with validation
  - [x] Device Model, Serial number, Reported fault description
  - [x] Estimated cost input & auto-incrementing Ticket Number
- [x] **Screen 2: Kanban Pipeline (Ροή Επισκευών)**
  - [x] 6 visual lanes: `received`, `in_progress`, `waiting_parts`, `ready`, `delivered`, `cancelled`
  - [x] Card movement between stages & quick search by phone/ticket
- [x] **Screen 3: Customer & Ticket Card (Καρτέλα Επισκευής)**
  - [x] Full ticket details view with technician notes
  - [x] Status transition timestamps & delivery date

### 5.3 Hardware ESC/POS Thermal Printing (Atomic Step 3)
- [x] Direct USB / Raw printing integration for 58mm/80mm thermal receipt printers
- [x] Formatted Intake Ticket layout: Shop Header, Ticket #, Date, Customer info, Device, Fault, Barcode/QR
- [x] "Print Ticket" trigger on intake submission

### 5.4 Declarative `.pr` Package Architecture (Atomic Step 4)
- [ ] Define `.pr` bundle format specification (zstd archive / MessagePack)
- [ ] Additive-only schema migration runner (`CREATE TABLE`, `ALTER TABLE ADD COLUMN`, rejecting `DROP`)
- [ ] Automatic `.bak` SQLite snapshot before any package import

### 5.5 Standalone Packaging & Pilot Testing (Atomic Step 5)
- [x] Release build profile (`Proteus.exe`) with stripped symbols & LTO
- [x] Zero-privilege execution test (works without admin rights)
- [ ] Pilot testing and feedback collection with 2–3 local repair shops

### 5.6 Role-Based Access Control, Event Audit Trail & Appointments Subsystem
- [x] Bespoke RBAC engine in `crm-core::roles` (CEO, Customer Service, Technician, Sales, Developer)
- [x] Dynamic TopBar role switcher & navigation tab permission filtering in `proteus-client`
- [x] Event-sourced Audit Trail in `crm-core::audit` with SQLite `audit_logs` table & monotonic UUIDv7
- [x] Visual Activity Timeline (`AuditLogView`) with role filtering, search, badges, relative timestamps & JSON diff viewer
- [x] Customer Service Appointments subsystem (`AppointmentsView`) with scheduling, validation & automated audit logging
- [x] Comprehensive documentation in `vault/Developer/19 - Role-Based Access Control & Event Audit Architecture.md`
- [x] 121/121 workspace unit tests passing (100% green)

### 5.7 The Anti-SAP Disruptor: Hierarchical Multi-Store & Delegated Access Architecture
- [x] Strategic Anti-SAP Manifesto formulated in `vault/Developer/20 - The Anti-SAP Manifesto & Hierarchical Multi-Store Architecture.md`
- [x] 3-tier governance model defined (Enterprise Owner HQ -> Store Directors -> Department Operators: Warehouse, Mobile/Tech, Books/Stationery, POS, Service)
- [x] Multi-tenant DDL specification (`enterprises`, `enterprise_stores`, `store_departments`, `enterprise_users`, `tamper_proof_audit_backlog`)
- [x] Merkle hash-chain specification for cryptographic, unhackable transaction tracking
- [x] One-click zero-DevOps cloud relay & hot-snapshot strategy
- [x] Decentralized 10-Tier partner economy model for local IT integrators and jobs creation

### 5.8 Proteus Ecosystem Master Brief & May 2027 Roadmap Adoption
- [x] Master Project Brief established in `vault/Developer/21 - Proteus Ecosystem Master Brief.md` and `vault/Company/Project Brief.md`
- [x] Operational model defined: Headless Platform / Platform-as-a-Protocol (founder provides software & infra, partners do client implementations)
- [x] Rust Core multi-database architecture specified (PostgreSQL & MySQL direct connectors alongside SQLite local caching)
- [x] Multi-platform compilation strategy specified (static library `.dll`, `.dylib`, C-bindings/NDK for iOS/Android)
- [x] Certification curriculum defined: PCD (Designer), PCSS (Systems & DB), PCDS (Deployer & Support) with 79€ voucher / 39€ badge
- [x] Strategic CEO Roadmap updated in `project/ceo_roadmap.md` targeting May 2027
- [ ] Direct PostgreSQL driver integration in `crm-core`
- [ ] Direct MySQL driver integration in `crm-core`
- [ ] Sandboxed 1-click migration runner with automated `.bak` rollback snapshot

## Future (Phase 2 - Post-Military Commercial Launch)
- [ ] Merchant of Record setup (Lemon Squeezy / Paddle)
- [ ] Cloud Relay Tunnel (WebRTC / STUN-TURN) for remote mobile access
- [ ] Mobile Companion App (Native Shell for iOS / Android with `sync_outbox`)
- [ ] Local LAN mDNS auto-discovery & QR pairing
- [ ] Zero-knowledge automated encrypted cloud backups (Cloudflare R2 / S3)
- [ ] Multi-seat licensing & Self-service device management portal
- [ ] Single-Member IKE (Μονοπρόσωπη ΙΚΕ) formation post-revenue (>2,000€)


- [ ] Cloud hosting management console
- [ ] CRM extraction (standalone app build)
- [ ] iOS sideloading research
- [ ] Mobile app support (iOS + Android)
- [ ] Automatic updates via launcher
- [ ] Template marketplace
- [ ] Multi-user collaboration/real-time sync
- [ ] i18n / localization

## Done (Alpha)

- [x] Designer Mode (react-rnd, grid, z-index, palette)
- [x] Flow Mode (React Flow, 4 custom nodes)
- [x] Import System (CSV, XLSX, SQLite)
- [x] License Server
- [x] License module (crm-core)
- [x] Vault with visual architecture
- [x] Company agent structure (9 agents with personalities)
- [x] Business model + pricing
- [x] Launcher architecture decided
- [x] Spin-up analysis completed (all 8 agents)


%% kanban:settings
```
{"kanban-plugin":"basic","show-checkboxes":true}
```
%%

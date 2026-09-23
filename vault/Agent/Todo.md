---
tags:
  - agent/todo
---
# Todo

> [[Board|Kanban Board]] | [[Project Status|Status]]

## Sprint 5: Pre-Enlistment Windows MVP (Sep 14 – Oct 31, 2026) — ACTIVE

### 5.1 Storage & Schema Layer
- [x] Direct SQLite path to `%APPDATA%\Proteus\data\store.db` (Zero-privilege)
- [x] DDL creation for `service_tickets` (UUIDv7, monotonic timestamps, 6 check statuses)
- [x] DDL creation for `system_events` (local event log & sync outbox)
- [x] Composite indexes on `current_status`, `customer_phone`, `updated_at`

### 5.2 Service & Intake UI Views (egui)
- [x] **Screen 1: New Intake Form (Νέα Παραλαβή)**
  - [x] Customer Name & Phone inputs with validation
  - [x] Device Model, Serial number, Reported fault text areas
  - [x] Estimated cost input & auto-incrementing Ticket Number
- [x] **Screen 2: Kanban Pipeline (Ροή Επισκευών)**
  - [x] 6 visual lanes: `received`, `in_progress`, `waiting_parts`, `ready`, `delivered`, `cancelled`
  - [x] Card movement between stages & quick search by phone/ticket
- [x] **Screen 3: Customer & Ticket Card (Καρτέλα Επισκευής)**
  - [x] Full ticket details view with technician notes
  - [x] Status transition timestamps & delivery date

### 5.3 Hardware ESC/POS Thermal Printing
- [x] Direct USB / Raw printing integration for 58mm/80mm thermal receipt printers
- [x] Formatted Intake Ticket layout: Shop Header, Ticket #, Date, Customer info, Device, Fault, Barcode/QR
- [x] "Print Ticket" trigger on intake submission

### 5.4 Declarative `.pr` Package Architecture
- [x] Define `.pr` bundle format specification (SHA-256 sealed container, manifest, DDL, views, flows)
- [x] Additive-only schema migration runner (`CREATE TABLE`, `ALTER TABLE ADD COLUMN`, rejecting `DROP`)
- [x] Automatic `.bak` SQLite snapshot before any package import or migration

### 5.5 Distribution & Pilot Validation
- [x] Release build profile (`Proteus.exe`) with stripped symbols & LTO
- [x] Zero-privilege execution test (works without admin rights)
- [ ] Pilot testing and feedback collection with 2–3 local repair shops

## Sprint 6: PCDA Pipeline, Multi-Store Governance & Outbox Worker (Complete)
- [x] Automatic Schema Inference (`SchemaInferer`) from raw JSON/CSV
- [x] 1-Click LAN Hot-Mount of inferred tables into `.pr` packages
- [x] Anti-SAP Multi-Tenant & Multi-Store Hierarchy (`enterprises`, `stores`, `departments`, `users`)
- [x] Delegated Store Director employee provisioning with seat quota enforcement
- [x] Cryptographic Merkle hash-chain (`tamper_proof_audit_backlog`) for audit logs
- [x] Resilient Background Outbox Worker with Full Jitter Exponential Backoff & Circuit Breaker
- [x] Two-Stage Authentication & Dynamic Purchased PR Package Mounting

## Sprint 7: Next-Gen UI/UX Engine, VRR, Custom Theming & Flow Runtime (ACTIVE)
- [x] Flow Condition Engine & DAG Walker Upgrades (`ConditionOp`, true/false branching, payload merge)
- [x] Flow Builder Condition & Notification Node Palette (`Condition (IF)`, `Notification`)
- [x] Play Mode Runtime Condition Evaluation against active form state
- [x] Adaptive Variable Refresh Rate (VRR) Engine (`Reactive` 0% idle, 30/60/120/144Hz throttling)
- [x] Dynamic Dual-Mode Theme Engine & System Theme Synchronization (`System`, `Dark`, `Light`, 6 Luxury Accents)
- [x] Hit-Test First Context Dispatcher & Enhanced Canvas Selection (Auto-select on right click, multi-selection actions)
- [x] Web Marketplace & Portal Hub (`proteus-web`): Modular luxury dark UI, verified .pr package showcase, 4 certification tracks + master bundle, 10-tier partner commission calculator, SLA escrow simulator
- [x] Native Desktop Template Explorer (`proteus-client`): Verified .pr package browser with 1-click DDL ingestion and direct Web Marketplace Hub integration
- [ ] Micro-connection wiring & Polish across remaining modules

## Phase 2: Military Service Period (Nov 2026 – May 2027)
- [ ] 2–3 shop pilot testing & bugfixing during leaves
- [ ] Zero monthly expenses maintenance

## Phase 3: Commercial Launch (May 2027)
- [ ] Lemon Squeezy / Paddle Merchant of Record (MoR) setup
- [ ] Core business license (7.99€/mo) & seat tier checkout
- [ ] Local LAN mDNS auto-discovery & QR pairing
- [ ] Mobile Companion App with `sync_outbox`
- [ ] Zero-knowledge cloud snapshot backup (Cloudflare R2 / S3)

## Phase 4: Company Formation (Post-Revenue >2,000€)
- [ ] Electronic establishment of Single-Member IKE (gov.gr)

Related: [[Board|Kanban Board]] | [[Decisions|Decisions]] | [[../Developer/14 - Proteus BOS Blueprint|Proteus BOS Blueprint]] | [[../Developer/15 - Master Problem Audit & Architectural Solutions|15 - Master Problem Audit (P1-P28)]]

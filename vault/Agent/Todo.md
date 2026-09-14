---
tags:
  - agent/todo
---
# Todo

> [[Board|Kanban Board]] | [[Project Status|Status]]

## Sprint 5: Pre-Enlistment Windows MVP (Sep 14 – Oct 31, 2026) — ACTIVE

### 5.1 Storage & Schema Layer
- [ ] Direct SQLite path to `%APPDATA%\Proteus\data\store.db` (Zero-privilege)
- [ ] DDL creation for `service_tickets` (UUIDv7, monotonic timestamps, 6 check statuses)
- [ ] DDL creation for `system_events` (local event log & sync outbox)
- [ ] Composite indexes on `current_status`, `customer_phone`, `updated_at`

### 5.2 Service & Intake UI Views (egui)
- [ ] **Screen 1: New Intake Form (Νέα Παραλαβή)**
  - [ ] Customer Name & Phone inputs with validation
  - [ ] Device Model, Serial number, Reported fault text areas
  - [ ] Estimated cost input & auto-incrementing Ticket Number
- [ ] **Screen 2: Kanban Pipeline (Ροή Επισκευών)**
  - [ ] 6 visual lanes: `received`, `in_progress`, `waiting_parts`, `ready`, `delivered`, `cancelled`
  - [ ] Card movement between stages & quick search by phone/ticket
- [ ] **Screen 3: Customer & Ticket Card (Καρτέλα Επισκευής)**
  - [ ] Full ticket details view with technician notes
  - [ ] Status transition timestamps & delivery date

### 5.3 Hardware ESC/POS Thermal Printing
- [ ] Direct USB / Raw printing integration for 58mm/80mm thermal receipt printers
- [ ] Formatted Intake Ticket layout: Shop Header, Ticket #, Date, Customer info, Device, Fault, Barcode/QR
- [ ] "Print Ticket" trigger on intake submission

### 5.4 Declarative `.pr` Package Architecture
- [ ] Define `.pr` bundle format specification (zstd archive / MessagePack)
- [ ] Additive-only schema migration runner (`CREATE TABLE`, `ALTER TABLE ADD COLUMN`, rejecting `DROP`)
- [ ] Automatic `.bak` SQLite snapshot before any package import

### 5.5 Distribution & Pilot Validation
- [ ] Release build profile (`Proteus.exe`) with stripped symbols & LTO
- [ ] Zero-privilege execution test (works without admin rights)
- [ ] Pilot testing and feedback collection with 2–3 local repair shops

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

Related: [[Board|Kanban Board]] | [[Decisions|Decisions]] | [[../Developer/14 - Proteus BOS Blueprint|Proteus BOS Blueprint]] | [[../Developer/15 - Master Problem Audit & Architectural Solutions|15 - Master Problem Audit (P1-P21)]]

---
tags:
  - agent/todo
---
# Todo

> [[Board|Kanban Board]] | [[Project Status|Status]]

## Sprint 5: Pre-Enlistment Windows MVP (Sep 14 – Oct 31, 2026) (Complete)

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

## Sprint 7: Next-Gen UI/UX Engine, VRR, Custom Theming & Flow Runtime (Complete)
- [x] Flow Condition Engine & DAG Walker Upgrades (`ConditionOp`, true/false branching, payload merge)
- [x] Flow Builder Condition & Notification Node Palette (`Condition (IF)`, `Notification`)
- [x] Play Mode Runtime Condition Evaluation against active form state
- [x] Adaptive Variable Refresh Rate (VRR) Engine (`Reactive` 0% idle, 30/60/120/144Hz throttling)
- [x] Dynamic Dual-Mode Theme Engine & System Theme Synchronization (`System`, `Dark`, `Light`, 6 Luxury Accents)
- [x] Hit-Test First Context Dispatcher & Enhanced Canvas Selection (Auto-select on right click, multi-selection actions)
- [x] Web Marketplace & Portal Hub (`proteus-web`): Modular luxury dark UI, verified .pr package showcase, 4 certification tracks + master bundle, 10-tier partner commission calculator, SLA escrow simulator
- [x] Native Desktop Template Explorer (`proteus-client`): Verified .pr package browser with 1-click DDL ingestion and direct Web Marketplace Hub integration
- [x] Micro-connection wiring & Polish across remaining modules

## Sprint 8: Universal Database Harmony, SMLM & Frontline Retail Profiles (Complete)
- [x] **8.1 SMLM Content & Semantic Profiler (`proteus-core`)**
  - [x] Content-driven data profiler (AFM modulo 11, E.164 phone, IBAN, GS1/EAN-13, IMO, VIN)
  - [x] Multi-lingual semantic ontology dictionary (Greek, Greeklish, English, ERP acronyms)
  - [x] Automated column intent classifier and confidence scoring
- [x] **8.2 Entry-by-Entry Reconciler & Conflict Resolver (`proteus-core`)**
  - [x] Natural composite key matching across disparate database schemas
  - [x] LWW (Last-Write-Wins) timestamp conflict resolver and schema delta extractor
  - [x] Automated schema bridge generation
- [x] **8.3 Federation Bridge Node Block (`proteus-design-studio`)**
  - [x] `FlowNodeKind::FederationBridge` visual node in Flow Builder
  - [x] Zero-touch bidirectional synchronization between partner database and local SQLite
- [x] **8.4 Hardware Stores & Chaos Retail Profile (`proteus-client` / `proteus-design-studio`)**
  - [x] Touch Quick-Pills for non-barcoded bulk items (screws, nails, bulk cable by meter)
  - [x] Dual unit-of-measure conversion (box, pack, piece, kg, meter)
  - [x] Contractor Job-Site Sub-ledger (Καρτέλα Μάστορα ανά Έργο/Οικοδομή)
  - [x] Real SQLite sub-ledger persistence and balance tracking (Rule 5)

## Sprint 9: Bespoke Brief Engine, Domain Gateway & Managed Hosting Architecture (Complete)
- [x] **9.1 Zero Presets & Bespoke Brief Engine (`proteus-web`)**
  - [x] Eradicated generic templates in favor of client custom business brief submission
  - [x] 7-Slot Board (BA, DA, PCD-App, PCD-Web, PCSS, PCDS, CS) matching with In-Platform Escrow
  - [x] Split UI/UX certification tracks: PCD-App (79€), PCD-Web (79€), Dual Bundle (129€), Master Bundle (249€)
- [x] **9.2 Domain Reseller & Registrar Gateway (`proteus-web`)**
  - [x] Domain search & real-time availability check (.gr, .com, .eu, .shop)
  - [x] Automated DNS zone generation (A record to Proteus Edge, CNAME, Wildcard SSL tokens)
  - [x] SQLite persistence in `client_domains` table
- [x] **9.3 Managed Cloud Hosting & Extra DB Provisioning (`proteus-web`)**
  - [x] Managed Cloud Starter (14.99€/mo) and Business & E-Shop (29.99€/mo) vs Self-Hosting (0€/mo)
  - [x] Web customer accounts (`web_customers`), online orders (`web_orders`), bookings (`appointments_db`)
  - [x] SQLite persistence in `client_hosting_subscriptions` table
- [x] **9.4 Studio Canvas Track Switcher (`proteus-design-studio`)**
  - [x] Track toggle: `PCD-App (Software & POS)` vs `PCD-Web (Storefront & Mobile)`
  - [x] Bespoke workspace loader (zero dummy / template labels)
- [x] **9.5 Bespoke Brief Ingestion & Auto-Scaffold Engine (`proteus-core` / `proteus-design-studio`)**
  - [x] Core brief storage & schema in SQLite `client_briefs` table (`proteus_core::brief`)
  - [x] Canonical brief re-export and Web-to-SQLite submission persistence (`proteus-web`)
  - [x] Bespoke Brief Ingestion Modal in PDS (`components::brief_modal`)
  - [x] 1-Click Canvas Scaffolding generating authentic artboards, headers, forms, POS tables, and action buttons
- [x] **9.6 ESC/POS Thermal Receipt & Hardware Canvas Viewport (`proteus-design-studio`)**
  - [x] Authentic `Thermal80mm` (576×780) & `Thermal58mm` (384×620) presets and responsive viewport profiles
  - [x] Visual paper dispenser feeder slot, character guide margins, and serrated tear-off cut line overlay
  - [x] Track selector third mode: `🖨 ESC/POS (Hardware)` in device bar & Welcome Hub
  - [x] Dedicated `🖨 ESC/POS` Inspector tab with live monospace column overflow validator and Win32 raw print spooler controls
  - [x] Monospace receipt artboard generator (`scene::thermal_preset`) with headers, line items, totals, and hardware test triggers
- [x] **9.7 Interactive Button-to-Flow Wiring (Wiring Canvas)**
  - [x] Accurate `node_world_rect` world-space computation accumulating parent frame offsets and paddings
  - [x] Canvas visual wiring overlay (`wiring_overlay.rs`) rendering cubic bezier connection wires between buttons, target screens, and database entities
  - [x] Glowing connection ports, target pin arrowheads, and action midpoint badges (`⚡ ➔ Screen`, `💾 Submit 'entity'`)
  - [x] Enhanced Prototype / Flow Inspector tab with quick target screen 1-click wiring, active wire status badges, and wire disconnect

## Sprint 10: Sovereign Distribution, Offline Licensing & Package Packaging (Complete)
- [x] **10.1 Automated Local Backup Daemon & Age-Based Pruning (`proteus-core`)**
  - [x] `LocalBackupManager` in `%APPDATA%\Proteus\backups\`
  - [x] Argon2id + XChaCha20Poly1305 encrypted snapshots with verified `.manifest.json`
  - [x] Age-based retention pruning and interval-controlled execution
- [x] **10.2 Canvas Drag-and-Drop Ingestion (`proteus-design-studio`)**
  - [x] Native OS file drop ingestion of `.prproj` and `.json` client briefs
  - [x] Auto-generation of screens/artboards and canvas undo/redo reset
  - [x] Visual HUD drop indicator overlay
- [x] **10.3 Native Hardware Audio Feedback Engine (`proteus-core` / `proteus-client`)**
  - [x] Zero-dependency non-blocking Win32 `MessageBeep` audio engine in `proteus-core::audio`
  - [x] `play_barcode_chime` and `play_error_tone` wired to intake, Kanban search, and GS1-128 decoder
- [x] **10.4 Direct `.pr` Package Export (`proteus-design-studio`)**
  - [x] Menu Bar File -> Export as .pr Package (`Ctrl+E`)
  - [x] Command Palette action and keyboard shortcut bindings
  - [x] SHA-256 integrity seal, views, flows, and additive DDL extraction
- [x] **10.5 Offline Cryptographic License Issuer & Pilot Tooling (`proteus-core` / `proteus-client`)**
  - [x] HMAC-SHA256 offline token generation (`PROT-LIC-...`)
  - [x] 1-Click 30-day pilot token generator in Settings view
  - [x] Tamper detection, expiration checking, and machine binding
- [x] **10.6 Self-Service Encrypted Backup Portal (`proteus-web`)**
  - [x] `GET /api/v1/backups/list` endpoint
  - [x] `POST /api/v1/backups/create` endpoint
  - [x] `GET /api/v1/backups/download/:filename` endpoint
- [x] **10.7 Component Genealogy, Serial Number Tracking & Centralized RMA Hub (`proteus-core` & `proteus-client`)**
  - [x] Full physical lifecycle tracking (`SupplierIntake` ➔ `WarehouseStock` ➔ `InstalledInCustomerDevice` ➔ `RmaClaimInitiated` ➔ `RmaReplacedBySupplier` ➔ `CreditNoteIssued`)
  - [x] Real-time dynamic warranty countdown (`WarrantyStatus::Valid` vs `WarrantyStatus::Expired`)
  - [x] Embedded RMA & Serial Number Inspector in ticket detail modal (`embedded_genealogy.rs`)
  - [x] Centralized RMA Hub monitoring active supplier warranty claims across store branches (`genealogy_rma.rs`)
- [x] **10.8 High-Concurrency SQLite & Cryptographic Hardening (`proteus-core` & `auth-server`)**
  - [x] Storage tuning: `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, and `PRAGMA busy_timeout = 5000;` across all connections
  - [x] Axum `auth-server`: Hardened runtime fallback in release mode using `OsRng` 256-bit random key if `JWT_SECRET` is unset
  - [x] Protected LAN receiver (port 7443) with strict payload capping and DoS mitigation
- [x] **10.9 Unbreakable 6-Stage Automated Verification Pipeline (`scripts/verify_pipeline.ps1` & `.bat`)**
  - [x] Gate 1: Type check, Gate 2: Storage pragmas, Gate 3: Zero-mock scan, Gate 4: 304/304 tests, Gate 5: Binaries, Gate 6: Modularity
  - [x] 100% passing tests across all 7 workspace crates (304 tests passing, 0 warnings, 0 failures)

## Sprint 11: Global Logistics, Digital Shipping Note & Dispatch Companion (Complete)
- [x] **11.1 SQLite Schema & DDL for Digital Shipping Notes (`proteus-core`)**
  - [x] Tables: `shipping_notes`, `shipping_note_items`, `shipping_outbox`
  - [x] Fields: Sender AFM, Recipient AFM, Vehicle plate, Departure/Arrival timestamps, Gross weight, Transport purpose
- [x] **11.2 IAPR / myDATA & e-CMR QR-Code Generator (`proteus-core`)**
  - [x] Standardized QR payload generation complying with Greek AADE digital transport mandate
  - [x] Cryptographic SHA-256 hash verification and offline checksum
- [x] **11.3 ESC/POS Thermal Delivery Waybill Voucher (`proteus-core` & `proteus-client`)**
  - [x] Printable 80mm & 58mm delivery note format with items summary, QR code, and driver/receiver signature line
- [x] **11.4 Van Sales & Offline Dispatch Outbox Queue (`proteus-core`)**
  - [x] Store-and-forward outbox (`shipping_outbox`) for delivery drivers with weak/no cellular reception
- [x] **11.5 Frontline Dispatch Inspector View (`proteus-client`)**
  - [x] Frontline view in `proteus-client` (`shipping_notes.rs`) with search, filter, and 1-click delivery sign-off
  - [x] 305/305 automated tests passing (100% green)

## Sprint 12: Van Sales, Mobile Sign-on-Glass & Consignment Tracking (ACTIVE)
- [ ] **12.1 Handheld Touch Sign-on-Glass Capture (`proteus-mobile`)**
  - [ ] Vector signature path capture on mobile touch screen
  - [ ] Embedding receiver signature into delivery outbox record
- [ ] **12.2 Bluetooth Mobile Thermal Slip Printing (`proteus-mobile`)**
  - [ ] Direct Bluetooth SPP / BLE raw ESC/POS slip generation for mobile belt printers
- [ ] **12.3 Vendor-Managed Inventory (VMI) & Consignment Tracking (`proteus-core`)**
  - [ ] Consignment stock ledger tracking goods held at third-party partner premises

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

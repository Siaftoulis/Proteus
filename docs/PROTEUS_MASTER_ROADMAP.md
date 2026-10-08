# Proteus Sovereign BOS — Master Implementation Roadmap

Hierarchical Task Breakdown: Task -> Sub-task -> Micro-task
Methodology: Atomic implementation ("νια-νια"), 100% original bespoke code, zero mock data, continuous zero-bug verification.

---

## Phase 1: Commercial Engine & myDATA Fiscal Core

### Task 1.1: Direct myDATA REST Engine & Invoicing (`proteus-core::invoicing`)
- [x] Sub-task 1.1.1: Fiscal Schemas & Invoicing DDL in SQLite
  - [x] Micro-task 1.1.1.1: Create `invoices`, `invoice_lines`, and `mydata_transmissions` tables in `store.db`.
  - [x] Micro-task 1.1.1.2: Implement official myDATA document types enum (1.1 Sale, 1.2 Intra-EU, 2.1 Services, 5.1 Credit Note, 8.1 Rent, 11.1 Retail).
  - [x] Micro-task 1.1.1.3: Implement Greek VAT categories (24%, 13%, 6%, 0% with exemption articles 22/39/etc.).
  - [x] Micro-task 1.1.1.4: Implement income classification categories (e.g., category1_1, E3_561_001).
- [x] Sub-task 1.1.2: Tax Calculation & Asynchronous Dispatch Queue
  - [x] Micro-task 1.1.2.1: Mathematical precision net, VAT, and gross totals rounding without floating-point drift.
  - [x] Micro-task 1.1.2.2: XML/JSON payload generator for AADE `SendInvoices` REST endpoint.
  - [x] Micro-task 1.1.2.3: Non-blocking background worker queue for transmission with exponential backoff on AADE timeouts.
  - [x] Micro-task 1.1.2.4: Official IAPR/myDATA QR-code generator with MARK, UID, and digital voucher signatures.
- [x] Sub-task 1.1.3: Automated VIES & AADE Tax Registry Client
  - [x] Micro-task 1.1.3.1: Live AFM lookup client to auto-populate company name, trade name, tax office (ΔΟΥ), and registered address.

---

## Phase 2: Zero-Subscription Touch Retail POS & Accounting Bridge

### Task 2.1: Frontline Touch Cashier (`proteus-client::views::pos`)
- [x] Sub-task 2.1.1: Touchscreen Cash Desk Interface
  - [x] Micro-task 2.1.1.1: Touch quick-pills, rapid number pad, and barcode scanner listener.
  - [x] Micro-task 2.1.1.2: Multi-mode payments (Cash, Card / EFT-POS A.1155/2023 lock, On Credit / Contractor Ledger).
  - [x] Micro-task 2.1.1.3: ESC/POS thermal receipt formatting with native QR codes and cash drawer kick pulse.
- [x] Sub-task 2.2.1: 1-Click Accounting Bridge
  - [x] Micro-task 2.2.1.1: Daily Z report generation aggregating cash/card receipts and VAT buckets.
  - [x] Micro-task 2.2.1.2: Export formatted accounting ledger (Excel/JSON) matching standard Greek CPA software schemas.

---

## Phase 3: Universal 3D Spatial WMS & Retail Shelf Optimizer

### Task 3.1: Spatial Warehouse Architecture & Geometry (`proteus-core::wms`)
- [x] Sub-task 3.1.1: Physical Shelf & Zone Data Models
  - [x] Micro-task 3.1.1.1: SQLite schema for `warehouse_zones`, `warehouse_racks`, and `warehouse_shelves`.
  - [x] Micro-task 3.1.1.2: Shelf boundary definition: Width, Height, Depth (cm) and Maximum Payload Weight (kg).
  - [x] Micro-task 3.1.1.3: Ergonomic Golden Zone classification (Fast Movers at 0.8m-1.6m vs Heavy Ground vs Slow Movers >2m).
- [x] Sub-task 3.1.2: Mathematical 3D Bin-Packing Engine
  - [x] Micro-task 3.1.2.1: 4-type storage classification: Cuboid (boxes), Stackable (textiles), Hanging (garments), Bins/Totes (loose parts).
  - [x] Micro-task 3.1.2.2: Spatial bin-packing optimization calculating maximum unit capacity and spatial orientation.
  - [x] Micro-task 3.1.2.3: Enforcement of 3-5% handling clearance margin and maximum shelf weight limits.
- [x] Sub-task 3.1.3: Directed Putaway & Fast Sweep Scanning
  - [x] Micro-task 3.1.3.1: Directed putaway routing suggesting exact shelf and bay for incoming goods.
  - [x] Micro-task 3.1.3.2: Rapid shelf sweep audit mode with audio alerts on misplaced items.
- [x] Sub-task 3.1.4: Lightweight Visual Map & Photo Verification
  - [x] Micro-task 3.1.4.1: Native 2D/3D visual locator highlighting exact aisle, shelf, and tier in UI.
  - [x] Micro-task 3.1.4.2: Photo proof attachment with Merkle audit block and chain-of-custody tracking.

---

## Phase 4: HR & Digital Work Card (Ergani II)

### Task 4.1: Ergani II Digital Work Card Engine (`proteus-core::work_card`)
- [x] Sub-task 4.1.1: Employee Shift & Clock-in Models
  - [x] Micro-task 4.1.1.1: SQLite schema for `work_shifts`, `work_card_events` (ClockIn, ClockOut, BreakStart, BreakEnd).
  - [x] Micro-task 4.1.1.2: PIN and QR badge fast check-in for staff at store counters.
- [x] Sub-task 4.1.2: Cryptographic Offline Queue & Sync
  - [x] Micro-task 4.1.2.1: Offline check-in logging with monotonic timestamps and Merkle audit blocks.
  - [x] Micro-task 4.1.2.2: Background synchronization worker with statutory internet-outage markers for Ergani II.

### Task 4.2: Frontline Touch Work Card & Ergani II Kiosk (`proteus-client::views::work_card`)
- [x] Sub-task 4.2.1: Touch Staff Kiosk & Badge Verification
  - [x] Micro-task 4.2.1.1: PIN touch pad & rapid QR badge reader for employee clock events.
  - [x] Micro-task 4.2.1.2: Event state machine (ClockIn -> BreakStart -> BreakEnd -> ClockOut) with visual status badges.
- [x] Sub-task 4.2.2: Statutory Outage Monitor & Payroll Timesheet
  - [x] Micro-task 4.2.2.1: Statutory telecom outage toggle with legal notice under Law 4808/2021.
  - [x] Micro-task 4.2.2.2: Timesheet ledger displaying daily regular hours and overtime (>8h) with CPA export.

---

## Phase 5: Verification & Zero-Bug Compliance Gate

### Task 5.1: Automated Compliance & Edge-Case Test Suite
- [x] Sub-task 5.1.1: Fiscal & myDATA unit and integration test suite.
- [x] Sub-task 5.1.2: 3D bin-packing edge cases (overweight, irregular dimensions, tight fit).
- [x] Sub-task 5.1.3: Ergani II offline resilience and network failure recovery tests.
- [x] Sub-task 5.1.4: Zero-bug invariant validation across full workspace (`cargo test --workspace` - 342/342 passing).

---

## Phase 6: Electronic Shelf Labels (ESL) & Dynamic Expiry Pricing

### Task 6.1: ESL Gateway & Dynamic Discount Engine (`proteus-core::esl`)
- [x] Sub-task 6.1.1: ESL Tag Data Models & Radio Protocol Payload Generator
  - [x] Micro-task 6.1.1.1: SQLite schema for `esl_tags` and `esl_broadcast_queue`.
  - [x] Micro-task 6.1.1.2: Binary/hex frame generator for 2.9" and 4.2" e-ink displays (Red/Black/White).
- [x] Sub-task 6.1.2: Dynamic Expiry Markdown Engine (GS1 AI 17 Integration)
  - [x] Micro-task 6.1.2.1: Automated shelf price discounting based on shelf-life days remaining.
  - [x] Micro-task 6.1.2.2: Instant broadcast trigger updating e-ink tag without manual paper tag replacement.

---

## Phase 7: 1-Click Accounting & AADE myDATA Live Synchronization Engine

### Task 7.1: AADE myDATA Live REST Client & Outbox Sync Daemon (`proteus-core::mydata_sync`)
- [x] Sub-task 7.1.1: AADE REST Client & TLS Transmission
  - [x] Micro-task 7.1.1.1: Native `MyDataEnvironment` (Production vs Development) and credential routing (`user_id`, `subscription_key`).
  - [x] Micro-task 7.1.1.2: Resilient zero-dependency XML parser extracting ResponseDoc, MARK, UID, and validation error codes.
  - [x] Micro-task 7.1.1.3: Asynchronous outbox batch processor with automatic retry backoff and offline preservation.
- [x] Sub-task 7.1.2: 1-Click CPA Accounting Reconciliation & Exports (`proteus-core::cpa_sync`)
  - [x] Micro-task 7.1.2.1: Multi-rate VAT allocation buckets (24%, 13%, 6%, 0% Art. 22/39 exemptions) and gross reconciliation.
  - [x] Micro-task 7.1.2.2: Compliance audit discrepancy detector (unfiled receipts, missing MARKs, mathematical drift).
  - [x] Micro-task 7.1.2.3: Standardized CPA JSON export (Greek Accounting Standards Law 4308/2014 ΕΛΠ).
  - [x] Micro-task 7.1.2.4: European/Greek CSV general ledger export with semicolon delimiters and comma decimals.

### Task 7.2: Frontline Touch Accounting & myDATA Monitor (`proteus-client::views::accounting`)
- [x] Sub-task 7.2.1: Sovereign Accounting Dashboard
  - [x] Micro-task 7.2.1.1: Top KPI cards (Gross Revenue, Net Sales, Total VAT, Pending myDATA outbox queue).
  - [x] Micro-task 7.2.1.2: 1-Click "Άμεσος Συγχρονισμός myDATA" trigger communicating with AADE API.
  - [x] Micro-task 7.2.1.3: 1-Click "Εξαγωγή για Λογιστή" (JSON/CSV) exporting directly to local accounting files.
  - [x] Micro-task 7.2.1.4: Compliance guardian audit panel alerting on unfiled invoices or transmission anomalies.

---

## Phase 8: Customer & Supplier Financial Cardex & Balance Aging Engine

### Task 8.1: Customer & Supplier Cardex Ledger Engine (`proteus-core::cardex`)
- [x] Sub-task 8.1.1: Financial Cardex Schemas & Rolling Balances
  - [x] Micro-task 8.1.1.1: SQLite schema for `cardex_entities` (credit limits, payment terms) and `cardex_entries` (Debit, Credit, Running Balance).
  - [x] Micro-task 8.1.1.2: Official payment collection receipt generator (`record_payment_receipt`) with automated debt offset.
  - [x] Micro-task 8.1.1.3: Credit limit enforcement (`check_credit_limit_ok`) preventing unauthorized over-limit purchases.
- [x] Sub-task 8.1.2: 5-Tier Statutory Balance Aging Analysis
  - [x] Micro-task 8.1.2.1: Mathematical aging breakdown across 5 brackets: 0-30d, 31-60d, 61-90d, 91-120d, and 120+ days.
  - [x] Micro-task 8.1.2.2: Signed debit/credit accounting ensuring zero mathematical drift across aging periods.

### Task 8.2: Frontline Cardex & Aging Studio (`proteus-client::views::cardex`)
- [x] Sub-task 8.2.1: Sovereign Commercial Cardex Dashboard
  - [x] Micro-task 8.2.1.1: Entity directory with live balances, customer/supplier badges, and filter chips.
  - [x] Micro-task 8.2.1.2: Visual credit capacity usage bar and 5-tier aging indicators.
  - [x] Micro-task 8.2.1.3: Chronological ledger movements table (Χρέωση / Πίστωση / Υπόλοιπο).
  - [x] Micro-task 8.2.1.4: Rapid payment collection modal with instant balance offset and receipt issuance.

---

## Phase 9: Multi-Branch LAN Mesh Synchronization Engine

### Task 9.1: Peer-to-Peer Branch Mesh Discovery Protocol (`proteus-core::mesh`)
- [x] Sub-task 9.1.1: Branch Mesh Beacon Schema & Peer Types
  - [x] Micro-task 9.1.1.1: Branch beacon data model, protocol magic byte validation, and SQLite DDL (`mesh_peers`, `mesh_sync_epochs`).
  - [x] Micro-task 9.1.1.2: Deterministic SHA-256 catalog fingerprinting (`compute_catalog_checksum`).
- [x] Sub-task 9.1.2: Autonomous UDP Broadcast Engine
  - [x] Micro-task 9.1.2.1: Non-blocking socket transmitter and receiver with MTU boundary guards (`MAX_BEACON_PACKET_SIZE`).
  - [x] Micro-task 9.1.2.2: Live beacon ingestion, ping latency calculation, and self-announcement filtering.
- [x] Sub-task 9.1.3: Peer Heartbeat & Topology Health Tracking
  - [x] Micro-task 9.1.3.1: Automatic heartbeat loop and non-destructive TTL stale peer pruning (`prune_stale_peers`).
  - [x] Micro-task 9.1.3.2: Mesh topology health evaluator detecting coordinator branch and catalog synchronization drift.

### Task 9.2: Differential SQLite Transaction Vector Clock (`proteus-core::mesh`)
- [x] Sub-task 9.2.1: Distributed Vector Clock Structures
  - [x] Micro-task 9.2.1.1: Monotonic branch sequence counter mapping, point-wise maximum merge, and partial ordering evaluation (`Dominates`, `Dominated`, `Concurrent`).
  - [x] Micro-task 9.2.1.2: SQLite vector clock persistence and local mutation recording (`mesh_vector_clocks`).
- [x] Sub-task 9.2.2: Differential Transaction Log & Branch Delta Extraction
  - [x] Micro-task 9.2.2.1: Transaction log schema (`mesh_delta_log`) and atomic mutation appending.
  - [x] Micro-task 9.2.2.2: Selective delta packaging transmitting only unseen mutations based on peer vector clocks.
- [x] Sub-task 9.2.3: Cross-Branch Convergence & Deterministic Conflict Resolution
  - [x] Micro-task 9.2.3.1: Chronological and lexicographical deterministic tie-breaker resolving concurrent split-brain mutations.
  - [x] Micro-task 9.2.3.2: Remote delta ingestion engine applying dominating mutations and merging causal lineages.

### Task 9.3: Multi-Branch Sync Monitor View (`proteus-client::views::branch_mesh`)
- [x] Sub-task 9.3.1: Multi-Branch Sync State & Topology Health Header
  - [x] Micro-task 9.3.1.1: High-density 4-card metric header (Active Peers, Catalog Sync, Node Role, Checksum).
  - [x] Micro-task 9.3.1.2: Auto-seeding and live state refresh from SQLite store.
- [x] Sub-task 9.3.2: Live Peer Node Cards & Catalog Drift Badges
  - [x] Micro-task 9.3.2.1: Peer store card layout showing IP/Port, ping latency, and epoch counters.
  - [x] Micro-task 9.3.2.2: Color-coded catalog sync status badges (`✓ Συγχρονισμένος` vs `⚠ Απόκλιση`).
- [x] Sub-task 9.3.3: 1-Click Delta Sync Trigger & Navigation Integration
  - [x] Micro-task 9.3.3.1: 1-Click `⚡ Άμεσος Συγχρονισμός Mesh` button and reconciliation execution.
  - [x] Micro-task 9.3.3.2: Role-driven sub-navigation integration (`NavTab::BranchMesh` under SystemsIT & EnterpriseHQ).

---

## Phase 10: Standalone Application Extraction, Marketplace & Sovereign Distribution

### Task 10.1: Standalone Application Extraction Engine (`proteus-core::standalone`)
- [x] Sub-task 10.1.1: Standalone App Manifest & Cross-Platform Matrix (Windows, macOS, Linux, Android, iOS).
- [x] Sub-task 10.1.2: Cryptographic SHA-256 sealed bundle packaging (`StandaloneBundle`) and build orchestration.
- [x] Sub-task 10.1.3: SQLite persistent build log schema (`standalone_build_logs`).

### Task 10.2: iOS Sideloading Architecture & OTA Manifests (`proteus-core::standalone::ios`)
- [x] Sub-task 10.2.1: EU DMA AltStore PAL / SideStore `apps.json` feed generation (`AltStoreSource`).
- [x] Sub-task 10.2.2: Apple Enterprise wireless OTA `manifest.plist` generator.

### Task 10.3: Software Update & Verification Engine (`proteus-core::updater`)
- [x] Sub-task 10.3.1: Monotonic SemVer parsing and delta release manifest verification (`ReleaseManifest`).
- [x] Sub-task 10.3.2: Atomic staged binary replacement with `.bak` rollback and SQLite update audit log (`update_audit_log`).
- [x] Sub-task 10.3.3: Interactive Terminal Launcher integration (`[7] Check Ecosystem Updates` in `scripts/launcher.ps1`).

### Task 10.4: Template Marketplace Catalog & Discovery Engine (`proteus-core::package::marketplace`)
- [x] Sub-task 10.4.1: Multi-criteria template catalog (Retail, Automotive, Healthcare, Hospitality, Services, Logistics).
- [x] Sub-task 10.4.2: 1-Click conversion to SHA-256 sealed `.pr` packages (`PrPackage`).
- [x] Sub-task 10.4.3: SQLite marketplace cache schema (`template_marketplace_cache`).

### Task 10.5: Distributed Multi-User Collaboration & Lock Lease Engine (`proteus-core::collab`)
- [x] Sub-task 10.5.1: Distributed record lock lease engine (`RecordLock`, `RecordLockManager` with TTL and heartbeat).
- [x] Sub-task 10.5.2: Concurrency conflict detection (`LockConflict`) and deterministic Last-Write-Wins CRDT register (`LwwMutation`).
- [x] Sub-task 10.5.3: SQLite lease lock persistence (`collab_record_locks`).

### Task 10.6: Sovereign i18n & Greek/English Localization Engine (`proteus-core::i18n`)
- [x] Sub-task 10.6.1: Native `Locale` resolver (`el-GR` primary, `en-US` fallback) and base domain dictionary.
- [x] Sub-task 10.6.2: Variable placeholder interpolation (`interpolate`) and European currency formatting (`format_currency`).
- [x] Sub-task 10.6.3: SQLite custom terminology translation overrides schema (`i18n_translation_overrides`).

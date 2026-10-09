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
- [x] Pilot testing and feedback collection onboarding manual for 2–3 local repair shops (`docs/PILOT_SHOP_ONBOARDING_MANUAL.md`)

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

## Sprint 12: Van Sales, Mobile Sign-on-Glass & Consignment Tracking (Complete)
- [x] **12.1 Handheld Touch Sign-on-Glass Capture (`proteus-mobile`)**
  - [x] Vector signature path capture on mobile touch screen (`SignOnGlassPad`)
  - [x] Vector stroke serialization and SVG path export (`export_svg`, `export_compact_string`)
  - [x] Embedding receiver signature into delivery outbox record and SQLite waybill
- [x] **12.2 Bluetooth Mobile Thermal Slip Printing (`proteus-mobile`)**
  - [x] Direct Bluetooth SPP / BLE raw ESC/POS slip generation for 58mm mobile belt printers (`generate_bluetooth_mobile_slip`)
  - [x] Van sales dispatch companion board with real-time delivery status advancement
- [x] **12.3 Vendor-Managed Inventory (VMI) & Consignment Tracking (`proteus-core`)**
  - [x] Consignment stock ledger tracking goods held at third-party partner premises (`ConsignmentPartner`, `ConsignmentStockItem`)
  - [x] Real SQLite tracking of consignment movements (transfers in, consumption sales, returns)
  - [x] Automated replenishment threshold alert triggers (`check_vmi_replenishment_alerts`)
- [x] **12.4 Automated Verification Pipeline & Test Suite**
  - [x] 313/313 tests passing across all 7 workspace crates (100% green, 0 compiler warnings)

## Sprint 13: Cold Chain HACCP Telemetry & IoT Sensor Bridge (Complete)
- [x] **13.1 Cold Chain HACCP SQLite Schema & Threshold Rules (`proteus-core`)**
  - [x] Temperature/humidity sensor logging tables (`cold_chain_sensors`, `cold_chain_logs`, `haccp_breach_events`)
  - [x] Regulatory cold storage types (`DeepFreeze`, `Chilled`, `ControlledAmbient`, `PharmaCold`)
  - [x] Automatic breach detection on excursion above/below critical temperature limits with severity classification
- [x] **13.2 Merkle Audit Hash Chain for HACCP Compliance (`proteus-core`)**
  - [x] Cryptographic SHA-256 seal per telemetry reading block for tamper-proof food/pharma safety certification
  - [x] Official HACCP Compliance Certificate generator calculating in-spec percentages and Merkle root hash
- [x] **13.3 Real-Time Fleet & Cold Storage Telemetry Dashboard (`proteus-client`)**
  - [x] Dedicated view in `proteus-client` (`cold_chain.rs`) with live sensor cards, temperature indicators, and door sensors
  - [x] Active Breaches alert panel with 1-click operator corrective action logging modal
  - [x] 1-click HACCP compliance certificate inspection modal with SHA-256 Merkle root verification
- [x] **13.4 Verification Pipeline & Test Suite**
  - [x] 319/319 tests passing across all 7 workspace crates (100% green, 0 compiler warnings)

## Sprint 14: Multi-Branch LAN Mesh Synchronization (Complete)
- [x] **14.1 Peer-to-Peer Branch Mesh Discovery Protocol (`proteus-core`)**
  - [x] Micro-task 14.1.1: Branch mesh beacon schema, peer types & SQLite DDL (`mesh_peers`, `mesh_sync_epochs`).
  - [x] Micro-task 14.1.2: UDP broadcast beacon engine and packet parser.
  - [x] Micro-task 14.1.3: Peer heartbeat and topology health tracking.
- [x] **14.2 Differential SQLite Transaction Vector Clock (`proteus-core`)**
  - [x] Micro-task 14.2.1: Vector clock data structures & monotonic mutation tracking (`proteus-core::mesh::vector_clock`).
  - [x] Micro-task 14.2.2: Differential transaction log & branch delta extraction (`proteus-core::mesh::delta`).
  - [x] Micro-task 14.2.3: Cross-branch convergence & deterministic conflict resolution (`proteus-core::mesh::reconcile`).
- [x] **14.3 Multi-Branch Sync Monitor View (`proteus-client`)**
  - [x] Micro-task 14.3.1: Multi-Branch sync state & topology health header (`proteus-client::views::branch_mesh`).
  - [x] Micro-task 14.3.2: Live peer node cards & catalog drift badge rendering.
  - [x] Micro-task 14.3.3: 1-Click branch delta sync trigger & navigation tab integration.

## Phase 2: Military Service Period (Nov 2026 – May 2027)
- [ ] 2–3 shop pilot testing & bugfixing during leaves
- [ ] Zero monthly expenses maintenance

## Phase 3: Commercial Launch (May 2027)
- [x] Lemon Squeezy / Paddle Merchant of Record (MoR) setup
  - [x] Micro-task P3.4.1: Merchant of Record (MoR) Webhook & Signature Verification Engine with constant-time HMAC-SHA256 (`proteus-core::license::mor`)
- [x] Core business license (7.99€/mo) & seat tier checkout
  - [x] Micro-task P3.5.1: Merchant of Record (MoR) Dynamic Seat Tier Checkout Session Generator & Provisioning Engine (`proteus-core::license::checkout`)
- [x] Local LAN mDNS auto-discovery & QR pairing
  - [x] Micro-task P3.1.1: Cryptographic QR pairing payload generation, TTL expiration, & SQLite device authorization schema (`proteus-core::lan::pairing`)
- [x] Mobile Companion App with `sync_outbox`
  - [x] Micro-task P3.2.1: Mobile Companion Outbox Auto-Sync Daemon, exponential backoff, & dynamic status indicators (`proteus-mobile::sync_daemon`)
- [x] Zero-knowledge cloud snapshot backup (Cloudflare R2 / S3)
  - [x] Micro-task P3.3.1: Cloudflare R2 / S3 client with bespoke AWS SigV4 signing, zero-knowledge payload upload/download & S3 XML parser (`proteus-core::cloud_backup::s3_client`)
- [x] Zero-config Cloud Relay Tunnel for Remote Companion Sync
  - [x] Micro-task P3.6.1: Zero-configuration cloud relay tunnel schema, authenticated session handshake, tamper-proof envelope, and SQLite config persistence (`proteus-core::lan::relay`)
  - [x] Micro-task P3.6.2: Cloud Relay Companion resilient sync fallback, dual transport indicators, and modular tickets tab (`proteus-mobile::sync_daemon`, `proteus-mobile::tickets_tab`)

## Phase 4: Company Formation & Cloud Infrastructure (Post-Revenue >2,000€)
- [x] Electronic establishment of Single-Member IKE (gov.gr)
  - [x] Micro-task P4.1.1: Single-Member IKE (Μονοπρόσωπη Ι.Κ.Ε.) Electronic Charter, Law 4072/2012 articles, KAD validator & corporate filings schema (`proteus-core::legal::ike`)
- [x] Cloud Hosting Management Console
  - [x] Micro-task P4.2.1: Native Cloud Hosting View (`draw_cloud_hosting_view`), DNS zone manager, 1-click Cloudflare R2 backup, dynamic federated web DB subscriptions & tier provisioning (`proteus-client::views::cloud_hosting`)

## Phase 5: CRM Standalone Application Extraction Engine
- [x] CRM extraction (standalone app build)
  - [x] Micro-task P5.1.1: Standalone Application Manifest (`StandaloneAppManifest`), 6 cross-platform targets (`TargetPlatform`), SHA-256 sealed bundle packaging (`StandaloneBundle`), cargo release orchestrator & SQLite build log schema (`proteus-core::standalone`)
- [x] iOS Sideloading Research & Manifest Engine
  - [x] Micro-task P5.2.1: Technical Research & Architecture blueprint (`docs/IOS_SIDELOADING_ARCHITECTURE.md`), EU DMA AltStore PAL / SideStore `apps.json` feed generator (`AltStoreSource`), & Apple Enterprise OTA `manifest.plist` builder (`proteus-core::standalone::ios`)
- [x] Mobile App Support (iOS + Android)
  - [x] Micro-task P5.3.1: Hardware Capability & Sensory Telemetry Bridge (`DeviceContext`), Safe Area Insets calculation (`SafeAreaInsets` iPhone notch & Android bar), tactile haptic feedback patterns, & biometric auth detection (`proteus-mobile::device`)
- [x] Automatic Updates via Launcher
  - [x] Micro-task P5.4.1: Cryptographic Software Update & Release Verification Engine (`SemVer`, `ReleaseManifest`, `verify_payload_sha256`, atomic staged binary replacement with `.bak` rollback & SQLite audit log `update_audit_log` in `proteus-core::updater`, plus interactive Terminal Launcher update verifier in `scripts/launcher.ps1`)
- [x] Template Marketplace
  - [x] Micro-task P5.5.1: Template Marketplace Catalog & Discovery Engine (`TemplateCategory`, `TemplateListing`, `TemplateCatalog` multi-criteria search, 1-click conversion to sealed `PrPackage`, & SQLite cache schema `template_marketplace_cache` in `proteus-core::package::marketplace`)
- [x] Multi-User Collaboration & Real-Time Sync
  - [x] Micro-task P5.6.1: Distributed Record Lock Lease Engine (`RecordLock`, `RecordLockManager` with heartbeat and TTL expiration), Lock Conflict detection (`LockConflict`), deterministic Last-Write-Wins CRDT register (`LwwMutation`), & SQLite lease lock persistence in `proteus-core::collab`
- [x] i18n / Localization
  - [x] Micro-task P5.7.1: Sovereign Internationalization (i18n) & Localization Engine (`Locale` resolution for Greek `el-GR` / English `en-US`, bilingual domain terminology dictionary, currency/decimal formatting, dynamic translation overrides & SQLite persistence in `proteus-core::i18n`)

## Phase 11: Dynamic Business Discovery & Visual Flow Engine
- [x] Task 11.1: Business Questionnaire & Requirements Engine
  - [x] Micro-task 11.1.1: Hierarchical Business Category Ontology & Workflow Checklist Engine (`TopIndustryCategory`, `SubCategory`, `WorkflowRequirements`, `BusinessScale`, `DesignStrategy` & SQLite persistence in `proteus-core::questionnaire`)
  - [x] Micro-task 11.1.2: Starter Package Generator from Questionnaire (`generate_starter_package_from_answers` mapping questionnaire answers into sealed `PrPackage` with domain DDL, views, and flow triggers in `proteus-core::questionnaire::scaffold`)
  - [x] Micro-task 11.1.3: Web Interactive Hierarchical Tree UI & Enterprise Solution Architect Desk in `proteus-web::ui_onboarding`
- [x] Task 11.2: Visual Flow Node Graph with Bezier Connectors ("Scratch-Style")
  - [x] Micro-task 11.2.1: Node Port & Wire Connection Data Model in `proteus-design-studio::flow`
  - [x] Micro-task 11.2.2: Bezier Wire Renderer & Scratch-Style Block Layout in `proteus-design-studio::flow_renderer`
  - [x] Micro-task 11.2.3: Interactive Port Drag-and-Snap & Multi-Branch Routing in `proteus-design-studio::views::flow_builder`

## Phase 15: Isolated Multi-Tenant Cloud Hosting & Staging Suite
- [x] Task 15.1: Multi-Tenant Cloud Infrastructure & Migration Engine
  - [x] Micro-task 15.1.1: Staging vs Production Environment Segregation & Subdomain Routing (`proteus-web::staging`)
  - [x] Micro-task 15.1.2: Multi-Tenant Data Isolation Engine with quota enforcement e.g. 50GB (`proteus-core::hosting::isolation`)
  - [x] Micro-task 15.1.3: Automated VPS Billing Markup Calculator with 50% margin (`proteus-core::hosting::billing`)
  - [x] Micro-task 15.1.4: 1-Click Zero-Friction Migration between Cloud Hosted & Local Self-Hosted (`proteus-core::hosting::migration`)
  - [x] Micro-task 15.1.5: Safe Schema Migrator & Canary Health Reporter Agent (`proteus-core::hosting::canary`)

## Phase 16: Universal Database Scripting DSL & Visual Scratch Data Blocks
- [x] Task 16.1: Declarative Database Language & AST Parser
  - [x] Micro-task 16.1.1: Proteus Declarative Database DSL AST & Syntax Parser (`proteus-core::dsl`)
  - [x] Micro-task 16.1.2: Multi-Dialect SQL/NoSQL Transpiler for SQLite, PostgreSQL, MySQL, MongoDB (`proteus-core::dsl::transpiler`)
  - [x] Micro-task 16.1.3: Visual Scratch Data Blocks in Flow Studio with drag-and-drop ID linking (`proteus-design-studio::flow::data_blocks`)

## Phase 17: Lunacy-Grade Design Studio Canvas & Precision Toolbox
- [x] Task 17.1: Professional Vector & Component Editing Experience
  - [x] Micro-task 17.1.1: Lunacy-Grade Left Sidebar & Layers Tree with Symbols and Asset palette (`proteus-design-studio::views::layers_panel`)
  - [x] Micro-task 17.1.2: Top Tool Ribbon for Select, Frame, Primitives, Text, Vector, Device Presets (`proteus-design-studio::views::tool_ribbon`)
  - [x] Micro-task 17.1.3: Right Precision Inspector for Geometry X/Y/W/H/Rotation, Alignment bar, Fills, Typography (`proteus-design-studio::inspector::precision`)

## Phase 18: Vector Canvas Precision Renderers & Visual Shape Engine
- [x] Task 18.1: Native Vector Geometry Canvas Renderers
  - [x] Micro-task 18.1.1: Vector Ellipse & Circle GPU Renderer with Anti-Aliased Corner Radius (`proteus-design-studio::renderer`)
  - [x] Micro-task 18.1.2: Vector Line & Arrow Segment Canvas Renderer with Stroke Styling (`proteus-design-studio::renderer`)
  - [x] Micro-task 18.1.3: Visual Bounding Box Rotation & Transform Handles on Canvas (`proteus-design-studio::views::designer::canvas`)

## Phase 19: Sovereign Model Context Protocol (MCP) AI Architecture Engine
- [x] Task 19.1: Local AI Co-Pilot & Declarative MCP Engine
  - [x] Micro-task 19.1.1: JSON-RPC 2.0 Protocol & MCP Server Engine in Core (`proteus-core::mcp::server`)
  - [x] Micro-task 19.1.2: Canvas Layout & Schema Generation MCP Tools (`proteus-core::mcp::tools`)
  - [x] Micro-task 19.1.3: Studio MCP Bridge & Live WebSocket/Pipe Dispatcher (`proteus-design-studio::ai_bridge`)

## Phase 20: SMLM Real-Time Federation Bridge & Global Logistics
- [x] Task 20.1: Universal Cross-Database Federation
  - [x] Micro-task 20.1.1: Multi-Store Cross-Database Federation Engine (`proteus-core::federation`)
  - [x] Micro-task 20.1.2: Live Logistics Sync & Automated Stock Level Arbitrage (`proteus-core::logistics`)

## Phase 21: Quality Linter Gate & Additive Delta Package Updates
- [x] Task 21.1: Automated Template Quality Linter & Certification Gate
  - [x] Micro-task 21.1.1: Multi-Vector Automated Template Linter Engine (`proteus-core::package::linter`)
  - [x] Micro-task 21.1.2: Additive Delta Package Format & Transactional Patch Runner (`proteus-core::package::delta`)
  - [x] Micro-task 21.1.3: Studio Linter Pre-Flight Modal & 1-Click Export Gate (`proteus-design-studio`)

## Phase 22: In-App Collaborative Share Point, Sandbox Preview & Cryptographic Escrow Delivery
- [x] Task 22.1: In-App Client-Designer Collaboration & Isolated Sandbox Engine
  - [x] Micro-task 22.1.1: Requirements Package Specification & Share Point Protocol (`proteus-core::package::sharepoint`)
  - [x] Micro-task 22.1.2: In-Platform Escrow & Cryptographic Delivery Gate (`proteus-core::package::escrow`)
  - [x] Micro-task 22.1.3: Studio Interactive Sandbox Preview & Client Review Workspace (`proteus-design-studio`)

## Phase 23: Hardware-Sealed Vault & Monotonic Time Lock
- [x] Task 23.1: Monotonic Clock Validation & Anti-Rollback High-Water Mark Engine
  - [x] Micro-task 23.1.1: Monotonic High-Water Mark & Anti-Rollback Watchdog (`proteus-core::security::time_lock`)
  - [x] Micro-task 23.1.2: Hybrid Hardware Key & Operational DPAPI/TPM Sealing Engine (`proteus-core::security::vault`)
  - [x] Micro-task 23.1.3: Studio & Client Zero-Friction Unlock & Security Guardian UI (`proteus-client` / `proteus-design-studio`)

## Phase 24: Resilient Offline Card Payments & SoftPOS Store-and-Forward
- [x] Task 24.1: Autonomous Offline Card Settlement & Cellular Failover Bridge
  - [x] Micro-task 24.1.1: EMV Store-and-Forward (SaF) Offline Capture & Cryptographic Voucher Engine (`proteus-core::fiscal_pos::offline_saf`)
  - [x] Micro-task 24.1.2: Dynamic IRIS Offline QR Generator & Merchant Risk Quota Guard (`proteus-core::fiscal_pos::risk_quota`)
  - [x] Micro-task 24.1.3: Zero-Touch Cellular LAN Bridge & POS Hotspot Failover (`proteus-mobile::hotspot_bridge`)

## Phase 25: Universal Custom Hardware & Open-Source Peripheral Bus
- [x] Task 25.1: Declarative Peripheral Bus & Visual Hardware Automation Engine
  - [x] Micro-task 25.1.1: Multi-Protocol Peripheral Bus for Serial, USB-HID, BLE & MQTT (`proteus-core::hardware::bus`)
  - [x] Micro-task 25.1.2: Declarative Hardware Profile Manifest in `.pr` Packages (`proteus-core::hardware::profile`)
  - [x] Micro-task 25.1.3: Visual Hardware Scratch Nodes in Flow Studio for Relays & Sensors (`proteus-design-studio::flow::hardware_nodes`)

## Phase 26: Proteus Sovereign Appliance & Bootable ISO/IMG Builder
- [x] Task 26.1: Dedicated Appliance Operating System & Kiosk Deployment Engine
  - [x] Micro-task 26.1.1: Bare-Metal Appliance Profile & Immutable Read-Only RootFS Spec (`proteus-core::appliance::manifest`)
  - [x] Micro-task 26.1.2: Turnkey Bootable ISO / IMG Builder Pipeline for x86_64 & ARM64 (`proteus-core::appliance::builder`)
  - [x] Micro-task 26.1.3: Studio 1-Click USB Flasher & Microcontroller Firmware Provisioner (`proteus-design-studio::views::appliance_flasher`)

## Phase 27: Global Multi-Jurisdiction Fiscal Adapter & Universal ERP Bridge
- [x] Task 27.1: Jurisdiction-Agnostic Fiscal Gateway & International ERP Sync Engine
  - [x] Micro-task 27.1.1: Multi-Jurisdiction Fiscal Adapter Specification (`proteus-core::fiscal_adapter::jurisdiction`)
  - [x] Micro-task 27.1.2: Multi-Currency & Universal Decimal Math Ledger (`proteus-core::fiscal_adapter::currency`)
  - [x] Micro-task 27.1.3: Universal Outbound Accounting Webhook & REST Dispatcher (`proteus-core::fiscal_adapter::webhook`)

## Phase 28: Global Internationalization (i18n), Localized Formatting & E.164 Engine
- [ ] Task 28.1: Declarative i18n Dictionary & Global Localization Engine
  - [x] Micro-task 28.1.1: Declarative i18n Dictionary & Translation Resolver (`proteus-core::i18n::dictionary`)
  - [ ] Micro-task 28.1.2: Localized Date, Number & E.164 Phone Formatting Pipeline (`proteus-core::i18n::format`)
  - [ ] Micro-task 28.1.3: Studio Visual Locale Switcher & i18n Translation Manager (`proteus-design-studio::views::i18n_manager`)

Related: [[Board|Kanban Board]] | [[Decisions|Decisions]] | [[../Developer/14 - Proteus BOS Blueprint|Proteus BOS Blueprint]] | [[../Developer/15 - Master Problem Audit & Architectural Solutions|15 - Master Problem Audit (P1-P28)]]


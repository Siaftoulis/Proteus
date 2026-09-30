# Proteus (BOS & CRM Builder)

[![Build & Test Status](https://img.shields.io/badge/tests-305%20passed%20%2F%20100%25-brightgreen)](https://github.com/Siaftoulis/Proteus)
[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-blue)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20Android-lightgrey)](https://github.com/Siaftoulis/Proteus)
[![License](https://img.shields.io/badge/license-Proprietary-red)](LICENSE)

Welcome to **Proteus** — an autonomous, next-generation Business Operating System (BOS) and visual CRM builder engineered from first principles in **100% native Rust and egui**. 

Proteus replaces monolithic legacy ERPs and brittle web-app stacks with ultra-fast, local-first native executables that run offline with zero external dependencies and zero cloud integration tax.

---

## Strategic Vision & Core Philosophy

1. **100% Original Codebase (Zero Copied Boilerplate)**: Every layout engine, scene graph, reconciler, and database routine is designed and written bespoke from first principles.
2. **Zero Mock Data Policy**: Every interface, table, and metric reads and writes to real local SQLite storage (`store.db`), real hardware ports, or live federation endpoints.
3. **Average Joe Principle (Minimalist UX)**: Clean, high-density, low-cognitive-overhead native interfaces designed for frontline shop clerks, technicians, and warehouse dispatchers.
4. **Offline-First & Sovereign**: Immediate sub-millisecond execution with local SQLite WAL caching, background Merkle audit chains, and peer-to-peer LAN replication.

---

## Workspace Architecture (7 Modular Crates)

The project is structured as a cohesive Rust Cargo workspace with zero circular dependencies:

```
project/
├── proteus-core/           # Sovereign Headless Engine (SMLM, Reconciler, Genealogy, Merkle, Outbox)
├── proteus-design-studio/  # Visual IDE & Canvas Builder (Penpot-like Scene Graph, Flow DAG, ESC/POS)
├── proteus-client/         # Front-Desk Terminal (Service Intake, Kanban, RMA, Appointments, Cashier)
├── proteus-web/            # Sovereign Web Hub (Domain Reseller, Hosting Subscriptions, Custom Briefs)
├── proteus-mobile/         # Handheld Android/iOS Touch Companion (44pt touch targets, LAN outbox)
├── auth-server/            # Headless Axum Auth Service (Argon2id, JWT, Sliding-window Rate Limiter)
└── license-server/         # Cryptographic License & Activation Gateway (HMAC-SHA256, Machine-Binding)
```

---

## What Has Been Built & Operational

### 1. Visual Designer & Flow DAG Builder (`proteus-design-studio`)
- **Penpot-like Infinite Canvas**: Infinite pan/zoom, 8-point resize handles, multi-selection, and smart edge/center snapping.
- **Hierarchical Scene Graph**: Deterministic parent-child world-space coordinate accumulation, Z-order layering, and hit testing.
- **ESC/POS Thermal Canvas Viewport**: Dedicated 80mm & 58mm thermal presets with visual character guide margins, serrated paper tear-off cutlines, and direct raw spooling.
- **Interactive Wiring Overlay**: Cubic bezier visual wires connecting buttons, destination screens, and database entities with glowing connection ports and action badges.
- **Visual Flow Automation**: Reactive execution graph with `Trigger`, `Action`, `Condition (IF)`, `Notification`, and `FederationBridge` nodes.

### 2. Frontline Shop Operations & Terminal (`proteus-client`)
- **Service Intake Form**: Instant ticket generation with phone/name validation, device faults, and 1-click ESC/POS receipt generation.
- **Kanban Pipeline**: 6 operational stages (`received`, `in_progress`, `waiting_parts`, `ready`, `delivered`, `cancelled`) with fast text search and drag-and-drop workflow.
- **Ticket Detail Inspector**: Modal editing for technical notes, cost estimation, status updates, thermal reprints, and S/N linkage.
- **Contractor Job-Site Sub-ledger (Καρτέλα Μάστορα)**: Real SQLite tracking of credit balances, materials supplied per construction site, and cash disbursements.
- **Supplier Price Catalog Reconciler**: 1-click CSV/TSV vendor price ingestion with automated schema alignment and direct database updates.

### 3. Component Genealogy, Serial Number Tracking & RMA Hub (`proteus-core`)
- **Full Physical Lifecycle Tracking**: Tracks parts through `SupplierIntake` ➔ `WarehouseStock` ➔ `InstalledInCustomerDevice` ➔ `RmaClaimInitiated` ➔ `RmaReplacedBySupplier` ➔ `CreditNoteIssued`.
- **Dynamic Real-Time Warranty Countdown**: Automatic status computation (`WarrantyStatus::Valid { days_remaining }` vs `WarrantyStatus::Expired { days_expired }`).
- **Embedded RMA & S/N Inspector**: 1-click warranty claims, replacement serial binding, and chronological event audit trails directly within ticket details.
- **Centralized RMA Hub (`genealogy_rma.rs`)**: Dedicated queue monitoring active vendor warranty claims across all branches.

### 4. SMLM & Universal Database Harmony (`proteus-core`)
- **Small Language Context Model (SMLM)**: Fast, offline, content-driven data profiler:
  - Greek Tax ID (ΑΦΜ modulo 11 validation).
  - E.164 international phone formats.
  - GS1-128 barcodes, EAN-13, and SSCC container codes.
  - IBAN accounts, vehicle VINs, and IMO identifiers.
- **Multi-Lingual Semantic Ontology**: Automatic matching across Greek, Greeklish, English, and ERP acronyms (`pelatis`, `customer_name`, `cust_nm` ➔ `CustomerName`).
- **Entry-by-Entry Reconciler**: Composite natural key matching, Last-Write-Wins (LWW) conflict resolution, and automated database federation.

### 5. Cryptography, Security & Verification
- **High-Concurrency SQLite Storage**: `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, and `PRAGMA busy_timeout = 5000;` preventing multi-threaded write lock crashes.
- **Tamper-Proof Merkle Audit Backlog**: Cryptographic SHA-256 hash chains verifying transaction integrity.
- **Encrypted Backup Engine**: Argon2id KDF + XChaCha20-Poly1305 encryption for local/cloud snapshot archives (`.bak`).
- **Network Hardening**: Protected LAN receiver (port 7443) with strict payload capping and DoS mitigation.
- **Unbreakable Verification Pipeline (`verify_pipeline.bat` / `.ps1`)**: Automated 6-stage verification gate enforcing 100% test passes, storage safety, zero mock data, and compilation integrity.

---

## Automated Verification Pipeline

To ensure the codebase never breaks, execute the automated 6-gate verification pipeline:

```powershell
# Run the complete verification suite
.\scripts\verify_pipeline.ps1

# Or via double-click on Windows:
scripts\verify_pipeline.bat
```

### Pipeline Verification Gates:
1. **Gate 1**: Static Type & Compilation Audit (`cargo check --workspace --all-targets`).
2. **Gate 2**: Security & Storage Pragma Audit (`WAL` mode + `busy_timeout = 5000`).
3. **Gate 3**: Zero Mock Data Policy Audit (scans views for forbidden mock arrays).
4. **Gate 4**: 100% Automated Test Suite (`cargo test --workspace` — **304/304 passing**).
5. **Gate 5**: Executable Binary Artifact Compilation (`proteus-client`, `proteus-design-studio`).
6. **Gate 6**: Source Code Modularity & Line Threshold Audit.

---

## Quick Start & Running Locally

### Prerequisites
- [Rust 1.80+](https://rustup.rs/) (stable toolchain)
- Windows 10/11, macOS, or Linux

### Launching Applications

```bash
# 1. Launch Proteus Design Studio (Visual IDE & Flow Canvas)
pds.bat
# or
cargo run -p proteus-design-studio

# 2. Launch Proteus Client (Front-Desk Shop & POS Terminal)
launch.bat
# or
cargo run -p proteus-client

# 3. Launch Proteus Web Portal & Marketplace
cargo run -p proteus-web

# 4. Launch Entire Ecosystem in Background
launch_ecosystem.bat

# 5. Stop All Background Ecosystem Services
stop_ecosystem.bat
```

---

## Roadmap & Upcoming Milestones

- [x] **Universal Database Harmony & SMLM Engine** (Completed)
- [x] **Contractor Job-Site Sub-ledger & Supplier Reconciler** (Completed)
- [x] **Serial Number Genealogy & RMA Tracking Hub** (Completed)
- [x] **Rebranding & Identity Unification** (`proteus-core` / `proteus-design-studio`) (Completed)
- [x] **Unbreakable Quality & Security Verification Pipeline** (Completed)
- [x] **Digital Shipping Note & Dispatch Companion (`shipping_note`)** (Completed)
- [ ] **Van Sales & Mobile Sign-on-Glass**: Mobile on-the-road delivery signature capture and Bluetooth thermal printing.
- [ ] **Vendor-Managed Inventory (VMI) & Consignment Tracking**: Automated vendor replenishment signals on stock threshold breach.
- [ ] **Cold Chain HACCP Telemetry**: Automated temperature/humidity threshold logging with Merkle audit verification.

---

## License

Proprietary — All rights reserved. Designed and developed by **Siaftoulis / Proteus BOS Core Team**.

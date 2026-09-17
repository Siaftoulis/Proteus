# PROTEUS ECOSYSTEM: MASTER AGENT CONTEXT & ARCHITECTURAL BLUEPRINT
> **Version:** 1.0 (Current Status: September 2026 | Milestone Target: May 2027)  
> **Purpose:** Standalone, complete context prompt for AI coding agents and engineers. Copy-paste ready.

---

## 1. Executive Summary & Core Philosophy

### 1.1 What is Proteus?
**Proteus** (Proteus BOS / Proteus Ecosystem) is a **100% Native (Desktop & Mobile)** business operating system, custom CRM/ERP builder, and decentralized services marketplace. It is built to replace legacy browser-based bloatware (Salesforce, HubSpot) and monopolistic, predatory ERP giants (**SAP**, Oracle, Microsoft Dynamics) for small, medium, and large retail/service enterprises.

### 1.2 Core Value Proposition & Anti-SAP Stance
1. **The "Average Joe" Principle (Zero Cognitive Friction):**
   - Clean, luxury dark native aesthetic (obsidian `#0E0F12`, cards `#161820`, borders `#242836`).
   - Touch-friendly, zero clutter. Counter operators can complete an intake or transaction in **under 30 seconds with 3 clicks**.
2. **100% Native Rust & Extreme Efficiency:**
   - No Electron, no browser overheads, no JVM/ABAP bloat.
   - GPU-accelerated rendering at 60 FPS via `egui 0.31`.
   - Client runtime compiles to a single standalone executable of only **~5.0 MB** using under 45 MB RAM.
3. **Offline-First & Zero-Privilege Execution:**
   - Stores run uninterrupted during network outages using local-first SQLite (`%APPDATA%\Proteus\data\store.db`).
   - Requires zero Windows Administrator privileges to install or run.
4. **Platform-as-a-Protocol (Headless Operational Model):**
   - The founder and core team **do not** provide direct client implementations or end-user support.
   - The platform supplies the native software runtimes, database drivers, and marketplace infrastructure.
   - Configuration, database schemas, and on-site support are decentralized to an army of certified independent partners (PCD, PCSS, PCDS) via the internal marketplace.
5. **Anti-Extortion Pricing (Cost-Plus Cloud, Zero Seat-Tax):**
   - 98% cheaper than SAP (TCO of ~14.000€ vs ~1.350.000€ for a 30-store chain over 3 years).
   - Transparent, bracketed self-hosted licensing (7.99€ base up to 199€/mo flat cap).
   - Extra cloud seats scale purely on actual server compute costs (1.00€–1.50€/seat), eliminating predatory per-user penalties.

---

## 2. Technical Architecture: The 3 Product Pillars

The project is structured as a **Cargo Workspace** with 6 bespoke crates:

```
project/
├── Cargo.toml                  # Workspace manifest (members: crm-core, crm-ui, proteus-client, proteus-web, auth-server, license-server)
├── crm-core/                   # Shared backend engine (DB, crypto, print, roles, audit, sync)
├── crm-ui/                     # Pillar 1: Proteus Designer (Visual canvas & studio .exe)
├── proteus-client/             # Pillar 2: Proteus Client (Shop counter runtime .exe)
├── proteus-web/                # Pillar 3: Web Portal & Marketplace API (Axum REST backend)
├── auth-server/                # Microservice: JWT, Argon2id, Google OAuth
└── license-server/             # Microservice: Machine activation, seat limits
```

### 2.1 Pillar 1: Proteus Designer (`crm-ui`)
- **Nature:** 100% Free native desktop visual builder (Windows, macOS).
- **Infinite GPU Canvas:** Smooth pan & zoom (0.1x to 5.0x), grid snap, freeform layout.
- **Node Tree & Scene System:** Visual containers, buttons, inputs, tables, cards, and 1-click pre-styled composite blocks (*KPI Metric Card*, *Intake Form Card*).
- **Bespoke HUD Controls:** Selection handles, floating monospace dimension pills (`W × H`), 1-click Quick Alignment bar (Left, Center, Right, Top, Middle, Bottom), curated 10-swatch luxury palette.
- **Flow Engine:** Visual node-wiring canvas for event triggers, actions, and conditional business logic.
- **Data Viewer:** Direct SQLite data inspection and editing.
- **Project Storage:** Saves locally as declarative `.prproj` project files. Local export to `.pr` is gated to route delivery through the marketplace.

### 2.2 Pillar 2: Proteus Client (`proteus-client`)
- **Nature:** Production runtime executable for shop counters, store branches, and field technicians.
- **Granular RBAC & Dynamic TopBar:**
  - `👑 CEO`: Full access to all subsystems, audit logs, and settings.
  - `🎧 Customer Service`: Streamlined to Intake, Pipeline, and Appointments (no technical margins or developer schema clutter).
  - `🛠 Technician`: Laboratory Kanban pipeline and technical notes.
  - `📈 Sales Consultant`: Client contracts, appointments, and price quotes.
  - `💻 Developer`: Schema designer, additive migrations, raw data inspection.
  - *Dynamic tab filtering:* Tabs re-render instantly upon role selection with automatic fallback navigation.
- **Key Operational Screens:**
  1. *Screen 1 (Intake / Νέα Παραλαβή):* Sub-30s device intake, customer phone validation, auto-generated `#Ticket` number, direct print trigger.
  2. *Screen 2 (Kanban Pipeline / Ροή Επισκευών):* 6 stages (`Received`, `InProgress`, `WaitingParts`, `Ready`, `Delivered`, `Cancelled`), live text search, fast status progression buttons.
  3. *Screen 3 (Ticket Detail Modal):* Diagnostics notes, repair cost update, status switcher, reprint receipt.
  4. *Screen 4 (Hardware ESC/POS Printing & Settings):* Direct RAW pass-through via native Win32 Spooler (`winspool.drv`), 58mm/80mm width toggle, shop header/footer customization, hardware test print.
  5. *Screen 5 (Activity Timeline & Audit Trail):* Monotonic UUIDv7 event logs, role-colored badges, relative human timestamps ("μόλις τώρα", "πριν 5 λεπτά"), collapsible JSON diff inspector.
  6. *Screen 6 (Appointments Subsystem):* Scheduling for customer service/sales, input validation, automated audit event emission.
  7. *Screen 7 (IT Support Dashboard):* System health checks, 6-digit Remote Session PIN generation, connected shop roster, SLA contract monitor.
  8. *Screen 8 (Specialist Dashboards):* Intake speed benchmarking (<30s target), CSAT feedback, price quote builder, Tier commission tracker.
  9. *Screen 9 (Developer Studio):* Visual entity inspector, additive-only migration runner (`ALTER TABLE ADD COLUMN`), enterprise work orders.

### 2.3 Pillar 3: Proteus Web Hub & Marketplace (`proteus-web`)
- **Nature:** Axum REST API backend powering the web portal, anti-bypass compiler gate, and escrow.
- **Anti-Bypass Compiler Gate:** Validates that client payment is funded in platform escrow before compiling and delivering `.pr` packages to the client runtime.
- **10-Tier Sliding Commission:**
  - Tier 0 (Uncertified): 50% platform take (50/50 split).
  - Tier 1: 40%, Tier 2: 35%, Tier 3: 30% ... down to Tier 10: 4%–5% (covering only Stripe/MoR fees for enterprise agencies).
- **In-Platform Escrow & SLA Contracts:** Two-party digital signatures (shop + technician) with milestone payout release.
- **Pricing Quote Engine:** Real-time mathematical calculation of bracketed licensing and cloud add-ons.

### 2.4 Shared Core Engine (`crm-core`)
- **Database:** Local SQLite engine with custom encryption (**XChaCha20-Poly1305** + **Argon2id KDF**).
- **Upcoming Drivers:** Direct native connectors for **PostgreSQL** and **MySQL**.
- **Hardware Printing:** Native ESC/POS raw command generator and Win32 Spooler interface.
- **RBAC Matrix:** Built-in role permissions enum and enforcement logic.
- **Audit Logger:** Append-only event stream (`SystemEvent`) recording operator identity, role, description, and JSON diffs.
- **P2P Sync Engine:** HTTP sync server, push/pull protocol, Last-Write-Wins (LWW) conflict resolution using monotonic timestamps and UUIDv7.

---

## 3. The Workforce & Certification Model

To scale without hiring thousands of internal support reps, Proteus relies on an accredited network of computer science students, junior developers, and freelance IT integrators:

### 3.1 The 3 Official Certifications
1. **PCD (Proteus Certified Designer):** Visual canvas, design tokens, UI layouts for desktop & mobile.
2. **PCSS (Proteus Certified Systems & DB Specialist):** Relational schema design, SQL migrations, query optimization, PostgreSQL/MySQL data integrity, backup policies.
3. **PCDS (Proteus Certified Deployer / Support Specialist):** POS/terminal installations, local networking, Windows/Mac terminal setups, receipt printer hardware.

### 3.2 Certification Unit Economics
* **One-Off Exam Voucher:** **79€** (COGS ~4€ $\rightarrow$ ~95% gross profit margin).
* **Annual Verified Partner Badge:** **39€ / year** for active marketplace directory listing.
* **All-in-One Certification Bundle:** **149€** one-off.

### 3.3 Quality Assurance & Integrity
* **Sandboxed 1-Click Migrations:** The engine executes migrations in a dry-run test sandbox with automated `.bak` rollback snapshots prior to applying schema changes.
* **Reputation System:** 1–5 star client reviews. Ratings dropping below 4.2 lead to automatic badge suspension and re-testing requirements.

---

## 4. Financial Architecture & Revenue Streams

### 4.1 Marketplace Take-Rates
* **Implementation Services & Customization Gigs:** **18%** platform take-rate (82% to certified partner).
* **Monthly Support Retainers:** **18%** recurring take-rate.
* **Ready Templates (Digital Goods):** **30%** platform take-rate.

### 4.2 Proteus Core License (Self-Hosted / On-Premise)
*Bracketed Tiers (Predictable, Fair Pricing):*
* **Base (1–4 users):** **7,99€ / month** flat.
* **5–20 users:** **+1,50€** / seat / month.
* **21–60 users:** **+1,00€** / seat / month.
* **61–150 users:** **+0,60€** / seat / month.
* **150+ users (Enterprise Local Cap):** **199,00€ / month flat** (unlimited local users).
* *Annual billing discount:* 2 months free (10 months paid for 12 months access).

### 4.3 Managed Cloud Add-ons
* **Automated Cloud Backups:** **+9,99€ / month flat** (cold snapshots to Cloudflare R2 / S3, 30-day retention).
* **Managed Live Cloud DB (Multi-Tenant Small/Medium):**
  * $\le$20 users: +15€ / month (Infra cost ~5€ $\rightarrow$ 66% margin).
  * 21–100 users: +45€ / month (Infra cost ~12€ $\rightarrow$ 73% margin).
  * 101–500 users: +120€ / month (Infra cost ~28€ $\rightarrow$ 76% margin).
* **Enterprise Dedicated Cloud (Single-Tenant Managed IaaS - e.g. 5,000 seats):**
  * Never flat-capped: **1,50€ – 2,00€ / seat / month**.
  * 5,000 seats = **7.500€ – 10.000€ / month**.
  * Dedicated Bare-Metal Cluster cost (Hetzner EPYC/Ryzen HA Postgres + sync nodes): **~500€ – 600€ / month**.
  * **Net profit:** **~6.900€ – 9.400€ / month (~85% margin)** per enterprise client.

---

## 5. Multi-Store Architecture (The "Public / Retail Chain" Model)

```
                           ┌───────────────────────────────┐
                           │    ENTERPRISE OWNER / HQ      │
                           │   (Κεντρική Διοίκηση / CEO)   │
                           │  - Καθορισμός Quotas Αδειών   │
                           │  - Global Inventory & P&L     │
                           │  - Master Tamper-Proof Audit  │
                           └───────────────┬───────────────┘
                                           │
         ┌─────────────────────────────────┴─────────────────────────────────┐
         ▼                                                                   ▼
┌─────────────────────────────────┐                         ┌─────────────────────────────────┐
│   STORE 01: ΣΥΝΤΑΓΜΑ (PUBLIC)   │                         │     STORE 02: ΤΣΙΜΙΣΚΗ ΘΕΣ/ΚΗ   │
│   Store Director (Διευθυντής)   │                         │   Store Director (Διευθυντής)   │
│   Allocated Quota: 25 Seats     │                         │   Allocated Quota: 18 Seats     │
└────────────────┬────────────────┘                         └────────────────┬────────────────┘
                 │                                                           │
   ┌─────────────┼─────────────┬─────────────┐                 ┌─────────────┼─────────────┐
   ▼             ▼             ▼             ▼                 ▼             ▼             ▼
[ΑΠΟΘΗΚΗ]   [ΤΗΛΕΦΩΝΙΑ]   [ΒΙΒΛΙΑ/POS]  [SERVICE]         [ΑΠΟΘΗΚΗ]     [ΤΑΜΕΙΟ]      [SERVICE]
Barcodes    IMEI Check    ISBN Scan     Kanban            Barcodes      POS Speed     Kanban
Stock-In    Carrier Plans Quick Pay     Repairs           Stock-In      Receipts      Repairs
4 Seats     8 Seats       10 Seats      3 Seats           3 Seats       10 Seats      5 Seats
```

1. **Delegated Provisioning:** HQ allocates a seat quota (e.g. 25 seats) to a Store Director. The Store Director creates department accounts without IT tickets.
2. **Departmental Specialization:**
   - *Warehouse:* Barcode stock-in/out, inter-store transfers.
   - *Mobile / Tech:* IMEI serial tracking, carrier contracts, cross-store stock lookups in <1s.
   - *Books & Stationery / Quick POS:* ISBN scan, rapid checkout in <5s, loyalty points.
   - *Service Lab:* Digital repair ticket, barcode label printing, 6-stage Kanban.
3. **Cryptographic Tamper-Proof Audit Ledger (Merkle Hash Chain):**
   - Each event in `tamper_proof_audit_backlog` stores `SHA-256(previous_hash + payload + timestamp)`.
   - Modifying past records breaks the chain, triggering an immediate *Integrity Panic* that isolates the compromised store and alerts HQ.

---

## 6. Strict Development Rules (AGENTS.md)

1. **100% Bespoke Original Codebase (Strict Rule):**
   - Under no circumstances copy code blocks, templates, or implementations from third-party repositories.
   - Every module, struct, UI component, and database routine must be designed natively from first principles.
   - Only standard crates declared in `Cargo.toml` (`eframe`, `egui`, `rusqlite`, `serde`, `chrono`, `uuid`, `axum`) via public APIs.
2. **File Size Limit:** Keep source files modular and strictly **under 400 lines** wherever practical.
3. **100% Passing Test Suite & Zero Warnings:**
   - All tests across all crates must pass (`cargo test --workspace`). Currently **121 / 121 tests pass (100% green)**.
   - Clean compilation with **0 compiler warnings**.

---

## 7. Master Execution Roadmap (Target: May 2027)

### Phase 1 — Core Construction (Current $\rightarrow$ May 2027)
- [x] Shared Rust Core (`crm-core`): SQLite, crypto, audit logs, roles, ESC/POS printer spooler.
- [x] Standalone Shop Runtime (`proteus-client`): Intake, Kanban, appointments, audit timeline, specialist dashboards.
- [x] Web Backend Engine (`proteus-web`): Pricing brackets, certification tiers, escrow contracts, compiler gate.
- [ ] Direct Database Drivers: Direct connection to **PostgreSQL** and **MySQL** in `crm-core`.
- [ ] Multi-platform static libraries (`.dll`, `.dylib`, C-bindings/NDK for iOS/Android).
- [ ] Sandboxed 1-click migration runner with automated `.bak` rollback snapshot.

### Phase 2 — Design Partners Pilot
- Pilot deployments in 2–3 commercial repair/retail businesses with Lifetime Licenses.
- Real-world counter stress testing, weekly UX feedback, published case studies.

### Phase 3 — Marketplace & Hub Public Launch
- Launch Web Portal storefront.
- Open certification exams (PCD, PCSS, PCDS) to CS students and freelance integrators.
- Enable in-platform escrow contracts for client implementation gigs.

---

## 8. Key Repository Paths & Documentation Quick-Links
- **Rust Workspace:** `project/Cargo.toml`
- **Core Engine:** `project/crm-core/src/` (`lib.rs`, `audit.rs`, `roles.rs`, `tickets.rs`, `printer.rs`, `encryption.rs`, `db.rs`, `sync.rs`)
- **Designer Tool:** `project/crm-ui/src/` (`main.rs`, `renderer.rs`, `inspector.rs`, `flow.rs`, `scene/`)
- **Client App:** `project/proteus-client/src/` (`main.rs`, `app.rs`, `views/` [`intake.rs`, `pipeline.rs`, `ticket_detail.rs`, `appointments.rs`, `audit_log.rs`, `settings.rs`, `support.rs`, `dashboards.rs`, `developer.rs`])
- **Web Backend:** `project/proteus-web/src/` (`main.rs`, `marketplace.rs`, `portal.rs`, `contracts.rs`)
- **Vault Documentation:** `vault/Developer/` (Docs 01 to 21, including `17 - Ecosystem Standards`, `18 - Financial Model`, `19 - RBAC & Audit`, `20 - Anti-SAP Manifesto`, `21 - Master Brief`)
- **Strategic CEO Roadmap:** `project/ceo_roadmap.md`
- **Session History:** `vault/Agent/Session Log.md` & `vault/Agent/Board.md`

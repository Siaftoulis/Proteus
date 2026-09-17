# Proteus Ecosystem — Strategic CEO Roadmap
### Target Milestone: May 2027

---

## 1. Product Vision & Operational Model
* **"The Native Business OS & Headless Marketplace Protocol."**
* **Zero Browser Overhead & 100% Data Ownership:** Native Rust engines for Windows, macOS, Android, and iOS. Local-first SQLite caching with direct enterprise connectors for PostgreSQL & MySQL.
* **Headless Platform Architecture:** The core team builds software and protocols; customization, database setups, and ongoing support are delivered by a decentralized army of certified independent partners (PCDA, PCD, PCSS, PCDS) via the internal marketplace.
* **Anti-Monopoly Stance:** 98% cheaper than legacy ERPs (SAP/Oracle), zero seat-tax extortion, 1-click cloud deployments, and luxury dark UX.

---

## 2. The Three Product Pillars
| Pillar | Target Platform | Licensing / Economics |
|---|---|---|
| **1. Proteus Designer** | Native Desktop (Windows, macOS) | **100% Free** download & local export (`.prproj`). Zero friction for designers. |
| **2. Proteus Client** | Native Desktop & Mobile (Win, Mac, iOS, Android) | **Bracketed Core License** (7.99€ base $\rightarrow$ 199€ flat cap). |
| **3. Proteus Web Hub** | Web Portal & Backend (Rust/Axum) | **Marketplace Take-Rate** (18% services, 30% templates, 79€ certifications). |

---

## 3. Financial Architecture & Unit Economics
* **Core On-Premise License (Bracketed Tiers):**
  * Base (1–4 users): **7,99€ / month** flat
  * 5–20 users: **+1,50€** / seat / month
  * 21–60 users: **+1,00€** / seat / month
  * 61–150 users: **+0,60€** / seat / month
  * 150+ users (Enterprise Local Cap): **199,00€ / month flat**
* **Managed Cloud Add-ons:**
  * Automated Cloud Backups: **+9,99€ / month flat** (R2/S3 cold snapshots)
  * Managed Live Cloud DB: $\le$20 users: +15€/mo; 21–100 users: +45€/mo; 101–500 users: +120€/mo
  * Enterprise Dedicated Cloud (Single-Tenant IaaS): **1,50€ – 2,00€ / seat / month** (~85% profit margin)
* **Marketplace Take-Rate:**
  * Implementation Gigs & Customization: **18%** take-rate (82% to partner)
  * Monthly Support Retainers: **18%** recurring take-rate
  * Digital Goods / Templates: **30%** take-rate

---

## 4. Certification & Workforce Model
* **PCDA (Proteus Certified Data/Business Analyst):** The primary client-facing architect. Requirements elicitation, legacy data mapping, automated schema inference (JSON/CSV), GS1-128 barcode parsing, business rule definitions, and Certified Data Contract sign-off before PCD/PCSS touch the system.
* **PCD (Proteus Certified Designer):** Visual canvas, design systems, UI layouts.
* **PCSS (Proteus Certified Systems & DB Specialist):** PostgreSQL/MySQL schemas, SQL migrations, data integrity.
* **PCDS (Proteus Certified Deployer / Support Specialist):** POS/terminal deployments, local networking, terminal setup.
* **Economics:** 79€ exam voucher (~95% margin), 39€/year verified badge renewal, 149€ all-in-one bundle.
* **Quality Assurance:** Sandboxed 1-click migrations with dry-run snapshots and reputation ratings (suspension if < 4.2).

---

## 5. Master Execution Roadmap (Target: May 2027)

### Phase 1 — Core Construction (Through May 2027)
- [x] Shared Rust Core (`crm-core`): SQLite persistence, encryption, audit logs, roles matrix, ESC/POS spooler.
- [x] Standalone Shop Runtime (`proteus-client`): Intake, Kanban pipeline, appointments, audit timeline, specialist dashboards.
- [x] PCDA Analyst Pipeline & Connector (`crm-core` & `proteus-client`): Schema inference, GS1-128 decoder, declarative rules engine, internal event bus, Analyst Studio.
- [x] Web Backend Engine (`proteus-web`): Pricing brackets, certification tiers, escrow contracts, compiler gate.
- [ ] Direct Database Drivers: Direct connection to **PostgreSQL** and **MySQL** in `crm-core`.
- [ ] Mobile/Multi-platform compilation: Static library (`.dll`, `.dylib`, C-bindings/NDK for iOS/Android).
- [x] Sandboxed 1-Click Migrations: Dry-run execution runner with automated `.bak` rollback snapshot (`crm-core::migrations` & `proteus-client::views::developer`).

### Phase 2 — Design Partners Pilot
- Deploy to 2–3 selected retail / service repair businesses with Lifetime Licenses.
- Intensive stress testing in daily production counters.
- Weekly UX refinement sprints and real-world case studies/testimonials.

### Phase 3 — Marketplace & Hub Public Launch
- Launch Web Portal storefront and template marketplace.
- Open certification exams (PCD, PCSS, PCDS) to computer science students and freelancers.
- Enable in-platform escrow contracts for client implementation gigs.

# Proteus (CRM-Builder / BOS) — Master AI Agent Context & Prompting Guide

## 1. Project Identity & Strategic Mission
- Project Name: Proteus (CRM-Builder / BOS).
- Core Nature: Autonomous, sovereign Business Operating System (BOS) and visual CRM builder engineered in 100% native Rust and egui.
- Strategic Objective: Displace legacy Greek and EU ERPs (SoftOne, Epsilon Net, Impact) and eliminate recurring SaaS subscription taxes (€3,000–€7,000/yr per business) through zero-cost direct AADE myDATA REST API integration, Ergani II digital work card compliance, local-first offline SQLite architecture, and native hardware automation.
- Deployment Formats: Native desktop binaries (Windows/Linux/macOS), handheld mobile touch companions (Android), and sovereign web/API services.

## 2. Workspace Architecture (Cargo Workspace — 7 Crates)
- Root: `c:\Users\PDS_Dev\1_Production\Projects\CRM-Builder`
- Workspace Cargo Root: `project/Cargo.toml`
- Workspace Crates:
  1. `proteus-core`: Headless sovereign business engine. Contains DataEngine, SQLite storage with WAL mode, SMLM (Small Language Context Model for Greek AFM modulo-11, GS1-128, IBAN, VIN), WMS 3D bin-packing, cold chain HACCP telemetry with Merkle audit chains, component genealogy and RMA tracking, contractor sub-ledgers (Καρτέλα Μάστορα), direct AADE myDATA sync, LAN P2P mesh replication, thermal printer drivers (ESC/POS, TSPL/ZPL), and flow DAG execution engine.
  2. `proteus-design-studio`: Visual IDE and CRM canvas builder in Rust and egui. Features Penpot-like infinite canvas, 8-point resize handles, hierarchical scene graph (ElementNode), multi-selection, snap-to-grid/edges, ESC/POS thermal receipt canvas viewport (80mm/58mm), widget palette, and visual flow DAG wiring.
  3. `proteus-client`: Frontline shop terminal and POS. Features repair ticket service intake, 6-stage Kanban repair pipeline, ticket inspector modal, contractor site sub-ledger, supplier CSV price reconciler, and live HACCP telemetry monitor.
  4. `proteus-web`: Web hub and client portal built with Axum. Handles domain/hosting management, marketplace modules, digital briefs, client portal, telemetry UI, onboarding, and legal/compliance pages.
  5. `proteus-mobile`: Handheld touch companion for Android and mobile terminals. Features sign-on-glass vector signature pad, van sales delivery taskboard, offline outbox, and Bluetooth 58mm thermal receipts.
  6. `auth-server`: Headless authentication service with Argon2id password hashing, JWT tokens, and sliding-window rate limiting.
  7. `license-server`: Cryptographic licensing gateway with HMAC-SHA256 tokens, machine hardware binding, and offline lease verification.

## 3. Seven Non-Negotiable Rules & Dogmas (from AGENTS.md)
1. 100% Original Codebase: Zero copied boilerplate or external repo code. Bespoke architecture written from first principles using standard Cargo crates via public APIs only.
2. Zero Mock Data Policy: No hardcoded dummy arrays or fake mock views. All data must read and write to real SQLite (`store.db` / DataEngine) or real endpoints.
3. Average Joe Principle (Minimalist UX): Clean, high-density, low cognitive overhead interfaces designed for shop clerks and warehouse staff. No visual clutter.
4. Strict Hierarchical Micro-Tasks ("νια-νια" 1.1.1, 1.1.2): Implement strictly ONE atomic micro-task per turn, verify it, and report. Never bundle multiple tasks or write giant files in one turn.
5. Discord-Compatible Output Format: Pure, minimalist Discord-compatible text. No tables, no complex links, no unnecessary emojis.
6. Compliance & Security by Design: Adherence to GDPR, Greek Law 4624/2019, ISO/IEC 27001, SOC 2 Type II, HIPAA, Ergani II, AADE myDATA direct API. Encrypted backups via XChaCha20-Poly1305, Argon2id KDF, and Merkle SHA-256 audit logs.
7. Zero Interactive Question Modals: Never call blocking interactive question tools (e.g. ask_question). Autonomously pick the optimal path and execute to completion.

## 4. Current Implementation State & Key Subsystems
- Database: High-concurrency SQLite WAL mode (`PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;`).
- Flow Engine & Automation: Node-based DAG with Trigger, Action, Condition (IF), Notification, and FederationBridge nodes.
- myDATA & Tax: Direct REST API connection to AADE (myAADE credentials). Direct XML/JSON transmission, QR code generation, zero per-document fee.
- Hardware Integration: Native raw ESC/POS thermal receipt printing, TSPL label printing, Electronic Shelf Labels (ESL) over BLE/radio, RFID/near-field tote intake.
- Cold Chain & Logistics: Real-time sensor telemetry, excursion breach severity checks, HACCP certificate generation with cryptographic Merkle proof chains.
- WMS & Spatial Storage: 3D bin-packing algorithm, ergonomic golden-zone shelf slotting, carton container seals.
- Verification Pipeline: 6-gate audit (`scripts/verify_pipeline.ps1` / `scripts/verify_pipeline.bat`) validating compilation, WAL pragmas, zero mock data, 640+ automated tests, binary build, and file size thresholds.

## 5. Master Prompt for Incoming AI Agents
Below is the exact prompt to feed to any new AI agent so it operates with 100% alignment from step zero:

```text
You are Antigravity, working on Proteus (CRM-Builder / BOS) — an autonomous, sovereign Business Operating System and visual CRM builder engineered in 100% native Rust (egui, rusqlite, serde, axum).

CORE RULES (STRICTLY ENFORCED):
1. 100% Original Code: Zero copied code or external snippets. Bespoke first-principles architecture using standard crates only.
2. Zero Mock Data: Under NO circumstances use mock or fake data arrays. Always read/write to real SQLite storage (`store.db` / `proteus-core::DataEngine`), real endpoints, or real hardware APIs.
3. Minimalist UX (Average Joe Principle): High-density, sleek, intuitive UI for frontline staff. No visual clutter.
4. Hierarchical Micro-Tasks: Break tasks into micro-steps (1.1.1, 1.1.2). Implement strictly ONE atomic micro-task per turn, verify it with targeted tests (`cargo test -p <crate>`), and report.
5. Discord Output Format: Output pure, minimalist text compatible with Discord. Do NOT use markdown tables or complex links. No unnecessary emojis. Keep communication raw, dense, and actionable.
6. Zero Interactive Question Modals: NEVER call interactive blocking question tools. Decide autonomously and execute directly.
7. Compliance Standards: Strict adherence to GDPR, Greek Law 4624/2019, ISO 27001, SOC 2, HIPAA, Ergani II, and AADE myDATA direct REST API integration (zero third-party provider fees).

WORKSPACE STRUCTURE:
- project/proteus-core: Headless engine (DataEngine, SQLite, SMLM, WMS, Cold Chain, RMA, Merkle audit, myDATA, P2P mesh).
- project/proteus-design-studio: Visual IDE and CRM canvas builder (egui infinite canvas, scene graph, flow DAG, ESC/POS viewport).
- project/proteus-client: Frontline shop terminal (service intake, 6-stage Kanban, contractor sub-ledger, supplier reconciler).
- project/proteus-web: Web portal and marketplace (Axum).
- project/proteus-mobile: Android/touch companion (sign-on-glass, van sales).
- project/auth-server & project/license-server: Headless auth and cryptographic license gateway.

When assigned a task, declare the micro-step number (e.g. 1.1.1), execute the exact file modifications cleanly, verify with targeted tests, and report results concisely in Discord bullet format.
```

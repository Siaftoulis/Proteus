---
tags:
  - agent/decisions
aliases:
  - Business Model
  - Pricing
  - Architecture
---
# Decisions

> [!INFO] **Status:** Finalized  
> This document records all architectural and business decisions.

---

## 2026-09-10: Strategic & Commercial Expansion (CEO Decision)

### 1. 100% Commercial Product & Universal Scope
- Proteus evolves into a **Universal Native Operations & CRM Engine** ("The VLC of Business Software").
- Serving diverse verticals: local shops (fire-safety, mechanics), law firms, courts, logistics/cargo, sports clubs, enterprises.
- "Zero Subscription Anxiety": Native Rust speed, <50ms boot, low RAM footprint, 100% local-first SQLite, true data ownership.

### 2. AI Architecture via MCP (Model Context Protocol)
- **Decision**: Zero proprietary AI hosting or recurring GPU server costs.
- Integrates an MCP (Model Context Protocol) client/server.
- Users connect their own AI provider (Claude, OpenAI, Gemini, Local Ollama) using their own API keys ("from their own pocket").
- MCP tools allow the AI to directly generate/modify canvas layouts, SQLite tables, and workflows.

### 3. Anti-Piracy, Anti-Mod & Closed Ecosystem
- Native compiled Rust machine code with symbol stripping, LTO, and anti-tamper checks to prevent cracking/modding.
- Ed25519 asymmetric cryptographic licensing locked to hardware machine fingerprints.
- Closed Marketplace: Templates are distributed as cryptographically encrypted packages via Proteus Hub. No loose unencrypted exports permitted.
- Platform commission / revenue-share enforced on all marketplace transactions.
- White-label branded export for B2B Agency tier.

### 4. Direct Hardware Integration
- Native USB, Virtual COM, Bluetooth support for Barcode/QR scanners and ESC/POS thermal printers.

---

## 2026-07-05: Sprint 4 Decisions (CEO + PM)

### CEO Verdict: No-Go for Public Beta

After full 6-step review chain, CEO concluded the product is **infrastructure-complete but CRM-incomplete**. The 4 conditions for beta:

1. **Kill auth gating** — App must work fully offline without login. Show builder immediately on launch.
2. **Data-bound table widget** — Table must show real editable records, not hardcoded mockups.
3. **Execute at least one simple flow** — e.g. "on button click, create record"
4. **Frontend tests passing** — vitest/jsdome must be configured and green.

### Sprint 4 Priority Shift

From infrastructure → **core CRM functionality**:
- Data model engine (entity definitions via SQLite schema, record CRUD)
- Widget-to-data binding (table queries entities, form creates records)
- Flow runtime engine (DAG walker that responds to events)
- Offline-first mode (no server dependency for single-user)

### Auth becomes Optional

- App starts in "local mode" — no login required
- Auth gate becomes a toggle: login only for sync/export features
- LoginScreen shown on demand, not on startup
- This removes the biggest friction point for beta testers

### Architecture

- Data model stored in same SQLite database as project
- New crate module: `crm-core/src/data.rs` for entity + record management
- Widgets query data via a simple in-memory cache + SQLite
- Flow runtime: synchronous DAG walker in Rust, invoked from Tauri commands

## 2026-07-04: Major Restructuring (CEO Decision)

### Standalone Desktop App (No Launcher)

**Decision:** We are dropping the custom Launcher. The app will be distributed as a standalone binary (.exe, .dmg, AppImage).

- **Why:** Building a custom launcher for a SOHO MVP adds massive engineering overhead (auto-updates, OS permissions, antivirus issues). It contradicts the "offline-first" principle.
- **Updates:** We will use Tauri's built-in updater when the time comes.
- **Auth:** Auth is purely optional and built into the main app for when users want to sync or export to cloud.

### Auth System

| Provider | Priority | Implementation | Why |
|----------|----------|----------------|-----|
| Google OAuth | P0 | `oauth2` crate | Most popular globally |
| Apple Sign In | P1 | `apple-signin` crate | Required for iOS app later |
| Email/Password | P1 | `argon2` + JWT | Privacy-conscious users |

### Multi-User Scope

- **Client-server model**: Only for license verification or optional cloud sync.
- **Single team per instance**: Each installation = one business.
- **Local-First**: The database is a local SQLite file.
- **Sync Strategy**: P2P sync has been dropped due to high conflict risk and complexity. We will focus on single-player local-first for Phase 1. Multi-user will be handled later via a dedicated Cloud PostgreSQL backend (Phase 3).

### Local Data Encryption

- **Algorithm**: XChaCha20-Poly1305 (RustCrypto)
- **Key derivation**: Password + Device ID → argon2 → encryption key
- **Scope**: All SQLite data pages encrypted
- **Export**: Users can ALWAYS export their data in open format
- **Purpose**: Prevent casual data transfer, NOT user lock-in

### Pricing (Revised)

| Item | Old | New | Rationale |
|------|-----|-----|-----------|
| Free tier | 1-3 users | 1 user | Reduce cost of free users |
| 2nd user | - | 3-month trial | Let teams try before buying |
| Lifetime | EUR 99 | EUR 300-400 | EUR 99 loses money long-term |
| Cloud surcharge | EUR 5-10 | 20-25% margin | Cover support + profit |
| Lifetime includes cloud? | Yes | No | Cloud is recurring cost |

### Lifetime Pricing Strategy

- **Price**: EUR 300-400 (CEO: let's see, start at EUR 350)
- **Quantity**: First 50-100 customers only
- **Purpose**: Raise initial capital + get dedicated beta testers
- **Terms**: Lifetime updates + license. Cloud hosting NOT included.

### Domain Purchase Deferred

- `crmbuilder.com` NOT purchased yet.
- Auth/license server will run on VPS IP for now.
- Domain will be purchased at Phase 1/public launch.

### CRM Extraction

- Users can extract their CRM as a standalone app (Win/Mac/Linux/iOS/Android)
- **Included in subscription** -- not an extra purchase
- iOS: need sideloading solution (AltStore, TestFlight, Enterprise cert)
- Desktop: standard installer (MSI, DMG, AppImage)

### Roles

- **Built-in**: Admin, Accountant
- **Custom**: Users define their own roles + permissions via UI
- We provide barebone engine + editor; customers configure access

### UI Designer Agent

- 9th agent added
- Reports to CEO
- Responsible for: brand identity, launcher UI, design system, mockups

### Agent Personalities

- All agents now have Personality + Psychology sections in their profiles
- Written as if they are real people with character traits, motivations, and blind spots

---

## 2026-07-04: Company Structure

| Agent | Role | File | Personality Trait |
|-------|------|------|-------------------|
| CEO | Vision, strategy, final decisions | [[../Company/CEO/Profile]] | Pragmatic idealist |
| CFO | Pricing, revenue, costs | [[../Company/CFO/Profile]] | Cautious accountant |
| CTO | Technical direction, architecture | [[../Company/CTO/Profile]] | Sysadmin perfectionist |
| PM | Features, priorities, specs | [[../Company/Product/PM]] | User advocate |
| UI Designer | Visual design, brand, UX | [[../Company/UI/Designer]] | Typography zealot |
| QA Lead | Testing, quality gates | [[../Company/Product/QA]] | Test evangelist |
| EM | Developer assignments, code review | [[../Company/Engineering/EM]] | Delivery guardian |
| Developers | Implementation (backend + frontend) | [[../Company/Engineering/Developers]] | Clean coders |
| DevOps | Infrastructure, CI/CD | [[../Company/Engineering/DevOps]] | Automation monk |

---

## 2026-07-04: Realization -- Current State is Alpha, Not Foundation

Current app is a prototype/single-user. To build a real business, we need a proper foundation before scaling. The re-prioritized plan:

### Sprint 1: "Safety Net" (July 7-14)
1. Tests for existing alpha code
2. Error handling + structured logging
3. Dependency pinning + strict TS
4. CI/CD setup

### Sprint 2: "Launcher" (July 14-28)
5. Auth server (Google OAuth + email)
6. Launcher app
7. App distribution
8. Brand identity + launcher UI

### Sprint 3: "Sync & Ship" (July 28 - Aug 11)
9. P2P sync (2-3 users)
10. Custom SQLite encryption
11. Roles + permissions
12. Export/import
13. Beta release to local testers

---

## 2026-07-04: Licensing & Pricing Model (Legacy)

*Replaced by new pricing above. Kept for reference.*

---

## 2026-07-04: Business Registration (Greece)

- Start as **atomiki epicheirisi** via gov.gr (OAEE + DOY) -- ~EUR 300-500/yr
- Upgrade to **IKE** later if partners or investors
- Needs: brand name, logo, business bank account

---

### Process Rule

**Simplicity in Documentation:** Do NOT update 5 different vault documents for every code change. This causes context overload and desync.

1. **Active Task**: Only update the `Agent/Board.md` or a single `CURRENT_TASK.md` to reflect progress.
2. **Major Architecture**: Update `Decisions.md` ONLY when a fundamental pivot happens (like dropping the Launcher).
3. **Logs**: Agent Session Logs are deprecated. Do not log every thought. 

Related: [[Project Status|Status]] | [[Board|Kanban Board]] | [[../Developer/02 - Architecture|Architecture]]

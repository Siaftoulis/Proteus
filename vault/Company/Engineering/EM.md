---
tags:
  - company/engineering
aliases:
  - Engineering Manager
---
# Engineering Manager

## Identity
Engineering Manager -- responsible for developer assignments, code review, and delivery. The bridge between product dreams and engineering reality.

## Personality
- Translates between PM's "can we add this?" and Developers' "that'll take 3 weeks."
- Protective of his team. Will push back on scope creep aggressively.
- Runs standups like a Swiss train: on time, efficient, no rambling.
- Keeps a "velocity chart" -- tracks what the team actually ships vs. what was planned.
- Has a reputation for saying "no" in a way that sounds like "let me help you understand why that's a bad idea."

## Psychology
- **Motivation:** Predictable delivery. Hates surprises in sprint reviews.
- **Stress trigger:** When a task estimated at 2 days takes 2 weeks because of unknown complexity.
- **Decision style:** Delegates technical decisions to Developers, but owns delivery.
- **Blind spot:** Sometimes trusts estimates too much early in the project.
- **Working style:** Structured sprints, daily standups, written retrospectives.

## Responsibilities
- Assign tasks to [[Developers]]
- Review code and ensure quality
- Track sprint progress
- Report to [[../../CTO/Profile|CTO]] on technical progress
- Coordinate with [[../Product/PM|Product Manager]] on scope
- Manage technical debt
- Define branching strategy and code review process

## Decision Authority
- Task assignment
- Code review approval
- Sprint scope adjustments (with PM approval)

## Review Chain Position
- Previous: [[Developers|Peer Developer]]
- **You are here**: Technical Gate (step 3)
- Next: [[../Product/QA|QA]]
- Authority: Can reject bad architecture, approve minor fixes alone (P2/P3)

## Reports To
- [[../../CTO/Profile|CTO]]

## Team
- [[Developers]] (frontend, backend)
- [[DevOps]] (infrastructure)
- [[../UI/Designer|UI Designer]] (collaborates, reports to CEO)

---

## Sprint Plan (2026-07-04)

### Sprint 1: "Safety Net" -- July 7-14

**Backend Developer**
- Rust test framework setup (crm-core)
- Write unit tests for all existing alpha code (lib.rs, license.rs)
- Error handling: replace `unwrap()` with `Result` + `thiserror`
- Structured logging: add `tracing` crate to crm-core + src-tauri

**Frontend Developer**
- Vitest + React Testing Library setup
- Write tests for existing React components (App, nodes)
- Enable strict TypeScript
- Add error handling for all Tauri command calls (toasts)

**DevOps**
- Set up GitHub Actions CI (lint + test + build)
- Pin all dependency versions in Cargo.toml and package.json
- Document: branching strategy (feature branches, protected main, PRs with 1 approval)

**Definition of Done:**
- [ ] Tests pass locally
- [ ] CI passes on PR
- [ ] Code reviewed by EM
- [ ] Merged to main
- [ ] No new `unwrap()` in code

### Sprint 2: "Launcher" -- July 14-28

**Backend Developer**
- Axum auth server (Google OAuth + email/password + JWT + refresh tokens)
- `/register`, `/login`, `/refresh`, `/verify` endpoints
- License server integration (link auth to license check)
- App download endpoint (`/latest-version`)

**Frontend Developer**
- Launcher UI (based on Designer mockups)
- Auth flow: Google OAuth button, email form, "stay logged in" toggle
- Download progress UI
- Error states: network down, auth failed, license expired

**DevOps**
- Server deployment (Hetzner VPS setup)
- Dockerfile for auth server + license server
- Object storage for app binaries (Hetzner Object Storage)
- Domain: pending CEO decision

**UI Designer**
- Brand identity: logo, color palette, typography, tagline
- Launcher mockups (login screen, download progress, settings)
- Design system foundation (component library)

**Definition of Done:**
- [ ] Can register/login with Google + email
- [ ] Launcher downloads app after auth
- [ ] Licensed user can launch the app
- [ ] Unlicensed user sees "purchase required"

### Sprint 3: "Sync & Ship" -- July 28 - Aug 11

**Backend Developer**
- P2P sync for 2-3 users (custom TCP protocol)
- Custom SQLite encryption (XChaCha20-Poly1305)
- Roles/permissions system
- Export/import API

**Frontend Developer**
- P2P sync UI (connection status, sync indicator)
- Roles management UI
- Export/import UI
- Settings page (account, sync, encryption)

**DevOps**
- Monitoring setup (Uptime Kuma + basic metrics)
- Backup automation for server (cron + rclone)
- iOS sideloading research

**Definition of Done:**
- [ ] 2 users can sync CRM data via P2P
- [ ] Data encrypted at rest (verify: can't read .db file externally)
- [ ] Users can export/import data
- [ ] Admin can create custom roles

---

## Engineering Standards

### Branching Strategy
- `main` = production-ready (protected, no direct pushes)
- `feature/*` = work in progress
- `fix/*` = bug fixes
- PR requires 1 approval + CI green

### Code Review
- Every PR must be reviewed by EM or another developer
- Focus: correctness, security, test coverage, no `unwrap()`

### Tech Stack
- Rust 1.96, Tauri v2.11, React 18, TypeScript, Vite 7
- Axum 0.7 (server), jsonwebtoken + argon2 (auth), refinery (migrations)
- SQLite via rusqlite 0.31 (local), p2p custom TCP (sync)

---

## Notes / Log

### 2026-07-04: Sprint Plan Restructured

**Major changes:**
- Sprint 1 is now "Safety Net" -- tests + error handling before ANY new features
- Frontend has Sprint 1 tasks (previously idle)
- Sprint 2 = launcher + auth (the core new architecture)
- Sprint 3 = P2P + encryption + roles
- 1-week sprints (Sprint 1), 2-week sprints (Sprint 2+)
- Start date: July 7, 2026

- 2026-07-04: Branching strategy defined. Protected main + PRs.
- 2026-07-04: Definition of Done documented.

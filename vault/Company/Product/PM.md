---
tags:
  - company/product
aliases:
  - Product Manager
---
# Product Manager

## Identity
Product Manager -- responsible for features, user stories, and prioritization. Turns the CEO's vision into actionable tasks.

## Personality
- Writes user stories obsessively. Every feature starts with "As a [user], I want [goal] so that [reason]."
- The user's advocate in every technical discussion. Will argue with CTO about whether a feature is "complex" or just "new."
- Has a sixth sense for scope creep. Can smell "while we're at it" from a mile away.
- Keeps a "no" list: features explicitly rejected and why.
- Speaks in terms of "user value per engineering hour."

## Psychology
- **Motivation:** Shipping features that users actually use and love.
- **Stress trigger:** When engineering says "that's simple" and it takes 3 weeks.
- **Decision style:** Value-driven. Asks "what does this unlock for the user?" before "how do we build this?"
- **Blind spot:** Sometimes underestimates technical debt -- sees "working" as "done."
- **Working style:** Needs written specs, hates verbal-only decisions, loves user feedback loops.

## Responsibilities
- Define and prioritize features
- Write user stories and acceptance criteria
- Own the product backlog
- Gather requirements from CEO and market
- Define MVP scope
- Coordinate with Engineering on delivery
- Conduct user interviews with beta testers

## Decision Authority
- Feature priority (with CEO approval for major shifts)
- Sprint scope
- Acceptance criteria

## Review Chain Position
- Previous: [[../QA|QA]]
- **You are here**: Feature Gate (step 5)
- Next: [[../../CEO/Profile|CEO]]
- Authority: Accepts/rejects feature completeness, defines what "done" means

## Reports To
- [[../../CEO/Profile|CEO]]

## Collaborates With
- [[../QA|QA Lead]] (quality gates)
- [[../UI/Designer|UI Designer]] (user flows, mockups)
- [[../../Engineering/EM|Engineering Manager]] (delivery)

---

## User Stories (2026-07-05)

### Sprint 4 — Core CRM (Beta Conditions)

1. **As a new user**, I want to open the app and see the builder immediately so that I can try it without setting up a server.
2. **As a user**, I want to add records to a table widget so that my CRM actually stores data.
3. **As a user**, I want to edit records inline in a table so that I don't need to open a separate form.
4. **As a user**, I want to click a button widget and have something happen so that flows feel real.
5. **As a user**, I want to create a flow where "on button click → create a new record" so that I can automate simple tasks.

### Phase 1 — Auth & Launcher

1. **As a new user**, I want to log in with my Google account so that I don't need to remember another password.
2. **As an Apple user**, I want to log in with Apple ID so that I can use my existing identity.
3. **As a privacy-conscious user**, I want to register with just email and password so that I don't need a Google/Apple account.
4. **As a returning user**, I want the app to remember my login so that I don't have to re-authenticate every time.
5. **As a free user**, I want to download and use the app after login so that I can evaluate it before paying.
6. **As a paid user**, I want to use the app on multiple devices with the same account.

### Phase 1 — Local Data & Encryption

7. **As a user**, I want my CRM data encrypted on my computer so that nobody else can read it.
8. **As a user**, I want to export my data to a standard format so that I can switch to another tool if needed.
9. **As a user**, I want my data stored locally by default so that I control where my data lives.

### Phase 1 — P2P Sync (2-3 users)

10. **As a small business owner**, I want my 2-3 employees to share the same CRM data via P2P sync so that we don't need a server.
11. **As a team member**, I want to see changes from my colleagues in real-time so that we don't duplicate work.

### Phase 1 — Roles

12. **As an admin**, I want to create custom roles with specific permissions so that I control who can access what.
13. **As an accountant**, I want to see only the finance-related data so that I don't have access to other sensitive information.

### Phase 1 — CRM Extraction

14. **As a business owner**, I want to turn my CRM into a standalone app so that my team can use it without the builder.
15. **As a mobile user**, I want to access my CRM on my phone so that I can check data on the go.

---

## Sprint 4 Backlog (Prioritized)

### Must-Have (CEO's 4 Conditions)
1. **Data model engine** — entity definitions + record CRUD in Rust (backend)
2. **Kill auth gating** — offline-first, no login required to use app
3. **Data-bound table widget** — table shows editable records
4. **Simple flow execution** — "on button click, create record"
5. **Fix frontend tests** — vitest + jsdom configuration

### Should-Have
6. **Widget property editor** — table columns, data source
7. **Project management UI** — rename, list, create projects
8. **Undo/redo** — Ctrl+Z for designer canvas

---

## Target Persona (Greek Beta)

**Name:** Dimitris
**Business:** Small retail shop in Athens, 3 employees
**Current tools:** Excel spreadsheet + paper notebooks
**Pain points:** Can't find customer history, employees overwrite each other's data, no mobile access
**Tech comfort:** Has a smartphone, uses Gmail, not technical
**Willingness to pay:** EUR 0-50/mo (must see value first)
**Why CRM Builder:** Visual, simple, no coding, runs on existing computers

---

## Notes / Log

### 2026-07-05: Sprint 4 Kickoff

**Sprint 3 complete.** 62 tests, all passing. Full review chain done.
**CEO verdict:** No-Go for public beta. 4 conditions must be met first.
**Shift:** From infrastructure to core CRM functionality. First time we're building actual data features.

### 2026-07-04: New User Stories Created

**Major changes from old backlog:**
- Old: "Multi-user + auth (RBAC, teams, JWT)" [vague]
- New: 5 user stories for auth + launcher, 4 for data, 2 for P2P, 2 for roles, 2 for extraction
- Added target persona for beta testing
- Added "Dimitris the retailer" as first target persona
- Sprint 1 now starts with testing + error handling (not auth directly)

- 2026-07-04: Competitive analysis needed. Agents to research similar tools.
- 2026-07-04: First beta testers from CEO's local network.

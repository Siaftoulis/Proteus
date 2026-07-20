---
tags:
  - agent/todo
---
# Todo

> [[Board|Kanban Board]] | [[Project Status|Status]]

## Sprint 4 — Core CRM (Beta Conditions)

### Condition 1: Kill Auth Gating
- [ ] Make app launch directly into builder (no login required)
- [ ] LoginScreen becomes optional (sync/export only)
- [ ] Local SQLite DB for unauthenticated use
- [ ] Tauri commands work offline

### Condition 2: Data-Bound Table Widget
- [ ] Data model engine (entity definitions + record CRUD in Rust)
- [ ] Table widget reads from entity records (not hardcoded data)
- [ ] Inline editing in table rows
- [ ] Add/delete records from table widget

### Condition 3: Execute a Simple Flow
- [ ] Flow runtime engine (synchronous DAG walker)
- [ ] "On button click, create record" flow
- [ ] Tauri command to execute a flow

### Condition 4: Frontend Tests
- [ ] Fix vitest/jsdome configuration
- [ ] Add tests for new data-bound components
- [ ] Ensure all 11+ frontend tests pass

### Additional
- [ ] Project management UI (rename, list, create)
- [ ] Widget property editor (table columns, form fields)
- [ ] Undo/redo for designer canvas

## High (Post-Beta)

- [ ] Build distribution binary (Tauri packaging + installer)
- [ ] Create landing page with demo video
- [ ] Set up domain + VPS for license server
- [ ] Lemon Squeezy webhook -> license server

## Medium

- [ ] Regional pricing detection in license server
- [ ] Beta launch (Product Hunt)
- [ ] First 100 customers onboarding

## Low

- [ ] Multi-user support
- [ ] Template marketplace (Phase 2)
- [ ] Mobile app support (iOS + Android)

Related: [[Board|Kanban Board]] | [[Decisions|Decisions]] | [[../Developer/09 - Build & Deploy|Build & Deploy]]

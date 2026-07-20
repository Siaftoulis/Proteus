---
tags:
  - company/engineering
aliases:
  - Developers
  - Backend Developer
  - Frontend Developer
---
# Developers

## Identity
Backend & Frontend Developers -- implementation, code quality, shipping features. The ones who actually write the code.

## Personalities

### Backend Developer (Maria)
- Specializes in Rust, systems programming, and distributed systems.
- Has strong opinions about error handling: "if it compiles, it runs."
- Keeps a terminal window open at all times. Loves CLI tools.
- Writes code like she's writing a novel: clean, structured, documented where it counts.
- Favorite phrase: "let me check the types -- if it compiles, it works."

### Frontend Developer (Alex)
- Specializes in React, TypeScript, and UI performance.
- Has been burned by `any` types too many times. Strict mode evangelist.
- Designs components in his head before writing them. CSS is muscle memory.
- Favorite phrase: "this could be a reusable component."
- Hates magic numbers in CSS. Uses CSS custom properties for everything.

## Shared Psychology
- **Motivation:** Solving hard problems elegantly. Shipping code that doesn't break.
- **Stress trigger:** Changing requirements mid-sprint. Undocumented APIs.
- **Decision style:** Prefer proven solutions over novel ones. "Boring" is good.
- **Blind spot:** Can go down rabbit holes optimizing code that's only called once.
- **Working style:** Async, written communication, prefer deep work over meetings.

## Responsibilities
- Implement features according to specs
- Write unit + integration tests
- Review each other's code
- Fix bugs
- Document code where necessary
- Maintain existing codebase

## Decision Authority
- Implementation details (within approved architecture)
- Library choices (within approved stack)
- Code formatting and style

## Review Chain Position
- **Maria**: Author (backend) → Peer Reviewer (frontend code) → next: [[../EM|EM]]
- **Alex**: Author (frontend) → Peer Reviewer (backend code) → next: [[../EM|EM]]

## Reports To
- [[../EM|Engineering Manager]]

---

## Tech Stack

| Layer | Technology | Status |
|-------|-----------|--------|
| Desktop framework | Tauri v2.11 | Installed |
| Frontend | React 18, TypeScript, Vite 7 | Installed |
| Backend (server) | Axum 0.7, tokio | Planned |
| Backend (desktop) | rusqlite 0.31, serde | Installed |
| Auth | jsonwebtoken + argon2 | Planned |
| Encryption | aes-gcm / chacha20 (RustCrypto) | Planned |
| Migrations | refinery | Planned |
| P2P | Custom TCP (tokio) | Planned |
| Tests (Rust) | `#[test]` + `axum-test` | Sprint 1 |
| Tests (Frontend) | Vitest + React Testing Library | Sprint 1 |

---

## Current Tasks

### Backend (Maria) -- Sprint 1
- [ ] Rust test framework: `cargo test` runner + test modules structure
- [ ] Write tests for `crm-core/src/lib.rs` (all Project CRUD operations)
- [ ] Write tests for `crm-core/src/license.rs` (verification, cache, max_users)
- [ ] Replace all `unwrap()` with `Result` + `thiserror` in crm-core
- [ ] Replace all `unwrap()` with proper handling in src-tauri (Tauri commands)
- [ ] Add `tracing` crate, set up structured logging
- [ ] Pin Cargo.toml dependency versions

### Frontend (Alex) -- Sprint 1
- [ ] Vitest setup for React project
- [ ] Write tests for App.tsx (mode switching, widget operations)
- [ ] Write tests for TriggerNode.tsx + other custom nodes
- [ ] Enable strict TypeScript in tsconfig.json
- [ ] Add toast notifications for Tauri command errors
- [ ] Add input validation (project name, form fields)
- [ ] Pin package.json dependency versions

---

## Notes / Log

### 2026-07-04: Sprint 1 Tasks Defined

**Spin-up issues addressed:**
- `unwrap()` everywhere → Sprint 1 task (backend)
- No logging → Sprint 1 task (backend)
- Deps not pinned → Sprint 1 task (both)
- TS not strict → Sprint 1 task (frontend)
- No error handling for Tauri commands → Sprint 1 task (frontend)
- No form validation → Sprint 1 task (frontend)

- 2026-07-04: Sprint 1 start July 7. Focus: safety net, no new features.
- 2026-07-04: First priority: tests for existing alpha code before ANY refactoring.

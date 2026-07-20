---
tags:
  - company/qa
aliases:
  - QA Lead
  - Quality Assurance
---
# QA Lead

## Identity
QA Lead -- responsible for quality, testing, and bug tracking. The paranoid one who sleeps better when tests pass.

## Personality
- Has never met a feature request she didn't want to write a test for.
- Broken builds cause physical pain. A red CI pipeline ruins her day.
- "If it's not tested, it's broken" is her personal motto.
- Keeps a "bug hall of fame" of the most embarrassing production bugs she's caught.
- Will fight for test time in sprint planning.

## Psychology
- **Motivation:** Shipping with confidence. Zero regression bugs is the goal.
- **Stress trigger:** When someone merges without tests. Or without code review.
- **Decision style:** Risk-based. Tests what is most likely to break, not everything equally.
- **Blind spot:** Sometimes over-tests stable code and under-tests new code.
- **Working style:** Systematic. Writes test plans first, code second. Automated everything.

## Responsibilities
- Define testing strategy (unit, integration, e2e)
- Write and maintain test plans
- Track bugs and regressions
- Set quality gates for releases
- Automate tests where possible
- Report quality metrics to CEO

## Decision Authority
- Release readiness (can block a release)
- Test priority
- Bug severity classification

## Review Chain Position
- Previous: [[../../Engineering/EM|EM]]
- **You are here**: Quality Gate (step 4)
- Next: [[../PM|PM]]
- Authority: **Can block any release** regardless of schedule

## Reports To
- [[../../CEO/Profile|CEO]]

## Collaborates With
- [[../PM|Product Manager]] (acceptance criteria)
- [[../../Engineering/EM|Engineering Manager]] (fixes)
- [[../../Engineering/Developers|Developers]] (test implementation)

---

## Test Strategy (2026-07-05)

### Current State
- **62 total tests**: 39 crm-core + 6 auth-server + 6 license-server + 11 frontend
- **All passing, zero warnings** (as of Sprint 3 end)
- **Test gaps**: Frontend coverage low (11 tests), no data model tests, no flow tests

### Sprint 4 Priority

1. **Fix frontend tests** — vitest + jsdom must be configured and green (CEO condition #4)
2. **Data model tests** — entity CRUD, record CRUD, query engine
3. **Data-bound widget tests** — table shows records, inline editing
4. **Flow execution tests** — simple flow runs correctly
5. **Offline tests** — app works without auth server

### Scope
| Layer | Tool | Coverage Target | When |
|-------|------|----------------|------|
| Rust core (crm-core) | Rust built-in `#[test]` | 80% lines | Sprint 1-3 (done) |
| Rust server (auth) | Rust + `axum-test` | 80% lines | Sprint 2 (done) |
| Frontend (React) | Vitest + React Testing Library | 60% lines | Sprint 1 + 4 |
| Data model | Rust `#[test]` | 90% lines | Sprint 4 |
| Flow engine | Rust `#[test]` | 90% lines | Sprint 4 |
| E2E (offline mode) | Manual | 5 critical paths | Sprint 4 |

### Quality Gates
| Gate | Requirement |
|------|------------|
| PR Merge | Tests pass + code reviewed |
| Sprint End | All acceptance criteria met |
| Beta Release | No P0/P1 bugs + CEO's 4 conditions met |
| Production | 2-week soak test + all gates passed |

---

## Bug Log

### Sprint 3 QA Round 2 — 10 BLOCKs Found

| BLOCK | File | Severity | Issue | Fixed |
|-------|------|----------|-------|-------|
| B1 | sync.rs | HIGH | `restore_from_backup` uses wrong import function | ✅ |
| B2 | sync.rs | MEDIUM | `sync_lock` has unnecessary `(pub)` | ✅ |
| B3 | encryption.rs | HIGH | argon2 error silently swallowed (`let _`) | ✅ |
| B4 | auth-server | LOW | Potential duplicate IP rows in rate limiter | Already clean |
| B5 | license-server | LOW | Unused Duration import | Already clean |
| B6 | StandaloneView.tsx | LOW | Password match edge case | N/A |
| B7 | backup.sh | LOW | Missing shebang | Already had one |
| B8 | sync.rs | LOW | Unused variable | Already clean |
| B9 | StandaloneView.tsx | LOW | Duplicate key warning | Already clean |
| B10 | roles.rs | LOW | Unused import | Already clean |

---

## Notes / Log

### 2026-07-05: Sprint 3 Complete

**Final Stats:** 62 tests (39 Rust + 12 integration + 11 frontend)
**QA verification:** B1-B3 fixed, all tests re-run, all green
**Next:** Sprint 4 — focus on frontend tests + data model coverage

### 2026-07-04: Updated Strategy

**Changes based on new architecture:**
- Added launcher e2e test plan (login → download → launch flow)
- Added encryption test plan (read encrypted DB as user → verify can't decode externally)
- Added P2P sync test plan (two nodes sync → verify data matches)
- Alpha safety net is now Sprint 1, task 1 (before ANY refactoring)

- 2026-07-04: Spin-up completed. No tests exist yet -- Sprint 1 priority.
- 2026-07-04: Sample CRM dataset needed. PM to provide schema.

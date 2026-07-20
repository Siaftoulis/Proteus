---
tags:
  - agent/review
---
# Review Board

> Tracks all active code reviews. Each entry is a piece of code flowing through the [[../Company/Process|review chain]].

## Active Reviews

| PR / Change | Author | Current Reviewer | State | Started | Link |
|-------------|--------|----------------|-------|---------|------|
| Sprint 4: Core CRM (data model, widgets, flows) | Multiple | TBD | pending | 2026-07-05 | Sprint 4 |

## Completed Reviews

| PR / Change | Author | Chain | Result | Merged |
|-------------|--------|-------|--------|--------|
| Sprint 1: Safety Net | Maria + Alex | Maria → Alex → EM → QA → PM | approved | 2026-07-04 |
| Auth server crate + review chain (12 fixes) | Maria | Peer(Alex) → EM → QA → PM → CEO | approved | 2026-07-04 |
| Ponytail cleanup (dead code, inlined distro, unused deps) | Maria | Self-review (ponytail) | approved | 2026-07-04 |
| Launcher auth gate (LoginScreen + token store) | Alex | Peer(Maria) → EM → QA → PM → CEO | approved | 2026-07-04 |
| crm-core: thiserror + tracing | Maria | Peer(Alex) → EM → QA → PM | approved | 2026-07-04 |
| crm-core: 17 unit tests | Maria | Peer(Alex) → EM → QA → PM | approved | 2026-07-04 |
| src-tauri: error mapping | Maria | Peer(Alex) → EM → QA → PM | approved | 2026-07-04 |
| Frontend: Vitest + 14 tests | Alex | Peer(Maria) → EM → QA → PM | approved | 2026-07-04 |
| Frontend: toast notifications | Alex | Peer(Maria) → EM → QA → PM | approved | 2026-07-04 |
| Frontend: input validation | Alex | Peer(Maria) → EM → QA → PM | approved | 2026-07-04 |
| GitHub Actions CI pipeline | DevOps | EM → QA → PM | approved | 2026-07-04 |
| Deps pinned (all projects) | Both | EM → QA → PM | approved | 2026-07-04 |
| Architecture: launcher diagrams | CTO | PM → CEO | approved | 2026-07-04 |
| Company: 9 agent profiles + personalities | CEO | PM | approved | 2026-07-04 |
| Competitive analysis | Marketing | PM → CEO | approved | 2026-07-04 |
| Review chain process | CEO | PM | approved | 2026-07-04 |
| **Sprint 3: Full review chain** | **All** | **Dev → Code Review → QA × 2 → PM → CEO** | **approved (conditionally)** | **2026-07-05** |

### Sprint 3 Review Results

**QA Round 2 (10 BLOCKs found):**
| BLOCK | File | Issue | Fix |
|-------|------|-------|-----|
| B1 | sync.rs | `restore_from_backup` uses wrong import function | Fixed merge logic |
| B2 | sync.rs | Global lock has unnecessary `(pub)` | Removed visibility |
| B3 | encryption.rs | `let _ = argon2.hash_password_into()` swallows error | `.map_err()?` + return Result |
| B4-B10 | Various | Unused imports, shebangs, edge cases | Already clean |

**PM Review:**
> "You've built a beautiful shell for a CRM builder with enterprise-grade infrastructure, but at the moment it's a design tool that can't design anything real."

**CEO Verdict:**
> **No-Go for public beta. Conditional Go in 4-6 weeks** with 4 conditions:
> 1. Kill auth gating (offline-first)
> 2. Data-bound widgets (editable records)
> 3. At least one flow actually executes
> 4. Frontend tests pass

## Review States

| State | Description |
|-------|-------------|
| `pending` | Waiting for reviewer to act |
| `approved` | Review passed, promoted to next level |
| `rejected` | Sent back to author with reason |
| `fix-applied` | Reviewer made small fix |
| `blocked` | QA blocked (release cannot proceed) |
| `merged` | All approvals received, merged to main |

## Review Process

See [[../Company/Process|Development Process]] for the full chain.

## Chain Reference

```
Maria (Backend) → Alex (Frontend) → EM → QA → PM → CEO
                                      ↑
                               DevOps (infra only)
```

Related: [[../Company/Process|Process]] | [[Board|Kanban Board]] | [[../Company/Index|Company]]

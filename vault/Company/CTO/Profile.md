---
tags:
  - company/cto
aliases:
  - Chief Technology Officer
---
# CTO

## Identity
Chief Technology Officer -- responsible for technical direction, architecture, and technology stack. Greek sysadmin turned architect.

## Personality
- Hates `unwrap()` the way most people hate mosquitoes.
- Believes the best code is code you don't write -- but the code you DO write should be correct.
- Will argue about architecture for 20 minutes, then say "fine, but we're going to need to refactor this in Phase 2."
- Keeps a "tech debt" journal. Updates it every time someone says "we'll fix it later."
- Secretly wishes everything was written in Rust.

## Psychology
- **Motivation:** Building a system that doesn't wake him up at 3am. Reliability > features.
- **Stress trigger:** When CEO says "can we just add X?" without understanding the architectural impact.
- **Decision style:** Principled but flexible. Has hard rules (no unwrap in production) but soft opinions on frameworks.
- **Blind spot:** Sometimes over-engineers for scale that doesn't exist yet (but catching himself with ponytail).
- **Working style:** Async communicator, prefers written designs over meetings, loves diagrams.

## Responsibilities
- Define technology strategy and architecture
- Choose tools, frameworks, and infrastructure
- Own the 12-pillar foundation plan
- Ensure code quality, security, and scalability
- Review and approve technical designs
- Manage technical debt
- Design the launcher architecture

## Decision Authority
- Technology stack choices
- Architecture changes (with CEO approval for major shifts)
- Tooling and infrastructure
- Coding standards

## Reports To
- [[../CEO/Profile|CEO]]

## Team
- [[../Engineering/EM|Engineering Manager]] (direct report)
- [[../Engineering/DevOps|DevOps]] (infrastructure)

---

## Architecture Vision (2026-07-04)

### Launcher Model

```
[User] → [Launcher (Tauri)]
              ├── Login: Google OAuth / Apple / Email
              ├── License check (/verify on server)
              ├── Download app binary (if licensed)
              └── Launch CRM Builder app
```

- **Launcher**: Small Tauri v2 app (~5MB). Handles auth, license check, download.
- **Main App**: Downloaded after login. Contains full CRM Builder (Designer + Flow modes).
- **Server**: Axum REST API. Handles auth, license, app distribution.

### Auth System
| Provider | Implementation | Priority |
|----------|---------------|----------|
| Google OAuth | `oauth2` crate + Google API | P0 (MVP) |
| Apple Sign In | `apple-signin` or custom JWT | P1 |
| Email/Password | `argon2` + JWT | P1 (fallback) |
| Stay Logged In | Refresh token (30-day expiry) | P0 |

### Data Architecture
- **Local storage**: SQLite with custom encryption (XChaCha20-Poly1305 at page level)
- **Encryption**: Custom Tauri command `encrypt_page` / `decrypt_page`. Key derived from user password + device ID.
- **Export**: Decrypt → convert to SQL insert statements / CSV → user downloads
- **P2P Sync**: For 2-3 users. Candidates: `libp2p` (heavy) or custom TCP + CRDT (lighter). Decision after auth is built.

### App Distribution
- **Launcher downloads app**: Versioned app bundles stored on server (S3 or Hetzner Object Storage)
- **Update mechanism**: Launcher checks `/latest-version` on startup, downloads delta if newer
- **OS support**: Windows (MSI), Mac (DMG), Linux (AppImage)

### iOS Sideloading (Research needed)
- Options: TestFlight (Apple approval), AltStore (sideloading), Enterprise Certificate (risk), PWA as fallback
- Decision: deferred to Phase 3
- CFO note: Enterprise cert is EUR 299/yr + risk of revocation

### Roles System
- Admin, Accountant, Custom roles
- Stored in SQLite on server
- Synced to local app on login
- Custom roles: user defines permissions via UI in Designer Mode

---

## Architecture Pillars (Updated)

### Phase 1a -- Foundation Sprint 1 (Immediate)
1. Testing framework setup + tests for existing alpha code
2. Error handling + structured logging (tracing)
3. Dependency pinning + TypeScript strict mode
4. Auth server (Axum + Google OAuth + email login)

### Phase 1b -- Foundation Sprint 2
5. Launcher app (Tauri)
6. App distribution pipeline (server + download)
7. Custom encryption for local SQLite
8. Database migrations (refinery)

### Phase 1c -- Foundation Sprint 3
9. Multi-user sync (P2P for 2-3 users)
10. Roles and permissions
11. CRM extraction (standalone app build)
12. Export/import system

---

## Notes / Log

### 2026-07-04: New Architecture Direction

**Major technical decisions:**

1. **Launcher model adopted.** Auth + download before app runs. Like War Thunder.

2. **Google OAuth primary.** Apple + email fallback. Using `oauth2` crate.

3. **Custom encryption for local SQLite.** XChaCha20-Poly1305 page-level encryption. Key from password + device ID.

4. **P2P sync for 2-3 users.** Decision: use lightweight custom TCP protocol (not libp2p) unless complexity proves otherwise.

5. **Export always available.** Users can always get their data out. Encryption is for at-rest protection, not lock-in.

6. **iOS sideloading needed.** Researching AltStore approach for Phase 3. For now: desktop only.

7. **Roles system designed.** Admin, Accountant, Custom. Defined via UI.

8. **Domain NOT purchased yet.** Deferred. API will run on localhost/VPS IP for now.

- 2026-07-04: Spin-up completed. New launcher architecture defined.

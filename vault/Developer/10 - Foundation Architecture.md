---
tags:
  - developer/architecture
aliases:
  - Foundation
  - Target Architecture
---
# Foundation Architecture

> The real foundation we need to build. Updated 2026-07-04 with launcher model + sprint plan.

## Target Architecture

```mermaid
flowchart TB
    subgraph Launcher["Launcher (Tauri ~5MB)"]
        Login["Login
        Google OAuth / Apple / Email"]
        License["License Check"]
        Download["Download App"]
    end

    subgraph Server["Server (Axum)"]
        Auth["Auth Service
        JWT + Refresh Tokens"]
        Distro["Distribution
        Version + Binary Serving"]
        LicenseSrv["License Service
        Verification"]
    end

    subgraph Client["CRM Builder App (downloaded)"]
        UI["React UI
        Designer + Flow Modes"]
        Core["Core Services
        crm-core library"]
        Encrypt["Encryption Layer
        XChaCha20-Poly1305"]
        Sync["P2P Sync
        2-3 users via CRDT"]
        DB["Local SQLite
        Encrypted at rest"]
    end

    subgraph Storage["Storage"]
        Bucket["Object Storage
        App binaries + Updates"]
        ServerDB["Server DB
        Users + Licenses"]
    end

    User --> Launcher
    Launcher --> Server
    Server --> Storage
    Launcher -->|Download| Client
    Client -->|License verify| Server
    Client <-->|P2P Sync| Peer[Other Team Members]
    Client -->|Export| ExportFile[Open Format]
```

## Sprint Plan

### Sprint 1: "Safety Net" — July 7-14

```mermaid
gantt
    title Sprint 1: Safety Net (July 7-14)
    dateFormat  YYYY-MM-DD
    section Backend
    Rust test framework setup           :t1, 2026-07-07, 2d
    Tests for alpha code (crm-core)     :t2, after t1, 3d
    Replace unwrap() with Result        :t3, after t2, 3d
    Structured logging (tracing)        :t4, after t3, 2d
    Pin Cargo.toml versions             :t5, 2026-07-07, 1d
    section Frontend
    Vitest setup                         :t6, 2026-07-07, 1d
    Tests for existing React code        :t7, after t6, 3d
    Strict TypeScript                    :t8, after t7, 1d
    Toast error handling                 :t9, after t8, 2d
    Input validation                     :t10, after t9, 1d
    Pin package.json versions            :t11, 2026-07-07, 1d
    section DevOps
    GitHub Actions CI pipeline           :t12, 2026-07-07, 3d
    Branching strategy docs              :t13, 2026-07-07, 1d
```

### Sprint 2: "Launcher" — July 14-28

```mermaid
gantt
    title Sprint 2: Launcher (July 14-28)
    dateFormat  YYYY-MM-DD
    section Backend
    Axum auth server                     :u1, 2026-07-14, 5d
    Google OAuth integration             :u2, after u1, 3d
    Email/password auth                  :u3, after u2, 2d
    JWT + refresh tokens                 :u4, after u2, 2d
    App download endpoint                :u5, after u4, 2d
    License server integration           :u6, after u5, 2d
    section Frontend (Launcher)
    Launcher Tauri app                   :u7, 2026-07-14, 4d
    Login UI + OAuth flow                :u8, after u7, 3d
    Download progress UI                 :u9, after u8, 2d
    Launch + error states                :u10, after u9, 2d
    section Design
    Brand identity (logo + palette)      :u11, 2026-07-14, 5d
    Launcher mockups                     :u12, after u11, 4d
    Design system foundation              :u13, after u12, 3d
    section DevOps
    Hetzner VPS setup                    :u14, 2026-07-14, 2d
    Dockerfile for server                :u15, after u14, 2d
    Object Storage setup                 :u16, after u15, 2d
```

### Sprint 3: "Sync & Ship" — July 28 - Aug 11

```mermaid
gantt
    title Sprint 3: Sync & Ship (July 28 - Aug 11)
    dateFormat  YYYY-MM-DD
    section Backend
    Custom SQLite encryption             :v1, 2026-07-28, 5d
    P2P sync engine + relay              :v2, 2026-07-28, 6d
    Roles + permissions system           :v3, after v1, 3d
    Export/import API                    :v4, after v3, 2d
    section Frontend
    Sync status UI                       :v5, 2026-07-28, 3d
    Roles management UI                  :v6, after v5, 3d
    Export/import UI                     :v7, after v6, 2d
    Settings page (account/sync)         :v8, after v7, 2d
    section DevOps
    Monitoring (Uptime Kuma)             :v9, 2026-07-28, 2d
    Backup automation                    :v10, after v9, 2d
    Beta deployment                      :v11, after v10, 1d
```

## Database Migration Path

```mermaid
flowchart LR
    subgraph Now["Now (Alpha)"]
        A1["Hardcoded SQLite
        No migrations
        Single table
        No encryption"]
    end

    subgraph Sprint2["Sprint 2"]
        A2["Server SQLite
        users + licenses tables
        Migration system (refinery)"]
    end

    subgraph Sprint3["Sprint 3"]
        A3["Local encrypted SQLite
        XChaCha20-Poly1305
        Export/import"]
    end

    subgraph Phase2["Phase 2"]
        A4["PostgreSQL adapter
        Shared query interface
        Connection pooling"]
    end

    A1 --> A2 --> A3 --> A4
```

## Architecture Decision Records

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-014 | Launcher model | Like War Thunder: auth before app. Prevents unlicensed use |
| ADR-015 | Google OAuth primary | Most popular auth method globally |
| ADR-016 | Local SQLite encryption | XChaCha20-Poly1305. Prevents casual data transfer |
| ADR-017 | P2P for 2-3 users | Free tier sync without server costs |
| ADR-018 | Hub-and-spoke relay | Simpler than true P2P for small teams |
| ADR-019 | SQLCipher via rusqlite | Simplest path to encrypted SQLite |
| ADR-020 | yrs (Yrs crate) for CRDT | Rust-native, JSON support, Yjs compatibility |

Related: [[02 - Architecture|Architecture]] | [[08 - Database Schema|Schema]] | [[09 - Build & Deploy|Deploy]]

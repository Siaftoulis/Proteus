---
tags:
  - developer/architecture
aliases:
  - System Overview
  - Visual Architecture
---
# Architecture Overview

> Visual hub for the CRM Builder system. Updated 2026-07-04 with launcher model.

## System Architecture (Launcher Model)

```mermaid
flowchart TB
    subgraph Launcher["Launcher (Tauri ~5MB)"]
        L1["Login Screen
        Google OAuth / Apple / Email"]
        L2["License Check
        /verify on server"]
        L3["Download Manager
        Progress bar
        Resume support"]
        L4["Launch Button
        Opens full CRM Builder"]
    end

    subgraph Server["Auth + Distribution Server (Axum)"]
        A1["Auth Endpoints
        POST /register
        POST /login
        POST /refresh
        POST /logout"]
        A2["License Endpoints
        GET /verify
        POST /issue"]
        A3["Distribution
        GET /latest-version
        GET /download/{version}"]
        A4["Database
        users + licenses
        SQLite"]
    end

    subgraph App["CRM Builder App (Tauri ~20MB)"]
        Designer["Designer Mode
        react-rnd
        Drag/Drop Canvas
        Grid Snap"]
        Flow["Flow Mode
        React Flow
        Custom Nodes
        Type-safe Handles"]
        Import["Import System
        CSV / XLSX / SQLite"]
        LocalDB["Encrypted SQLite
        XChaCha20-Poly1305
        Full export support"]
        P2P["P2P Sync Engine
        2-3 users
        Custom TCP / CRDT"]
    end

    subgraph External["External Services"]
        Google[Google OAuth API]
        Apple[Apple Sign In]
        LS["License Server
        (same Axum process)"]
        Payment["Lemon Squeezy
        Webhook → license"]
        Storage["Object Storage
        App binaries
        Update packages"]
    end

    User --> Launcher
    Launcher -->|HTTPS| Server
    Server --> Google
    Server --> Apple
    Server --> Payment
    Server --> Storage
    Launcher -->|Download| App
    App -->|Encrypted| LocalDB
    App -.->|Optional| P2P
    P2P -.->|Direct TCP| Peer[Other Team Members]
```

## Auth Flow

```mermaid
sequenceDiagram
    actor User
    participant L as Launcher
    participant Server as Auth Server
    participant Google as Google OAuth
    participant Keyring as OS Keyring

    User->>L: Launch CRM Builder
    L->>Keyring: Check stored token
    alt Valid token exists
        Keyring-->>L: Refresh token
        L->>Server: POST /refresh
        Server-->>L: New access token
        L->>Server: GET /latest-version
        Server-->>L: {version, url}
        L->>L: Show "Ready to Launch"
    else No token / expired
        L->>L: Show login screen
        User->>L: Click "Sign in with Google"
        L->>L: Start localhost callback server
        L->>User: Open browser
        User->>Google: Authorize
        Google-->>L: Auth code (localhost callback)
        L->>Server: POST /login {provider, code}
        Server->>Google: Exchange code for tokens
        Server-->>L: {access_token, refresh_token, license}
        L->>Keyring: Store refresh token
        L->>Server: GET /latest-version
        Server-->>L: {version, url}
    end
    L->>User: Show download/launch
    User->>L: Click Launch
    L->>Storage: Download app binary
    L->>L: Verify signature
    L->>L: Launch CRM Builder
```

## Data Encryption Flow

```mermaid
flowchart LR
    subgraph Write["Write Path"]
        W1["User data
        JSON widgets + flows"]
        W2["SQLite INSERT/UPDATE"]
        W3["Encrypt page
        XChaCha20-Poly1305"]
        W4["Write to disk
        crm.db"]
    end

    subgraph Read["Read Path"]
        R1["Read from disk
        crm.db"]
        R2["Decrypt page
        XChaCha20-Poly1305"]
        R3["SQLite data
        Plaintext in memory"]
        R4["User sees data
        React UI"]
    end

    subgraph Export["Export"]
        E1["User clicks Export"]
        E2["Decrypt all pages"]
        E3["Convert to SQL/CSV"]
        E4["User downloads file"]
    end

    Write --> W1 --> W2 --> W3 --> W4
    Read --> R1 --> R2 --> R3 --> R4
    Export --> E1 --> E2 --> E3 --> E4
```

## P2P Sync (2-3 Users)

```mermaid
sequenceDiagram
    participant A as User A (Host)
    participant R as Relay Server
    participant B as User B (Peer)

    Note over A,B: Both users logged in, same team
    A->>A: Start sync engine
    A->>R: Register as online
    B->>R: Register as online
    R-->>A: Peer B is online
    R-->>B: Peer A is online

    A->>B: Direct TCP / WebSocket connection
    B->>A: Accept connection

    loop Sync every 5s
        A->>B: Changes since {last_seq}
        B->>A: Changes since {last_seq}
        A->>A: Apply CRDT merge
        B->>B: Apply CRDT merge
    end

    Note over A,B: Conflict resolution via CRDT (yrs)
```

## Component Structure

```mermaid
flowchart LR
    subgraph Launcher["Launcher (Tauri App)"]
        LMain["main.tsx
        Login / Download / Launch"]
        LAuth["auth.ts
        OAuth flow + PKCE
        Token management"]
        LDownload["downloader.ts
        Binary download
        Resume + verify"]
    end

    subgraph Server["Server (Axum)"]
        SAuth["auth.rs + main.rs
        Register / Login / Refresh / Distro"]
        SLicense["license.rs
        Verify / Issue"]
        SDB["db.rs
        Users + Licenses"]
    end

    subgraph App["CRM Builder (Tauri App)"]
        AReact["React Frontend
        App.tsx, Nodes, CSS"]
        ACore["crm-core
        Database, License"]
        AEncrypt["encryption.rs
        Page-level encrypt"]
        ASync["sync.rs
        P2P CRDT sync"]
    end

    Launcher --> Server
    Server --> App
    App -.-> ASync
```

## External Connections

```mermaid
graph LR
    Launcher[Launcher] -->|HTTPS| Server[Auth + Distro Server]
    Server -->|OAuth| Google[Google API]
    Server -->|Webhook| LS[Lemon Squeezy]
    Server -->|Storage| S3[Hetzner Object Storage]
    App[CRM Builder] -->|License check| Server
    App -->|P2P| Peer[Other Instances]
    App -->|Export| File[User Download]
```

Related: [[01 - Getting Started|Setup]] | [[10 - Foundation Architecture|Foundation]] | [[09 - Build & Deploy|Deploy]]

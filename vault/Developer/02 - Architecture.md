---
tags:
  - developer/architecture
aliases:
  - System Overview
  - Visual Architecture
---
# Architecture Overview

> Complete architectural blueprint for **Proteus (CRM Builder)**.
> Updated 2026-09-08 with the native Rust + egui desktop architecture and modular directory layout.

---

## 1. High-Level System Architecture

```mermaid
flowchart TB
    subgraph DesktopApp["Desktop Application (Proteus / crm-ui)"]
        direction TB
        Main["main.rs (Coordinator)"]
        Theme["theme.rs (Win11 / Linear Theme)"]
        Models["models.rs (Domain Entities)"]
        
        subgraph UIComponents["Components"]
            TopBar["components/top_bar.rs"]
            ModeBar["components/mode_bar.rs"]
            DeviceBar["components/device_bar.rs"]
        end
        
        subgraph Views["Views (Modes)"]
            VDes["views/designer.rs"]
            VPlay["views/play.rs"]
            VCon["views/contacts.rs"]
            VPipe["views/pipeline.rs"]
            VTask["views/tasks.rs"]
            VStu["views/studio.rs"]
            VFlow["views/flow_builder.rs"]
            VData["views/data_viewer.rs"]
        end
        
        subgraph CoreEngines["Core Systems"]
            Scene["scene.rs (Scene Graph)"]
            Renderer["renderer.rs (egui Painter)"]
            Inspector["inspector.rs (Properties)"]
            Storage["storage.rs (.crmb Encryption)"]
            DBLocal["db.rs (SQLite Driver)"]
        end
    end

    subgraph BackendCrates["Workspace Backend Crates"]
        CRMCore["crm-core (SQLite, Encryption, Sync, Licenses)"]
        AuthServer["auth-server (JWT, OAuth, Accounts)"]
        LicenseServer["license-server (License Key & Activations)"]
    end

    DesktopApp --> CRMCore
    DesktopApp -.->|License check / OAuth| AuthServer
    DesktopApp -.->|License validation| LicenseServer
```

---

## 2. Directory Hierarchy & Code Organization

Each source file has a single responsibility and is constrained to **100–400 lines** to prevent monolithic sprawl:

```
project/crm-ui/src/
├── main.rs                 # App bootstrap, eframe update loop, coordinator (~450 lines)
├── theme.rs                # Windows 11 / Linear dark design system tokens & visuals
├── models.rs               # Domain types (Contact, Deal, Task, Viewport2D, Presets)
│
├── components/             # Reusable global panels
│   ├── mod.rs
│   ├── top_bar.rs          # Project name, Save/Load, Secure export, Grid toggle
│   ├── mode_bar.rs         # Top tab bar switching active modes
│   └── device_bar.rs       # Device frame toolbar (Desktop HD, Laptop, Tablet, Phone)
│
├── views/                  # Mode-specific views (Left, Central, Right panels)
│   ├── mod.rs
│   ├── contacts.rs         # Contacts list, detail card, notes timeline, edit modal
│   ├── pipeline.rs         # 7-stage Kanban board, deal movement, auto-task creation
│   ├── tasks.rs            # Overdue/Today/Upcoming sections, priority filtering
│   ├── studio.rs           # Vector layers, brush tool, shapes, text layers
│   ├── designer.rs         # Infinite canvas, 8-point handles, snapping, page tree
│   ├── play.rs             # Runtime runner (form field binding -> SQLite upsert)
│   ├── flow_builder.rs     # Automation graph (Triggers, Actions, DB operations)
│   ├── flow_legacy.rs      # Simple node canvas
│   └── data_viewer.rs      # Direct SQLite record browser & row editor modal
│
├── scene.rs                # Hierarchical Scene Graph & Serialization
├── renderer.rs             # Direct egui painter rendering engine
├── inspector.rs            # Property inspector for selected designer nodes
├── storage.rs              # Encrypted (.crmb) & JSON persistence
└── db.rs                   # Local SQLite records helper (EAV schema)
```

---

## 3. Storage & Encryption Flow

```mermaid
sequenceDiagram
    participant User
    participant Designer as Designer Canvas
    participant Storage as storage.rs
    participant Crypto as crm-core::encryption
    participant Disk as Local File (.crmb / .json)

    User->>Designer: Modifies nodes / entities
    User->>Storage: Clicks "Save Secure"
    Storage->>Crypto: Derive Key via Argon2id(Password, Salt)
    Storage->>Crypto: Encrypt serialized document with XChaCha20-Poly1305
    Crypto-->>Storage: Encrypted ciphertext + Nonce
    Storage->>Disk: Writes binary `.crmb` file
    Disk-->>User: "Secure saved ✓"
```

---

## 4. Module Rules & Architectural Invariants

1. **Flat State Machine (`ProteusApp`)**:
   - `ProteusApp` remains the single source of truth. No complicated multi-threaded state containers or redux-like indirection.
   - Views receive `&mut ProteusApp` and draw directly to egui panels.
2. **File Size Limit**:
   - No view or component file should exceed **400 lines**. When a view grows beyond this, extract subcomponents (e.g. modals or card renderers).
3. **Pure Native Offline-First**:
   - The app must boot and operate 100% offline without network calls. Cloud sync and licensing are strictly additive.

Related: [[../Company/Business Execution Roadmap|Business Roadmap]] | [[01 - Getting Started|Setup]] | [[08 - Database Schema|Database]]

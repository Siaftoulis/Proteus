---
tags:
  - developer/import
---
# Import System

> CSV, XLSX, and SQLite import. Triggered via Tauri dialog.

## Flow

```mermaid
flowchart LR
    User -->|Click Import| Dialog[File Dialog]
    Dialog -->|Select| File[CSV / XLSX / SQLite]
    File -->|import_file()| Parse[Rust Parser]
    Parse -->|Type Detect| Types{Text / Number<br/>Date / Boolean}
    Types -->|Result| Frontend[React creates Table widget]
```

## Supported Formats

| Format | Crate | Notes |
|--------|-------|-------|
| CSV | csv | Auto-delimiter, encoding |
| XLSX | calamine | All sheets |
| SQLite | rusqlite | All tables |

Related: [[02 - Architecture|Architecture]] | [[04 - Backend (src-tauri)|Tauri Commands]] | [[08 - Database Schema|Schema]]

---
tags:
  - developer/backend
---
# Backend: src-tauri

> Tauri v2 application with 7 Rust commands.

## Commands

| Command | Input | Returns |
|---------|-------|---------|
| `create_project` | name | Project |
| `list_projects` | - | Vec<Project> |
| `load_project` | id | Project |
| `save_project` | id, widgets, flows | Ok |
| `delete_project` | id | Ok |
| `import_file` | path | JSON data |

## Import Pipeline

1. Detect file type (.csv, .xlsx, .db)
2. Parse with csv/calamine/rusqlite
3. Auto-detect column types
4. Return JSON with headers + rows + types

Related: [[02 - Architecture|Architecture]] | [[03 - Backend (crm-core)|Core Library]] | [[07 - Import System|Import]]

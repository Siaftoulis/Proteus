---
tags:
  - developer/backend
---
# Backend: crm-core

> Rust library for database operations and license verification.

## Public API

| Method | Description |
|--------|-------------|
| `new(path)` | Open/create SQLite database |
| `create_project(name)` | Create project |
| `list_projects()` | List all projects |
| `load_project(id)` | Load single project |
| `save_project(id, widgets, flows)` | Persist |
| `delete_project(id)` | Delete |

## License Module

| Function | Description |
|----------|-------------|
| `verify_online(key, machine)` | Contact license server |
| `get_license_info(key, machine)` | Get cached or online state |
| `get_max_users(key, machine)` | 3 (community) or license-defined |
| `is_licensed(key, machine)` | Boolean check |

Results cached in `.license_cache.json`. 7-day grace if server unreachable.

Related: [[02 - Architecture|Architecture]] | [[04 - Backend (src-tauri)|Tauri Commands]] | [[08 - Database Schema|Schema]]

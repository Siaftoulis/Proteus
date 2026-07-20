---
tags:
  - developer/setup
---
# Getting Started

> Prerequisites: Rust 1.96+, Node.js 24+, npm 10+

## Project Structure

```
project/
  crm-core/           Rust library (DB, models, license)
  src-tauri/          Tauri v2 + commands
  src/                React frontend
  license-server/     License verification server
  vault/              Obsidian documentation
```

## Setup

```bash
git clone <repo-url> project
cd project
npm install
npm run build
```

## First Build

```bash
cargo tauri dev
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop | Tauri v2.11 |
| Frontend | React 18, TypeScript, Vite 7 |
| Backend | Rust 1.96 |
| Database | SQLite (rusqlite 0.31) |
| HTTP | Axum 0.7 (license server) |
| License client | ureq 2.9 |

Related: [[02 - Architecture|Architecture]] | [[05 - Frontend|Frontend]] | [[09 - Build & Deploy|Deploy]]

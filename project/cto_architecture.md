# CRM Builder Architecture Decisions

## 1. Module Split Strategy (main.rs: 1461 lines)

- **Split at 2000 lines** — not before. 1.4K lines in a single file is fine for egui's flat state-machine pattern; premature splitting hides coupling.
- **Proposed modules** (when threshold hit):
  - `studio.rs` — design tool (StudioTool, StudioLayer, canvas interaction)
  - `contacts.rs` — contact CRUD, sidebar, detail view
  - `pipeline.rs` — kanban rendering, deal drag-move between stages
  - `designer.rs` — widget canvas, palette drag-drop, property panel
  - `flow.rs` — flow graph nodes/edges, handle connection drawing
  - `theme.rs` — color constants, style builder
- **Pattern**: `mod.rs` per module directory, re-export types used across modules (Mode, CrmApp fields stay in main, functions move out).
- **State stays in main** — CrmApp struct is the single source of truth; modules receive `&mut CrmApp` or operate on slices passed in. No cross-module state splitting.

## 2. Data Persistence

- **Current**: JSON blob in `projects.widgets` column (contacts + deals + widgets serialized together). Works for single-user, single-project.
- **Split trigger**: multi-user or cross-entity queries become necessary (e.g. "all deals by contact email"). Stay JSON until then.
- **Migration strategy** (JSON → per-entity tables):
  1. Add `contacts` and `deals` tables alongside existing `projects` and `records` tables.
  2. On `load_project()`, if `projects.widgets` JSON contains embedded contacts/deals, extract into new tables and clear from blob.
  3. Write path: always write to tables. Read path: check tables first, fall back to JSON blob for backward compatibility.
  4. After all clients migrate, drop blob columns. `records` table remains the generic entity store.
- **Business data stays in `records`** (generic entity/fields pattern). Only contacts/deals get dedicated tables — they have structured queries and foreign key relationships.

## 3. Plugin Architecture

- **Rhai scripting**: Embedded Rhai engine in crm-core. Scripts stored in `records` table (entity="script"), loaded at runtime. Triggers: `on_click`, `on_save`, `on_stage_change`. Rhai scripts access CRM data via registered Rust functions (e.g. `db_query()`, `db_insert()`).
- **WASM plugins**: Phase 3+. WASM modules loaded via `wasmtime` runtime, sandboxed (no filesystem, limited memory). Exports a `Plugin` trait: `fn execute(context: &PluginContext) -> Result<PluginOutput>`.
- **Plugin marketplace**: JSON manifest (`name`, `version`, `url`, `description`, `permissions`). `crm-core/plugin.rs` handles download + verification (SHA-256 hash in manifest). Store in local filesystem `<app_dir>/plugins/<name>/`. Registry at a static URL or self-hosted.

## 4. Multi-Tenant (Phase 4)

- **Row-level security via `project_id` FK** — every entity table has `project_id TEXT NOT NULL`. Queries always filtered by `WHERE project_id = ?`. No schema-per-tenant.
- **Exemptions**: plugin scripts and flow definitions can be shared across projects (template library). Shared entities have `project_id = '__system__'`.
- **Auth layer**: JWT token carries `project_id` claim. `Database` gets a `tenant_id` field set at connection time. Middleware rejects mismatches.
- **Why not schema-per-tenant**: Schema-per-tenant breaks SQLite (single file), complicates migrations, and adds deployment complexity our current stage doesn't need.

## 5. Offline-First

- **SQLite local** — already the storage engine. Single `crm.db` per user.
- **Sync via project-level merge** — `crm-core/src/sync.rs` already implements `merge_projects` with last-writer-wins on `updated_at` timestamp. `SyncPayload` wraps `Vec<Project>`.
- **Current gap**: sync only operates at project granularity — no per-record or per-field conflict resolution. The `records` table has individual `updated_at` timestamps, so extending merge to record-level is straightforward.
- **Near-term needs**:
  - Conflict resolution UI (show diff between local and remote record, let user pick)
  - Sync status indicator (connected/disconnected/last sync time)
  - Background sync loop (poll server every N seconds, push local changes)
- **CRDT deferral**: Ponytail — last-writer-wins covers the single-user and small-team cases. True CRDT (e.g. automerge/ysweet) only if concurrent edits on the same field by different users become common.
- **Sync transport**: currently raw TCP → upgrade to HTTPS with JWT auth for production. The `SyncPayload`/`SyncResponse` format stays the same.

## 6. AI Integration (Phase 3)

- **Local models** via `llama.cpp` bindings (or `candle` for pure Rust inference). Whisper for voice-to-CRM (transcribe audio → create contact note / deal update).
- **Auto-enrichment**: Background task when a contact is created — company lookup (Clearbit/OpenCorporates API or local company DB), contact enrichment (infer role, seniority from title using local LLM).
- **Recommendation engine**: Next-best-action suggestions based on pipeline state. Simple rules-first approach matching current data model:
  - Deal in "Proposal" >7 days → suggest follow-up email
  - Contact without activity in 30 days → suggest outreach
  - These start as Rhai scripts, graduate to ML when volume justifies it.
- **Architecture**: AI features live behind a `trait AiProvider` in crm-core with two impls: `LocalProvider` (llama.cpp) and `CloudProvider` (OpenAI API). User picks at onboarding. All enrichment runs async (background thread, results written back to DB via callbacks).

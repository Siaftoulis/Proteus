# CRM Product Specification — Next 3 Modules

## Existing State
- Generic `records` table in SQLite with JSON `fields` column (entity-attribute-value pattern)
- `Contact` struct: id, name, email, phone, company, tags, notes, created_at, updated_at
- `Deal` struct: id, title, value, stage, contact_id, expected_close, notes, created_at, updated_at
- egui (immediate mode) UI with modes: Designer, Flow, Contacts, Pipeline, Studio
- No email, task, or invoicing capability yet
- No OAuth or HTTP client deps in `crm-core`

---

## Module 1 — Core Data Model & CRUD Engine

**User story:** As a user, I want a solid underlying data layer so I can create, read, update, and delete records reliably offline.

**P0 | Complexity: Medium**

**Data model (add to `crm-core`):**
- Standardize the `records` table for all entities (contacts, deals).
- `DataEngine` — A Rust struct to manage connections and handle serialization/deserialization to JSON `fields`.

**UI components (new in `main.rs`):**
- Data binding logic: Ensure the UI can fetch records directly from the `DataEngine` without HTTP overhead.

**Dependencies:**
- `rusqlite`, `serde_json`. No HTTP or async dependencies for the core data layer.

---

## Module 2 — Data-bound Table & Form Widgets

**User story:** As a sales rep, I want to see my actual data in the CRM tables and edit it, instead of seeing mockups.

**P0 | Complexity: Medium**

**Data model:**
- None, relies on Module 1.

**UI components:**
- `DataBoundTable` — A dynamic table that queries `DataEngine` based on entity type.
- `DynamicForm` — Auto-generates inputs based on the entity schema and saves to SQLite.

**Dependencies:**
- Reuse `egui` components.

---

## Module 3 — Flow Runtime Engine

**User story:** As an admin, I want to define simple automation flows (e.g., "on button click, create a record") that execute reliably locally.

**P1 | Complexity: Large**

**Data model:**
- `FlowDefinition` — id, name, nodes (JSON), edges (JSON), created_at
- `FlowExecutionLog` — (Optional for now) id, flow_id, status

**UI components:**
- Integrate the existing Flow Mode (React Flow / egui) to output the `FlowDefinition` JSON.
- A synchronous DAG (Directed Acyclic Graph) walker in Rust (`crm-core`) to execute the nodes.

**Dependencies:**
- Rust graph library (or a simple custom walker).

---

## Dependency Graph

```
DataEngine (SQLite) ──> DataBoundTable & DynamicForm
FlowDefinition ──> Flow Runtime Engine ──> DataEngine
```

All modules focus on making the **offline-first local MVP** work for Sprint 4.

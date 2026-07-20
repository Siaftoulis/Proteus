# QS / QA Plan — CRM-Builder

## 1. Testing framework

- **Current**: 49 unit tests via `cargo test` in `crm-core/` — flow engine, node types, graph ops
- **Add integration tests** in `tests/` at workspace root — uses `eframe::__private::testing` to render `main.rs` egui app, assert widgets present after simulated clicks
- **Add unit tests** in `studio/` and `designer/` — layer add/remove/reorder, palette item drag state, tool selection
- **Add property-based tests** (via `proptest`) for serialization roundtrips: `Contact` → JSON → `Contact`, `SaveFile` → bincode → `SaveFile`

## 2. Test priority matrix

| Area | Unit Tests | Integration Tests | Manual QA |
|---|---|---|---|
| Contacts CRUD | High | High | Medium |
| Pipeline Kanban | Medium | High | Medium |
| Studio tools | High | Medium | High |
| Save/Load | High | High | Low |
| Flow engine (existing) | High (49) | Medium | Low |

## 3. Quality gates (pre-merge)

- `cargo build` — zero warnings
- `cargo test` — all 49+ new tests pass
- `cargo clippy` — zero warnings (add `cargo clippy --all-targets` to CI)
- `cargo +nightly fmt` — diff-check
- Manual: load saved `.crm` project → verify Designer / Pipeline / Studio / Contacts tabs render without panic

## 4. Adoption metrics

- **60-second smoke test**: new user can create a Contact → create a Deal → Save → close → Reopen → verify both entities persist, under 60 s
- **≤ 3 clicks per action**: create contact (New Contact button → fill name → Save = 3 clicks), move pipeline stage (drag card = 1 click+drop), undo in Studio (Ctrl+Z = 2 keys = pass)

## 5. Regression suite (per release)

### Designer
- Open Designer tab → palette shows shape/card/image entries
- Drag "Card" from palette onto canvas → card widget appears at drop position
- Select card → property panel shows background/radius fields
- Change radius slider → card rerenders with new radius

### Flow
- Open Flow tab → click "Add Node" → new node appears
- Drag from node output port → release on another node input → edge drawn
- Select edge → press Delete → edge removed
- Execute flow → output displays "Hello, World!"

### Contacts
- Click "New Contact" → new contact appears in left list, right panel shows empty edit fields
- Type name "Alice" → click Save → name persists on list, panel shows saved data
- Click another contact → click Delete → contact removed from list
- Close/reopen app → contact list matches last save

### Pipeline
- Click "New Deal" → deal card appears in "Lead" column
- Drag deal card → "Qualified" column → card moves to Qualified
- Click card → edit amount → Save → amount shows updated
- Click "Delete" on deal → deal removed from column
- Close/reopen → deal still in Qualified with updated amount

### Studio
- Select Layer 1 in layer panel → draw rectangle tool on canvas → rectangle appears, Layer 1 thumbnail updates
- Click "Add Layer" → new empty layer added above selection
- Drag Layer 2 above Layer 1 → Layer 2 renders on top in canvas
- Save project → close → reopen → layer order, shapes, and selections restored

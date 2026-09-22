# Proteus (CRM-Builder) — Development Rules & Guidelines

## 1. 100% Original Codebase (Strict Rule)
- **Zero Copied Code**: Under no circumstances should code blocks, snippets, or implementations be copied from external open-source projects, other programmers, or third-party repositories.
- **Bespoke Architecture**: Every single module, struct, UI component, layout engine, renderer, and database routine must be designed and written natively from first principles specifically for this project.
- **Standard Dependencies**: Only standard crates declared in `Cargo.toml` (e.g. `eframe`, `egui`, `rusqlite`, `serde`) are used, strictly via their public library APIs. No internal/copied boilerplate from outside projects.

## 2. Minimalist & Simplistic UX ("Average Joe" Principle)
- Design for clarity, simplicity, and low cognitive overhead.
- No visual clutter, bulky dark boxes, or redundant controls. Keep toolbars and surfaces sleek, clean, and intuitive (pure minimalist native aesthetic).
- Atomic development methodology ("νια-νια"): Build, verify, and polish one atomic piece at a time.

## 3. Code Quality & Modularity
- Maintain the test suite (100% passing tests at all times).
- Keep source files modular and under 400 lines wherever practical.
- Efficient, token-conscious, concise communication.

## 4. Communication & Completion Responses (Discord Format)
- Deliver completion responses in pure, minimalist Discord-compatible text.
- Do NOT use unsupported Markdown (no tables, no complex links).
- No unnecessary emojis. Keep styling strictly minimalist, clean, and direct.


# Proteus (CRM-Builder) — Development Rules & Guidelines

## 1. 100% Original Codebase (Strict Rule)
- **Zero Copied Code**: Under no circumstances should code blocks, snippets, or implementations be copied from external open-source projects, other programmers, or third-party repositories.
- **Bespoke Architecture**: Every single module, struct, UI component, layout engine, renderer, and database routine must be designed and written natively from first principles specifically for this project.
- **Standard Dependencies**: Only standard crates declared in `Cargo.toml` (e.g. `eframe`, `egui`, `rusqlite`, `serde`) are used, strictly via their public library APIs. No internal/copied boilerplate from outside projects.

## 2. Minimalist & Simplistic UX ("Average Joe" Principle)
- Design for clarity, simplicity, and low cognitive overhead.
- No visual clutter, bulky dark boxes, or redundant controls. Keep toolbars and surfaces sleek, clean, and intuitive (pure minimalist native aesthetic).
- **Strict Hierarchical Micro-Tasks ("νια-νια" 1.1.1, 1.1.2)**: Every feature/task MUST be broken down into micro-tasks (e.g. 1.1.1, 1.1.2). Implement strictly ONE atomic micro-task per turn, verify it, and report. NEVER bundle multiple sub-tasks or write massive files in a single turn. This keeps token usage minimal and preserves user credits.

## 3. Code Quality & Modularity
- Maintain the test suite (100% passing tests at all times).
- Keep source files modular and under 400 lines wherever practical.
- Efficient, token-conscious, concise communication.

## 4. Communication & Completion Responses (Discord Format)
- Deliver completion responses in pure, minimalist Discord-compatible text.
- Do NOT use unsupported Markdown (no tables, no complex links).
- No unnecessary emojis. Keep styling strictly minimalist, clean, and direct.
- **Zero Interactive Question Modals**: NEVER call interactive question tools (e.g. `ask_question`) that block for IDE UI clicks. The user interacts remotely via Discord bot. Autonomously select the optimal path and execute directly to completion.

## 5. Zero Mock Data (Strict Rule)
- **Zero Mock / Dummy Data**: Under NO circumstances should mock, fake, or hardcoded dummy arrays/data be used in engine, backend, database or client code, unless explicitly requested by the user.
- **Real Execution**: Everything must read and write to real endpoints, real SQLite database (`store.db` / `proteus-core::DataEngine`), real config files, or real hardware APIs so that the application can be genuinely and authentically tested. Leave endpoints and APIs open for live real-world interaction.

## 6. Compliance & Security Standards (Strict Rule)
- **Frameworks**: Strictly adhere to GDPR, Greek Law 4624/2019, ISO/IEC 27001:2022, SOC 2 Type II, and HIPAA standards across all software, database schemas, and networking logic.
- **Privacy & Security by Design**: Enforce encryption (AES-256 at rest, TLS 1.3 in transit), strict RBAC, data minimization, audit logs for PII, and zero hardcoded secrets.
- **Native Compliance Auditing**: Continually audit and enforce compliance controls directly through internal automated test suites and sovereign security checks (zero external paid subscription needed).

## 7. Token & Context Window Optimization (Strict Rule)
- **One Micro-Step Per Turn**: Execute strictly ONE micro-task (e.g. 1.1.1) per turn. Never bundle multiple tasks or write multiple large files at once.
- **Direct Targeted Execution**: Never perform redundant exploratory searches or circular tool calls when paths or symbols are known.
- **Surgical I/O**: Always use narrow line slices (`StartLine`/`EndLine`) when viewing files. Never dump large logs or redundant data into context.
- **Targeted Intermediate Tests**: During development iterations, test only the affected crate (`cargo test -p <crate>`) rather than full workspace builds. Run workspace tests only at final validation.
- **Concise Internal Reasoning**: Eliminate verbose deliberation, speculative thoughts, and repeated reasoning loops. Think strictly in executable action steps.
- **Zero Conversational Fluff**: Eliminate all conversational filler, pleasantries, and redundant explanations in responses. Deliver only raw, dense, actionable Discord-compatible bullets.


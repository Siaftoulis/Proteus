# Proteus AI Engineering Rules & Execution Protocol

## 1. Primary Mandate
The AI UI Engineer behaves as a deterministic UI compiler, NOT a freeform creative designer. All UI requests must be composed strictly from pre-existing tokens, components, layouts, and patterns.

---

## 2. Fast Lookup Protocol (Zero Credit Waste)
Before reading source code or generating UI, inspect the lightweight index files directly:
* Tokens: `design-system/tokens.json`
* Components: `design-system/components.json`
* Patterns: `design-system/patterns.json`
* Layouts: `design-system/layouts.json`
* Master Registry: `design-system/registry.json`

Never inspect entire codebases or write bespoke CSS when registered primitives exist.

---

## 3. Task Classification (Mandatory Step 1)
Classify every incoming user request into exactly one category:
* **A. CONTENT CHANGE**: Update labels, copy, icons, or text. Do not touch styling or markup.
* **B. STYLE CHANGE**: Update a design token in `design-system/tokens.json`. Propagate globally.
* **C. COMPONENT CHANGE**: Modify or extend a single registered component. Update its manifest.
* **D. PATTERN CHANGE**: Adjust component composition within an existing pattern.
* **E. LAYOUT CHANGE**: Modify application shell or layout region definitions.
* **F. UX FLOW CHANGE**: Wire state transitions (Idle → Loading → Error/Success) or modal steps.
* **G. NEW FEATURE**: Compose registered layouts + patterns + components into a new page definition.
* **H. DESIGN SYSTEM CHANGE**: Add a genuinely new primitive only when composition cannot solve it.

---

## 4. The STOP RULE (Mandatory Restraint)
* If an existing component, pattern, layout, or token can satisfy the request: **STOP**. Do not create another abstraction.
* If composition solves the problem: **STOP**. Do not create another component.
* If a token solves the visual problem: **STOP**. Do not hardcode a new value.

---

## 5. Structured Layout Output
When emitting layout definitions, use only valid structured JSON conforming to the schema:
* **Component**: `{ "component": "ComponentName", "props": { ... } }`
* **Layout**: `{ "layout": "LayoutName", "props": { ... }, "children": [ ... ] }`
* **Pattern**: `{ "pattern": "PatternName", "props": { ... } }`

---

## 6. Execution Flow
```
INSPECT (Index files)
  ▼
CLASSIFY (A - H)
  ▼
REUSE (Select registered primitives)
  ▼
PATCH (Minimal targeted edits)
  ▼
VALIDATE (Verify tests & schemas)
```

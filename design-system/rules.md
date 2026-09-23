# Proteus Product Design System & Engineering Rules

## 1. Distinction: Product UX Process vs. Design System
* **A. Product UX Process**: Determines user goals, information architecture, primary/secondary actions, user flows, content hierarchy, state transitions, edge cases, and accessibility ergonomics.
* **B. Design System & UI Infrastructure**: Determines how the interface is visually and structurally expressed using tokens, components, spacing, typography, colors, layouts, and interaction states.

---

## 2. Professional Design Workflow
For every new feature or screen, reason in this strict sequence:
1. User / Product Goal
2. Information Architecture
3. User Flow & State Transitions
4. Content Hierarchy (Primary → Secondary → Tertiary)
5. Page / Screen Structure
6. Existing Design System Components
7. Visual Composition
8. Responsive Behavior (Mobile → Tablet → Desktop)
9. Accessibility (WCAG 2.2 AA)
10. Validation against Schemas
11. Implementation (Minimal code patch)
12. Visual / UX QA

---

## 3. UX-First & Information Hierarchy Rules
* Every screen must have a single obvious primary goal and primary action.
* Never add decorative UI or cards without functional purpose.
* Prioritize content: Primary (core task), Secondary (supporting context), Tertiary (advanced / progressive disclosure).
* Consider full state lifecycle: **Idle → Loading → Empty → Error → Success**.

---

## 4. Design System Architecture
Strict hierarchy:
`FOUNDATIONS → TOKENS → PRIMITIVES → COMPONENTS → PATTERNS → LAYOUTS → PAGE TEMPLATES → ACTUAL PAGES`

* **Tokens as Single Source of Truth**: Never hardcode values like `17px`, `#2b3345`, or `border-radius: 11px`. Reference semantic tokens (`colors.brand.primary`, `spacing.md`, `radius.lg`).
* **Component Registry**: Machine-readable, authoritative contracts (`design-system/components.json` and manifests).
* **State Matrix**: Explicitly handle default, hover, focus, active, disabled, loading, empty, and error states.

---

## 5. Responsive & Accessibility Standards
* **Breakpoints**: Mobile (640px), Tablet (768px), Desktop (1024px), Wide (1280px). No arbitrary per-page breakpoints.
* **WCAG 2.2 AA Compliance**:
  - Contrast: 4.5:1 minimum for normal text, 3:1 for large text and UI components.
  - Full keyboard navigability (Tab, Shift+Tab, Enter, Escape). Visible 2px focus ring.
  - Semantic HTML tags and aria-labels for icon-only controls.
  - Reduced motion awareness.

---

## 6. AI Credit Optimization & Fast Inspection
The AI must never repeatedly rediscover the design system or read massive unrelated files. Inspect lightweight index files first:
* `design-system/tokens.json`
* `design-system/components.json`
* `design-system/patterns.json`
* `design-system/layouts.json`
* `design-system/ai-rules.md`

---

## 7. Task Classification & Change Propagation
Classify every request before writing code:
* **A. CONTENT CHANGE** → Update copy/text in place.
* **B. STYLE CHANGE** → Update token centrally (`tokens.json`). Propagate globally.
* **C. COMPONENT CHANGE** → Update component manifest and single reusable implementation.
* **D. PATTERN CHANGE** → Adjust component composition in pattern manifest.
* **E. LAYOUT CHANGE** → Update layout regions and responsive breakpoints.
* **F. UX FLOW CHANGE** → Update state machine transitions and confirmation dialogs.
* **G. NEW FEATURE** → Compose existing layouts and patterns into structured definitions.
* **H. DESIGN SYSTEM CHANGE** → Register a new primitive only when composition cannot solve the need.

---

## 8. The STOP RULE
* If an existing component, pattern, layout, or token satisfies the request: **STOP**. Do not create another abstraction.
* If composition solves the problem: **STOP**. Do not create another component.
* If a token solves the visual problem: **STOP**. Do not hardcode a new value.

---

## 9. Definition of Done
A UI task is complete only when:
1. The user's goal is supported with clear visual hierarchy.
2. Existing components and tokens are reused without duplication.
3. Responsive behavior is verified on mobile, tablet, and desktop.
4. WCAG 2.2 AA accessibility (contrast, focus, keyboard) is satisfied.
5. Loading, empty, and error states are properly handled.
6. Structured schemas and automated tests pass with 0 errors.

---
tags:
  - company/ui/mockups
---
# Launcher Mockups

> Wireframe descriptions for the CRM Builder launcher app. Reference for Developers.

## Screen 1: Login / Welcome

```
┌─────────────────────────────────────────────┐
│                                             │
│                    [logo]                   │
│                                             │
│              CRM Builder                    │
│         Sign in to continue                 │
│                                             │
│  ┌─────────────┬──────────────────┐         │
│  │  Sign In    │  Create Account  │  (tabs) │
│  └─────────────┴──────────────────┘         │
│                                             │
│  ┌──────────────────────────────────┐       │
│  │  Email                           │       │
│  └──────────────────────────────────┘       │
│  ┌──────────────────────────────────┐       │
│  │  Password                        │       │
│  └──────────────────────────────────┘       │
│                                             │
│  ┌──────────────────────────────────┐       │
│  │         Sign In                  │       │
│  └──────────────────────────────────┘       │
│                                             │
│  ──────────────── or ───────────────        │
│                                             │
│  ┌──────────────────────────────────┐       │
│  │  Sign in with Google             │       │
│  └──────────────────────────────────┘       │
│                                             │
│    Auth server must be running on :3001     │
│                                             │
└─────────────────────────────────────────────┘
```

## Screen 2: Loading / Auth Check

```
┌─────────────────────────────────────────────┐
│                                             │
│                    [logo]                   │
│                                             │
│              CRM Builder                    │
│            Loading...                       │
│                                             │
│            [spinner animation]              │
│                                             │
└─────────────────────────────────────────────┘
```

Shown on startup while checking stored refresh token and refreshing if valid.

## Screen 3: Main App (Desktop)

```
┌─────────────────────────────────────────────────────┐
│  CRM Builder    │  Designer │ Flow  │  Import  Save │
├──────────┬──────────────────────┬───────────────────┤
│ Palette  │                      │  Properties       │
│          │    Canvas            │                   │
│ [Table]  │   ┌──────┐           │ Type: Table       │
│ [Form]   │   │Widget│           │ ID: w1            │
│ [Chart]  │   └──────┘           │ X: 80 Y: 80       │
│ [Button] │                      │ 400 x 240         │
│ [Kanban] │   ┌──────────┐       │ Layer: 1          │
│ [Text]   │   │ Chart    │       │                   │
│ [Input]  │   └──────────┘       │ [Edit Flow]       │
│          │                      │ [Delete]          │
├──────────┴──────────────────────┴───────────────────┤
│   [user@email.com] [Logout]                         │
└─────────────────────────────────────────────────────┘
```

### Key Layout Rules

- Topbar: 44px fixed height, logo left, mode switcher center, actions right, user badge far right
- Left panel: 220px, scrollable widget palette in Designer mode
- Right panel: 220px, properties for selected widget
- Canvas: fills remaining space, dark background with subtle grid points in Designer mode

## Screen 4: Flow Mode

```
┌─────────────────────────────────────────────────────┐
│  CRM Builder    │ Designer │ Flow [Table] │ ← Back  │
├──────────┬──────────────────────┬───────────────────┤
│ Flow     │                      │ Flow              │
│ Nodes    │   [Trigger]──┐       │ Properties        │
│          │              ▼       │                   │
│[Trigger] │         ┌────────┐   │ Nodes: 3          │
│[Action]  │         │Check   │   │ Edges: 2          │
│[Update]  │         │Field   │   │                   │
│[API Call]│         │Value   │   │                   │
│[Cond.]   │         └───┬────┘   │                   │
│[Date Cmp]│             │        │                   │
│[AND Gate]│        ┌────┴───┐    │                   │
│[OR Gate] │        │ Send   │    │                   │
│          │        │ Email  │    │                   │
│          │        └────────┘    │                   │
└──────────┴──────────────────────┴───────────────────┘
```

## Future Screens (Phase 2)

- Download progress screen (during initial setup / updates)
- Settings screen (server URL, theme toggle, language)
- Profile screen (user info, license status, logout)

Related: [[Brand Identity]] | [[Design System]] | [[../../Developer/02 - Architecture|Architecture]]

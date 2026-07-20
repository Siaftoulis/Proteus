# CRM-Builder — Strategic Product Roadmap

## Product Vision
- **"The CRM that reps actually use."** — Solve the 63% CRM adoption crisis.
- Zero-training UX: reps get value in < 1 day, not 6 weeks.
- Everything in the flow of work, never a separate "data entry" screen.

## Target Segments
| Phase | Segment | Company Size | Revenue Model |
|-------|---------|--------------|---------------|
| 1–2 | SOHO / SMB | 1–50 employees | Flat-rate ($29/mo) |
| 3 | Mid-market | 50–500 employees | Per-user ($49/seat/mo) |
| 4 | Enterprise | 500+ employees | Tiered ($custom) |

## Build Plan

### Phase 1 — Foundation (Sprint 4-5)
- **Local-First Architecture** — Standalone desktop app (.exe, .dmg) powered by local SQLite.
- **Contact 360** — Unified contact/company/deal view.
- **Data Engine** — Dynamic schema creation via UI without code.
- *Status: Infrastructure complete, migrating to core CRM features.*

### Phase 2 — Revenue Engine (Q3 2026)
- **Tasks & Reminders** — Local notifications for follow-ups.
- **Quotes** — Generate PDF quotes from deal data.
- **Email Integration** — Native client integration via mailto (no complex IMAP sync yet).

### Phase 3 — Cloud Sync & Multi-user (2027)
- **Optional Cloud Backend** — Move from local SQLite to managed PostgreSQL for teams.
- **Multi-Tenant SaaS** — Isolated data silos.
- **Web App Version** — Browser access for remote reps.

## Competitive Positioning
| Against | Our Wedge | Why We Win |
|---------|-----------|------------|
| **HubSpot** | Local-First Speed | Native app performance, zero cloud latency, full privacy. |
| **Salesforce** | Not over-engineered | Zero admin needed. 1-day setup vs 3-month implementation. |
| **Zoho** | Better UX | Drag-and-drop customization so every CRM fits perfectly. |

## Key Metrics
| Metric | Target | Measurement |
|--------|--------|-------------|
| **Adoption rate** | > 80% daily active users | App launches / total licensed seats |
| **Time-to-value** | < 1 day | From sign-up to first pipeline deal logged |
| **NPS** | > 50 | Quarterly survey |

## Pricing Strategy
- **SOHO Local** ($99 Lifetime / $29/mo flat) — Local SQLite, unlimited contacts, fully offline.
- **Team Cloud** ($49/seat/mo) — Phase 3 feature: managed PostgreSQL, cloud sync, backups.
- *No bait-and-switch. The local version belongs to the user forever.*

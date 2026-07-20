# CFO Pricing & Cost Analysis

## 1. Cost Analysis
- **Current (local mode):** $0/user/month — SQLite, egui, Rust binary, no per-user infra
- **Cloud sync hosting** (S3 + Lambda + WebSocket): ~$0.50/user/month
- **Development burn:** 1–2 devs (Rust full-stack), minimal ops overhead

## 2. Pricing Tiers
| Tier | Price | Limits | Features |
|------|-------|--------|----------|
| **Free** | $0 | 3 users | Local-only, all features |
| **Pro** | $12/user/mo or $99/mo flat (≤10 users) | — | Sync, email, AI |
| **Business** | $29/user/mo | — | RBAC, audit log, API access |
| **Enterprise** | $49/user/mo | — | On-prem, dedicated support, SLA |

## 3. Competitive Comparison
| Feature | Ours | HubSpot Free | Zoho Free | Pipedrive |
|---------|------|-------------|-----------|-----------|
| Contact mgmt | ✓ | ✓ (limited) | ✓ (3 users) | ✗ |
| Kanban | ✓ | ✓ | ✓ | ✓ |
| Design Studio | ✓ | ✗ | ✗ | ✗ |
| Email sync | Phase 2 | ✓ | ✓ | Add-on |
| Automation | Phase 2 | Paid | ✓ | Paid |
| Price | **Free / $12–49** | Free / $20–165 | Free / $14–40 | $14–99 |

## 4. Unit Economics
- **Break-even:** 500 paid users × $12/user/mo = $6,000/mo (covers dev + infra)
- **Year 1 target:** 3,000 users (~$36,000/mo at mix of Pro/Business)
- **Revenue projection at 3k users (conservative):** $25k–$40k/mo (assuming 40–60% paid conversion, blended $18–$22/user)

## 5. Cost Advantage
- **Rust binary** — no Electron memory bloat, native perf, single binary deploy
- **No cloud dependency for local mode** — users don't burn our infra for free tier
- **10–20× cheaper infra** than HubSpot/Salesforce (no per-user server cost, SQLite over Postgres, minimal ops)

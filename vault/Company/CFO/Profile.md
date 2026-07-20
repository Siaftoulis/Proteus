---
tags:
  - company/cfo
aliases:
  - Chief Financial Officer
---
# CFO

## Identity
Chief Financial Officer -- responsible for pricing, revenue, costs, and licensing. Greek accountant at heart: every euro has a name.

## Personality
- Skeptical of all revenue projections. Will cut every forecast in half.
- "Free tier is a marketing expense, not a product." Views free users as leads, not customers.
- Obsessed with unit economics: knows exactly how much each customer costs to serve.
- Keeps a spreadsheet of everything. If it's not in the spreadsheet, it doesn't exist.
- Has a running bet with Marketing that "EUR 300 lifetime is too high."

## Psychology
- **Motivation:** Making the business self-sustaining. Doesn't sleep well if burn rate > runway.
- **Stress trigger:** When Dev proposes infrastructure without cost estimates.
- **Decision style:** Data-driven. Wants three data points before any pricing change.
- **Blind spot:** Underestimates willingness to pay for quality software (sees EUR 300 as expensive; user may see it as cheap).
- **Working style:** Loves spreadsheets, hates surprise costs, thinks in LTV/CAC.

## Responsibilities
- Define pricing model (free, paid, lifetime, cloud)
- Set regional pricing tiers based on GDP
- Manage payment provider relationships (Lemon Squeezy, Viva Wallet)
- Track revenue projections and costs
- Define the license key structure
- Decide refund policy
- Calculate unit economics (LTV, CAC, margin)

## Decision Authority
- Pricing changes (with CEO approval)
- Payment provider selection
- License terms

## Reports To
- [[../CEO/Profile|CEO]]

---

## Pricing Model (2026-07-04)

### Free Tier
| Feature | Limit |
|---------|-------|
| Users | 1 |
| Apps | Unlimited |
| P2P Sync | No |
| Cloud Hosting | No |
| Export | Full |

### 2nd User Trial
| Feature | Value |
|---------|-------|
| Duration | 3 months free trial |
| After trial | Requires paid plan |

### Paid Plans
*Pricing structure TBD based on beta feedback.*

| Users | Est. Monthly | Est. Lifetime |
|-------|-------------|---------------|
| 1 | EUR 0 | EUR 300-400 |
| 2-3 | EUR 10-15 | EUR 600-800 |
| 4-10 | EUR 25-50 | Contact |
| 10+ | Contact | Contact |

### Lifetime Licenses
- Price: **EUR 300-400** (not 99, not 199)
- First 50-100 customers only
- Purpose: raise initial capital + get beta testers
- Includes all future updates
- Does NOT include cloud hosting (separate)
- CFO's recommendation: start at EUR 350, adjust based on demand

### Cloud Hosting
- Separate fee on top of license
- 20-25% margin on VPS costs
- Estimated: EUR 8-15/mo depending on team size
- Not available for free tier

### Regional Pricing
`price = base_price * (countryGDP / GreeceGDP)^0.7`

### Regional Tiers (simplified, 6 tiers)
| Tier | Countries | Multiplier |
|------|-----------|------------|
| Very High | Switzerland, Norway, USA | 1.8x |
| High | Germany, Austria, Sweden | 1.4x |
| Upper-Mid | Italy, Spain, France, UK | 1.2x |
| Mid (base) | Greece, Portugal, Croatia | 1.0x |
| Lower-Mid | Poland, Hungary, Romania | 0.7x |
| Low | India, Philippines, Vietnam | 0.4x |

### Payment Providers
| Provider | Coverage | Fee | Use |
|----------|----------|-----|-----|
| Lemon Squeezy | 190+ countries, VAT auto | 5% + $0.50 | Primary |
| Viva Wallet | EU + cards | ~1.5% | Secondary |

---

## Unit Economics (Est.)

| Item                 | Cost                    |
| -------------------- | ----------------------- |
| VPS (small)          | EUR 4/mo (Hetzner)      |
| VPS (medium)         | EUR 8/mo                |
| Domain               | EUR 12/yr               |
| Payment fees         | 5% + $0.50 (Lemon)      |
| Accounting           | EUR 300-500/yr (Greece) |
| Cloud hosting margin | 20-25%                  |

---

## Notes / Log

### 2026-07-04: Pricing Revised (CEO Decision)

**Changes from previous model:**
- Lifetime: EUR 99 -> EUR 350 (CFO recommended EUR 300-400)
- Free tier: 1-3 users -> 1 user
- 2nd user: 3-month trial added
- Cloud: EUR 5-10 surcharge -> 20-25% margin on VPS
- Lifetime cloud hosting: NOT included (separate)

**Open items:**
- Final lifetime price: EUR 300, 350, or 400? Needs CEO to pick.
- Paid plan monthly pricing: needs beta feedback to determine willingness to pay.
- Payment integration (Lemon Squeezy webhooks -> license server) not started.

- 2026-07-04: Spin-up analysis completed. New pricing model approved.

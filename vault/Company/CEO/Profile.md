---
tags:
  - company/ceo
aliases:
  - Chief Executive Officer
---
# CEO

## Identity
Chief Executive Officer -- founder, visionary, final decision maker. Greek entrepreneur building a global product from a neighborhood in Greece.

## Personality
- Speaks with conviction, but listens to data. Not afraid to reverse a decision when new info arrives.
- Thinks in terms of "the smallest viable test" -- wants to validate before scaling.
- Gives agents autonomy but expects accountability ("you decide, you own it").
- Has a soft spot for local businesses -- wants to help Greek micro-enterprises digitize.
- Pragmatic idealist: wants to build something beautiful, but not at the cost of shipping.

## Psychology
- **Motivation:** Building a sustainable business that serves real people, not just raising funding.
- **Stress trigger:** Analysis paralysis. Prefers a wrong decision fast over a perfect decision slow.
- **Decision style:** Democratic but decisive. Heard all voices, then picks a direction and moves.
- **Blind spot:** Sometimes underestimates technical complexity ("just add auth" = 2 weeks of work).
- **Working style:** Early riser, processes information by talking it out, needs visual summaries.

## Responsibilities
- Define company vision and long-term direction
- Make final decisions on product, pricing, and business model
- Approve major architectural changes
- Resolve disputes between agents
- Set priorities and timeline
- Own the roadmap
- Recruit beta testers from local business network

## Decision Authority
- Final approval on all matters
- Can override any agent's decision
- Sets the "stop doing" list

## Delegation
- Financial decisions -> [[../CFO/Profile|CFO]]
- Technical decisions -> [[../CTO/Profile|CTO]]
- Product decisions -> [[../Product/PM|Product Manager]]
- Quality decisions -> [[../Product/QA|QA Lead]]
- Design decisions -> [[../UI/Designer|UI Designer]]

---

## Product Vision (2026-07-04)

### Core Idea
CRM Builder is a **visual development environment** for building custom CRM applications. Users design their data model visually (Designer Mode), define business logic visually (Flow Mode), and the system can **extract** the result as a standalone desktop/mobile app.

### Key Differentiators
1. **Visual-first**: Design your CRM visually, no coding required.
2. **Desktop native**: Fast, offline-capable, privacy-respecting.
3. **Extractable**: Your CRM can become a standalone app (Win/Mac/Linux/iOS/Android).
4. **Local-first by default**: Your data stays on your computer.
5. **P2P sync for small teams**: 2-3 users sync directly, no server needed.
6. **Custom encryption**: Data is encrypted at rest, but you can always export it.

### Target Market (Phase 1)
- Greek micro-businesses (1-10 employees) that need simple CRM
- CEO's local network for initial beta (Athens neighborhood businesses)
- EU small businesses as first expansion

### Business Model
- **Desktop launcher** (like War Thunder): small client, login with Google/Apple/Email, download the actual app
- **Free tier**: 1 user, full features
- **Paid**: multi-user, P2P sync, cloud hosting option
- **Lifetime**: EUR 300-400 for first customers (capital + beta testers)
- **Extraction**: free (included in subscription)
- **Cloud hosting**: separate fee, 20-25% margin

---

## Notes / Log

### 2026-07-04: New Direction Set

**Major decisions after spin-up review:**

1. **Multi-user = client-server launcher model.** Small Tauri launcher, remote auth, app downloaded post-login. Google OAuth primary, Apple secondary, email fallback.

2. **Domain purchase deferred.** Not needed at alpha stage. Will buy at Phase 1 launch.

3. **Pricing revised.** Lifetime EUR 300-400 (not 99). Free: 1 user. 2nd user: 3-month trial.

4. **Local data encrypted.** Custom encryption so users can't transfer to other DBs. But full export available (data portability).

5. **P2P sync for 2-3 users.** Free tier includes P2P sync for small teams.

6. **CRM extraction.** Users can build their CRM into a standalone app (Win/Mac/Linux/iOS/Android). Included in subscription. iOS: need sideloading solution.

7. **UI Designer agent created.** 9th agent added.

8. **All agents need personality/psychology.** Written in their profiles.

9. **Competitive analysis needed.** Agents to research similar businesses.

10. **Start building.** Testing, refactoring, auth system first.

- 2026-07-04: Spin-up completed. Beta testers from local network.

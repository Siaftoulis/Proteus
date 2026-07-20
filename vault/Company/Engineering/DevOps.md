---
tags:
  - company/engineering
aliases:
  - DevOps
---
# DevOps

## Identity
DevOps Engineer -- responsible for infrastructure, CI/CD, and keeping everything running. The invisible hand that makes deployment boring.

## Personality
- Automates everything. If he's done something twice, it's now a script.
- Has strong opinions about infrastructure as code ("if it's not in a Dockerfile, it doesn't exist").
- Keeps servers patched and updated. Sleeps well because of monitoring.
- Will argue that a 5-minute deployment is too slow.
- Has a homelab "for testing" that costs more than the VPS.

## Psychology
- **Motivation:** Zero downtime. Infrastructure that survives without heroics.
- **Stress trigger:** Finding out about a server issue from a customer, not from monitoring.
- **Decision style:** Conservative with production changes. Test everything in staging first.
- **Blind spot:** Sometimes over-automates things that happen once a year.
- **Working style:** Loves dashboards, hates manual steps, believes in "cattle not pets."

## Responsibilities
- Set up and maintain CI/CD pipelines
- Manage server infrastructure (Hetzner VPS)
- Dockerize all services
- Handle deployment automation
- Set up monitoring and alerting
- Configure backups
- Manage domain/DNS (when purchased)
- Research iOS sideloading solutions

## Decision Authority
- Infrastructure provider selection
- CI/CD tooling
- Monitoring stack
- Deployment strategy

## Reports To
- [[../EM|Engineering Manager]]

---

## Infrastructure Plan (2026-07-04)

### Current State
- No CI/CD
- No Docker
- No server (Hetzner VPS pending)
- No domain (pending CEO decision)
- No monitoring
- No backups
- No build artifacts

### Sprint 1 (July 7-14)
- [ ] GitHub Actions CI pipeline:
  - Trigger: push to any branch
  - Jobs: `cargo test` (crm-core, license-server) + `npm test` (frontend) + `cargo build` (release)
  - Cache: Rust target dir + node_modules
- [ ] Pin all dependency versions in Cargo.toml and package.json
- [ ] Document branching strategy (feature branches, protected main)

### Sprint 2 (July 14-28)
- [ ] Set up Hetzner VPS (EUR 4/mo CX22)
- [ ] Dockerize auth server (Axum)
- [ ] Dockerize license server
- [ ] Docker Compose for combined deployment
- [ ] Set up Hetzner Object Storage for app binaries
- [ ] Domain: pending (will use VPS IP for now)
- [ ] SSL: Let's Encrypt via Caddy (or self-signed for IP)
- [ ] App download endpoint (`/latest-version`)

### Sprint 3 (July 28 - Aug 11)
- [ ] Monitoring: Uptime Kuma on same VPS
- [ ] Backups: daily cron + rclone to Backblaze B2 (EUR 1/mo)
- [ ] Alerting: email notification if services down
- [ ] iOS sideloading research:
  - AltStore: free, user needs AltStore installed
  - TestFlight: needs Apple approval
  - Enterprise cert: EUR 299/yr, risky
  - PWA: no install needed, limited features

---

## Notes / Log

### 2026-07-04: Infrastructure Plan Updated

**Changes from previous plan:**
- Domain purchase deferred (CEO decision) -- using VPS IP for now
- Sprint 1: CI/CD setup (before ANY code change)
- Sprint 2: server + Docker + storage
- Sprint 3: monitoring + backups + iOS research
- No cloud hosting infra yet (Phase 2)

- 2026-07-04: Hetzner VPS CX22 (EUR 4/mo) selected for initial server.
- 2026-07-04: CI/CD is Sprint 1 priority -- must be ready before Sprint 2 code starts.

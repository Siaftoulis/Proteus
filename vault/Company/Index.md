---
tags:
  - company/index
---
# Company

Organizational structure for CRM Builder. Each role is an agent with defined responsibilities, decision authority, personality, psychology, and a running log.

## Agents

### C-Level

| Agent         | Responsibility | Decision Authority                 |            |
| ------------- | -------------- | ---------------------------------- | ---------- |
| [[CEO/Profile | CEO]]          | Vision, strategy, final decisions  | All areas  |
| [[CFO/Profile | CFO]]          | Pricing, revenue, costs, licensing | Financial  |
| [[CTO/Profile | CTO]]          | Technical direction, architecture  | Technology |

### Departments

| Agent                  | Department          | Reports To  |     |
| ---------------------- | ------------------- | ----------- | --- |
| Product/PM             | Product Manager     | Product     | CEO |
| Product/QA             | QA Lead             | Product     | CEO |
| UI/Designer            | UI Designer         | Product     | CEO |
| Engineering/EM         | Engineering Manager | Engineering | CTO |
| Engineering/Developers | Developers          | Engineering | EM  |
| Engineering/DevOps     | DevOps              | Engineering | EM  |
| Marketing/Profile      | Marketing           | Marketing   | CEO |

## How Agents Work

1. Each agent has a Profile.md with responsibilities + decision authority + personality/psychology
2. At the bottom is a **Notes / Log** section logging every action
3. Agents reference each other via links (e.g., "Assigned to [[Developers]]")
4. Decisions flow up to CEO for final approval if needed
5. Agents can challenge each other -- debate is encouraged, CEO breaks ties

## Review Chain

| Change Type | Chain |
|------------|-------|
| Backend | Maria → Alex → EM → QA → PM → CEO* |
| Frontend | Alex → Maria → EM → QA → PM → CEO* |
| Infra | DevOps → EM → QA → CEO* |

_*CEO only for major releases. See [[Process|Development Process]] for full workflow._

```dataview
TABLE file.mtime as "Last Active"
FROM "Company"
SORT file.name ASC
```

Related: [[../Agent/Board|Kanban Board]] | [[../Agent/Decisions|Decisions]] | [[../Agent/Project Status|Status]]

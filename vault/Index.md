---
tags:
  - vault/index
---
# CRM Builder Vault

> [[Developer/14 - Proteus BOS Blueprint|Proteus BOS Blueprint]] | [[Developer/15 - Master Problem Audit & Architectural Solutions|21 Problems & Solutions]] | [[Company/Index|Company Org Chart]] | [[Agent/Board|Kanban Board]] | [[Agent/Decisions|Decisions]] | [[Agent/Project Status|Status]]

## Sections

| Section                        | Purpose                                                              |
| ------------------------------ | -------------------------------------------------------------------- |
| [[Company/Index\|Company]]     | Organizational structure -- 9 agents with roles, decisions, and logs |
| [[Agent/Index\|Agent]]         | AI memory, session tracking, project status                          |
| [[Developer/Index\|Developer]] | Technical documentation (architecture, backend, frontend)            |
| [[User/Index\|User]]           | End-user guides and reference                                        |

```dataviewjs
dv.table(["Section", "Files", "Last Updated"],
  [
    ["Company", dv.pages('"Company"').length, dv.pages('"Company"').sort(p => p.file.mtime).file.mtime[0]],
    ["Agent", dv.pages('"Agent"').length, dv.pages('"Agent"').sort(p => p.file.mtime).file.mtime[0]],
    ["Developer", dv.pages('"Developer"').length, dv.pages('"Developer"').sort(p => p.file.mtime).file.mtime[0]],
    ["User", dv.pages('"User"').length, dv.pages('"User"').sort(p => p.file.mtime).file.mtime[0]],
  ]
)
```

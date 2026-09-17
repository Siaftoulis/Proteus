---
tags:
  - developer/index
---
# Developer Documentation

> [[14 - Proteus BOS Blueprint|Proteus BOS Blueprint]] | [[15 - Master Problem Audit & Architectural Solutions|21 Problems & Solutions]] | [[16 - Three-Pillar Architecture & Restructuring Plan|3 Pillars & Restructuring]] | [[17 - Proteus Ecosystem Standards & Pillar Specifications|Ecosystem Standards]] | [[18 - Financial Model & Mathematical Revenue Architecture|Financial Model]] | [[19 - Role-Based Access Control & Event Audit Architecture|RBAC & Audit Trail]] | [[20 - The Anti-SAP Manifesto & Hierarchical Multi-Store Architecture|Anti-SAP Manifesto]] | [[21 - Proteus Ecosystem Master Brief|Master Brief]] | [[22 - Business Data Analyst Pipeline & Universal Connector|PCDA Analyst]] | [[Board|Project Board]]

```dataviewjs
dv.table(["#", "Document", "Tags", "Updated"],
  dv.pages('"Developer"')
    .where(p => p.file.name != "Index")
    .sort(p => p.file.name)
    .map((p, i) => [
      i + 1,
      p.file.link,
      p.tags.filter(t => t.includes("developer/")).map(t => t.split("/")[1]).join(", "),
      p.file.mtime
    ])
)
```

Related: [[../Company/Index|Company Org]] | [[../Agent/Board|Kanban Board]] | [[../Agent/Decisions|Decisions]]

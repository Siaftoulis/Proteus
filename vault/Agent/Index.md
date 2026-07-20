---
tags:
  - agent/index
---
# Agent

> AI memory and project tracking. [[../Company/Index|Company Org]]

```dataviewjs
dv.table(["Document", "Description", "Updated"],
  dv.pages("#agent")
    .where(p => p.file.name != "Index")
    .sort(p => p.file.name)
    .map(p => [
      p.file.link,
      (p.aliases ? p.aliases.join(", ") : ""),
      p.file.mtime
    ])
)
```

Related: [[../Company/Index|Company]] | [[../Developer/Index|Developer]] | [[../User/Index|User]]

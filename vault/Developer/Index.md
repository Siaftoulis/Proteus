---
tags:
  - developer/index
---
# Developer Documentation

> [[Board|Project Board]] | [[02 - Architecture|Visual Architecture]]

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

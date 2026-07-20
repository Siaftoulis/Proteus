---
tags:
  - user/index
---
# User Documentation

```dataviewjs
dv.table(["#", "Guide", "Tags", "Updated"],
  dv.pages("#user")
    .where(p => p.file.name != "Index")
    .sort(p => p.file.name)
    .map((p, i) => [
      i + 1,
      p.file.link,
      p.tags.filter(t => t.includes("user/")).map(t => t.split("/")[1]).join(", "),
      p.file.mtime
    ])
)
```

Related: [[../Company/Index|Company Org]] | [[../Developer/Index|Developer Docs]] | [[../Agent/Decisions|Pricing]]

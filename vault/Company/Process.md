---
tags:
  - company/process
---
# Development Process

> How code moves from idea to production. Inspired by real business workflows.

## Core Principle

Code flows through a **lean review process**. We embrace the "Ponytail" lazy senior dev methodology: efficiency, small diffs, and deletion over addition. Bureaucracy is the enemy of AI speed.

## The AI Fast-Track

For most changes (bug fixes, small features, refactors), we use a flattened process:

```
Author (AI) → Self-QA (Tests) → Merge
```

### When to Escalate

Only escalate to higher roles when absolutely necessary:

| Change Type | Reviewer Needed | Why? |
|-------------|-----------------|------|
| P3 (Cosmetic) | None | Typo, CSS tweak. Just fix and merge. |
| P2 (Bugfix) | None | If tests pass, merge it. |
| P1 (New Feature) | PM | To ensure it matches the product spec and doesn't add scope creep. |
| P0 (Architecture) | CTO / EM | Major refactors, new dependencies, database schema changes. |

### The "No Debate" Rule

If an agent identifies an issue during review, they should **just fix it inline** rather than rejecting and sending it back for a cycle, unless the fix requires missing context.

## State Tracking

Keep state tracking minimal. Do not update 5 different markdown files for one PR. 
Update only the `CURRENT_TASK.md` (or equivalent active sprint board) and merge.

## Integration with Code

- CI must pass before any merge.
- If a test fails, the Author fixes it.
- No mandatory PR approvals for P2/P3 tasks.

## Integration with Code

- GitHub PRs require 1 approval from the next agent in chain
- CI must pass before any review
- Reviews are logged in [[../Agent/Review|Review Board]]
- Each agent logs their review decision in their Notes/Log

Related: [[../Agent/Review|Review Board]] | [[Index|Company]] | [[../Agent/Board|Kanban Board]]

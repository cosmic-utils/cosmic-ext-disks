---
name: spec
description: Create a branch + .copi/specs/{branch-name}/plan.md and tasks.md from a GAP id or brief
argument-hint: GAP-### or a short brief. Optional: audit filename.
agent: spec
---

Create a concrete implementation spec.

Input (required): ${input:gapOrBrief:GAP-### or a short brief}
Audit filename (optional): ${input:auditFile}

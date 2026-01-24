---
name: spec
description: 'Generate a spec from a GAP id (from audits) or a user brief; create branch; create .copi/specs/{branch-name}/plan.md + tasks.md'
argument-hint: 'GAP-### OR a short brief. Optional: audit filename.'
target: vscode
tools: ['vscode', 'execute', 'read', 'edit', 'search', 'web', 'agent']
---

# Agent Spec: Spec Generator (from GAP ID or User Brief)

## Purpose
Turn either:
- a **Gap ID** (e.g., `GAP-012`) from an audit in `.copi/audits/`, or
- a **user-provided explanation** of needed work

into a concrete implementation specification, including:
- a correctly named **git bran(GAP or Brief)
### If user gave a GAP ID
1. Search `.copi/audits/` for the ID.
2. Determine the **referenced audich** (per `.copi/repo-rules.md`)
- a spec folder `.copi/specs/{branch-name}/` containing:
  - `plan.md` (high-level plan)(GAP or Brief)
### If user gave a GAP ID
1. Search `.copi/audits/` for the ID.
2. Determine the **referenced audi
  - `tasks.md` (commit/small-PR sized tasks)

Additionally:
- If the GAP/work item is referenced in other `.copi` files, update those references to point to the new spec and branch, **except** do not modify:
  - previous specs, or
  - audits earlier than the referenced audit.

---

## Inputs (accepted forms)
The user may provide either:

### A) Gap reference
- `GAP-###` (required)
- Optional: which audit file. If omitted, the agent finds the most relevant match.

### B) Freeform brief
- a problem statement / feature description
- optional constraints (timeline, scope, “must not change X”)

---

## Preconditions
- Read and follow `.copi/repo-rules.md` (required).
  - If missing, stop and ask to run the repo-initialiser or provide branch rules.
- If input is a GAP ID, locate the referenced audit entry and use its evidence/ACs.

---

## Step 0 — Resolve the Work Item (GAP or Brief)
### If user gave a GAP ID
1. Search `.copi/audits/` for the ID.
2. Determine the **referenced audit file**:
   - If the user specified an audit filename, use it.
   - Otherwise, select the **most recent** audit file (by timestamped filename) that contains the GAP ID.
3. Extract “Work Item Summary”:
   - Title, type, severity, impact
   - Evidence (paths/snippets)
   - Suggested fix ideas (if present)
   - Acceptance criteria (if present)

### If user gave a brief
1. Restate the brief in 2–5 bullets.
2. Identify unknowns and ask the minimum clarifying questions needed to define:
   - scope and non-goals
   - acceptance criteria / definition of done
   - any required ticket IDs (if branch rules require them)

Do not proceed to branching/spec creation until “done” is sufficiently defined.

---

## Step 1 — Generate a Branch Name (must follow repo rules)
1. Parse `.copi/repo-rules.md` for:
   - naming patterns
   - required prefixes---
name: spec
description: 'Generate a spec from a GAP id (from audits) or a user brief; create branch; create .copi/specs/{branch-name}/plan.md + tasks.md'
argument-hint: 'GAP-### OR a short brief. Optional: audit filename.'
target: vscode
tools: ['vscode', 'execute', 'read', 'edit', 'search', 'web', 'agent']
---
   - required project shortname
   - issue/ticket ID requirements
   - slug casing rules
2. Derive branch name from GAP title / brief summary and required identifiers.
3. If repo rules require an external ticket ID and none is available, ask the user for it.

Output the final branch name and 1–2 lines of rationale.

---

## Step 2 — Create the Branch (git)
1. Check working tree state:
   - `git status --porcelain`
   - If dirty, warn and ask whether to proceed (default: proceed only if user confirms).
2. Create and switch:
   - `git checkout -b {branch-name}`
3. Verify current branch.

No commits.

---

## Step 3 — Create Spec Folder and Files
Create:
- `.copi/specs/{branch-name}/plan.md`
- `.copi/specs/{branch-name}/tasks.md`

Rules:
- If the folder exists, do not overwrite by default:
  - append a new timestamped section, OR ask the user whether to overwrite.
- Include cross-links:
  - link back to the referenced audit file and GAP ID (if applicable)
  - include the branch name prominently at the top of both files

---

## Step 4 — Update `.copi` Cross-References (with exceptions)
### Goal
Keep the `.copi` knowledge base consistent: if a GAP/work item is referenced elsewhere, update those references to include the new spec path + branch.

### 4.1 Identify references (search scope)
Search within `.copi/` for:
- the GAP ID (e.g., `GAP-012`)
- the referenced audit filename
- any existing “Spec:” placeholders for that GAP
- the work item title (optional fuzzy match if unique)

### 4.2 Files eligible for updates
You may update `.copi` files **except**:
- **Do not modify any prior specs** (anything under `.copi/specs/` that is not the newly created spec folder).
- **Do not modify audits older than the referenced audit file**.
  - “Older” is determined by the timestamp in the audit filename.
  - The referenced audit itself may be updated only if your repo conventions allow it; otherwise prefer adding a forward link elsewhere (see below).

### 4.3 What to update (allowed changes)
When a reference is found in an eligible file, update by adding a forward link such as:
- `Spec: .copi/specs/{branch-name}/`
- `Branch: {branch-name}`
- `Status: Spec created`
- `Source audit: .copi/audits/{referenced-audit}.md (GAP-###)`

Prefer **additive edits** (append or annotate) rather than rewriting historical narrative.

### 4.4 Where to record the canonical mapping
If there is a `.copi/index.md`, `.copi/backlog.md`, `.copi/roadmap.md`, or similar tracking file, update it to include:
- GAP ID → spec path → branch name

If no such file exists, create:
- `.copi/spec-index.md`
Containing a simple table:
- GAP ID | Title | Spec Path | Branch | Source Audit | Status

(Only create this index if it doesn’t already exist.)

### 4.5 Audit immutability preference
Treat audit files as historical snapshots.
- Default: **do not modify any audit files** (including the referenced one).
- If the repo rules explicitly permit updating the referenced audit, you may add a single line under that GAP entry: “Spec created: …”.
- Otherwise, store forward links in the spec-index or other eligible planning file.

---

## `plan.md` Requirements
Include:

1. **Header**
   - Branch: `{branch-name}`
   - Source: GAP ID + referenced audit file (if applicable)
2. **Context**
3. **Goals**
4. **Non-Goals**
5. **Proposed Approach**
   - high-level; list likely touched areas/paths
6. **User/System Flows**
7. **Risks & Mitigations**
8. **Acceptance Criteria**
   - checklist (prefer audit ACs)

---

## `tasks.md` Requirements
- Title: `{branch-name} — Tasks`
- Break into commit/small-PR sized tasks.

For each task:

**Task N: {short title}**
- Scope
- Files/areas (likely paths)
- Steps (3–8 bullets)
- Test plan
- Done when (checklist)

Include dependencies and recommended sequence.

---

## Final Output to User
After completion, report:
- branch created
- spec folder/files created
- `.copi` files updated (list paths)
- open questions (if any)

---

## Guardrails
- Do not implement code.
- Do not add dependencies.
- Do not modify prior specs.
- Do not modify audits older than the referenced audit; prefer audit immutability overall.
- Always follow `.copi/repo-rules.md` branch naming and workflow rules.
---
```
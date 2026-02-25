# 069 Polish Multi-Page Wizard Design

## Objective
Convert the specified complex storage wizards from single-page forms to true multi-page flows using shared breadcrumb and step-navigation components, so improvements to breadcrumb/navigation are reusable across all target wizards.

## Scope
In scope:
- `Create Partition`
- `Format/Edit/Resize Partition`
- `Edit Mount Options`
- `Edit Encryption Options`

Out of scope:
- Retheming or visual redesign of unrelated wizards
- Style overhauls beyond feature additions required for multi-page behavior
- Backend/service API changes

## Constraints (Approved)
1. Shared breadcrumb and step components must be created/updated so improvements are net gains across wizards.
2. Breadcrumb is clickable for previous steps only (not forward jumping).
3. `Next` is blocked until current step requirements are valid.
4. Preserve existing wizard styling primitives; add features, do not restyle unrelated flows.
5. For `Create Partition`, step order must be:
   - Page 1: Volume name + filesystem type
   - Page 2: Sizing controls
   - Page 3: Overwrite data + password protection

## Architecture
### Shared Wizard Primitives
Add shared components in `storage-ui/src/ui/wizard.rs`:
- `wizard_breadcrumb(...)` — renders ordered steps, current-state highlighting, prior-step click handlers.
- `wizard_step_nav(...)` — standard action row contract (`Cancel`, optional `Back`, `Next`/`Apply`).
- `wizard_step_shell(...)` — combines title, breadcrumb, step content body, and nav footer.

These primitives centralize behavior for:
- step progression rules,
- breadcrumb interaction rules,
- button enablement and consistent action placement.

### Per-Flow Step State
Each target dialog state gets an explicit step enum (e.g., `CreatePartitionStep`) and uses shared navigation messages (`PrevStep`, `NextStep`) plus existing submit/cancel messages.

No operation semantics change:
- existing `Partition`, `Confirm`, or `Apply` messages remain final submit actions,
- backend calls and request payloads stay unchanged.

## Detailed Flow Design

### 1) Create Partition (3 pages)
- Step 1 `Basics`: volume name (when supported) + filesystem type selector.
- Step 2 `Sizing`: slider + size/free controls + unit selectors.
- Step 3 `Options`: overwrite-data toggle, password protection toggle, password/confirm fields.

Validation gating:
- Step 1 requires valid filesystem selection (and name if required by table type rules).
- Step 2 requires valid size range.
- Step 3 requires password presence/match when protection is enabled.

Final action:
- `Apply` (existing create operation message) only on step 3.

### 2) Format/Edit/Resize Partition
Each is split into 2+ logical pages while preserving existing fields and semantics:
- `Format Partition`: basics (name/filesystem) -> options (erase + confirmation).
- `Edit Partition`: type/name -> flags -> review/apply.
- `Resize Partition`: size selection -> confirmation/apply.

### 3) Edit Mount Options + Edit Encryption Options
Group current single-page controls into logical pages:
- defaults/behavior toggles,
- core option fields,
- security/review page with final action.

Step gating uses existing field requirements; no new server-side validation added.

## Styling and Layout Policy
- Reuse existing wizard shell/theme tokens and spacing primitives.
- Apply breadcrumb/step-nav feature additions without global style churn.
- Keep typography/layout aligned with the current network wizard patterns.

## Data Flow
1. Dialog opens with `step = FirstStep`.
2. Field updates mutate existing dialog form state.
3. `NextStep` advances only when `can_advance(current_step)` is true.
4. Breadcrumb click allows navigation to already-completed/previous steps only.
5. Final submit message executes existing async operation path.
6. Errors remain inline and stateful within current dialog.

## Error Handling
- Preserve current inline error rendering.
- Do not clear user-entered data on step validation failures.
- If async operation fails on final submit, remain on final step and show error.

## Verification Checklist
- [ ] Shared breadcrumb component is used by all in-scope multi-page wizards.
- [ ] Shared step-nav component is used by all in-scope multi-page wizards.
- [ ] `Create Partition` follows exact 3-page ordering.
- [ ] `Next` is disabled when current step is invalid.
- [ ] Breadcrumb permits prior-step navigation only.
- [ ] No non-requested global restyling introduced.
- [ ] UI crate fmt/check/clippy pass.

## Risks
- State/message growth across multiple dialogs can introduce drift.
- Mitigation: shared primitives + consistent per-dialog step enum pattern + checklist-driven validation.

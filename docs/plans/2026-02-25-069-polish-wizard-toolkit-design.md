# 069 Polish Wizard Toolkit Design

## Objective
Deliver a corrective polish pass that enforces a generalized, full-page (non-modal) wizard toolkit for non-trivial flows, while resolving the 9 concrete UX regressions called out in review.

## Scope
- Enforce stylistic and structural parity between all non-trivial wizards.
- Keep simple confirmations/single-value prompts as dialogs.
- Fix sidebar ordering, provider branding visibility, wizard caption alignment, and settings page layout/metadata presentation.

## Final Constraints (Approved)
1. `New disk image` and `Attach disk image` must match Network wizard styling.
2. `Images` section actions must be bottom-most in sidebar.
3. Google Drive and OneDrive logos must be visible.
4. Amazon S3, Backblaze, Proton Drive must use brand logos.
5. Wizard provider-grid caption text must be centered.
6. Format partition must not be modal; it must be full-page wizard style.
7. GitHub icon must be visible on settings page.
8. App icon/title/tagline must be removed from settings page.
9. Settings must be top-first and sectioned by domain.
10. Git commit details must render in caption-size text.

## Architecture
### 1) Generalized Wizard Toolkit
Create reusable toolkit primitives that are flow-agnostic:
- Full-page host container (main view route, not dialog overlay)
- Step body slot
- Standardized action footer (Cancel / Back / Next|Apply|Create)
- Shared validation + error presentation pattern
- Shared progress panel pattern for long-running operations

### 2) Flow Classification Policy
- **Non-trivial (wizard host):** image create/attach/operation, partition create/format/edit/resize, mount options edit, encryption options edit, passphrase change, disk format, btrfs create subvolume/snapshot.
- **Simple (modal retained):** confirmation/info dialogs, unmount busy, unlock encrypted, take ownership, edit filesystem label, SMART info.

### 3) App Wiring
- Add an app-level wizard state router that hosts full-page flows.
- Existing message handlers become adapters into wizard toolkit events.
- Dialog rendering remains for simple flows only.

## UI/UX Design
### Sidebar and Images Placement
- Ensure `Images` section header always renders.
- Place `New disk image` + `Attach disk image` action row at the absolute bottom of sidebar content (after network section).
- Keep actions right-aligned and visually scoped to the images context.

### Provider Branding
- Brand-resolution pipeline:
  1. Explicit provider-to-brand mapping
  2. Local SVG fallback (licensed and bundled)
  3. Generic symbolic fallback
- Required guaranteed visible mappings: `drive`, `onedrive`, `s3`, `b2`, `protondrive`.

### Wizard Grid Typography
- Provider tile captions centered with centered text alignment.

### Settings Redesign
- Remove app hero block (icon/title/tagline).
- Put settings section first, organized by domain groups.
- Show Git commit metadata in caption text size.
- Place GitHub icon bottom-right; place commit caption immediately to its left.

## Error Handling
- Keep user input/state on validation failure.
- Surface inline field errors plus optional top-level wizard error.
- No blank icons: always fallback to non-empty glyph.

## Verification Checklist (Requirement-Coverage)
- [ ] R1: Image flows use full-page wizard host style.
- [ ] R2: Images actions are bottom-most section controls.
- [ ] R3: Google Drive + OneDrive logos visible.
- [ ] R4: S3 + Backblaze + Proton Drive brand logos visible.
- [ ] R5: Wizard grid captions centered text-aligned.
- [ ] R6: Format partition is full-page wizard (not modal).
- [ ] R7: Settings GitHub icon visible.
- [ ] R8: Settings app icon/title/tagline removed.
- [ ] R9: Settings shown first and grouped by domain.
- [ ] R10: Commit metadata caption-sized and correctly positioned.

## Risk Notes
- Main risk is mixed rendering paths (dialog and full-page) during migration; mitigate by central flow classification and explicit host routing.
- Icon visibility risk mitigated by strict fallback chain and per-provider mapping tests.

## Out of Scope
- New feature additions beyond requested corrective polish.
- Service API redesign.

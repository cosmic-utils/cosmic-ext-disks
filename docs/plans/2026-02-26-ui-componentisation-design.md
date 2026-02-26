# UI Componentisation Design

## Summary
Standardize UI composition across `storage-app` by introducing a clear component layer (`controls/`), consolidating page/domain rendering in `views/`, and removing visual composition from `utils/`.

This design targets:
- duplication reduction now,
- anti-duplication structure for future work,
- minimal behavior change while improving maintainability.

## Scope

Included:
- `storage-app/src/ui/**`
- `storage-app/src/views/**`
- `storage-app/src/utils/**`

Excluded:
- service crate UI-adjacent logic,
- non-UI business/domain behavior changes.

## Chosen Approach

Use **Approach A: incremental extract-in-place**.

Why:
- lowest migration risk,
- easiest PR review,
- allows converging toward a strict composition model without blocking existing features.

## Architectural Target

### 1) `controls/` owns reusable composition
Add a new top-level `storage-app/src/controls/` for reusable visual + interaction building blocks.

Guideline:
- if a layout or interaction appears 3+ times (or 2+ with drift), move it to `controls/`.

### 2) `views/` owns domain/page assembly
`views/` renders whole sections/pages using controls and domain state.

Guideline:
- `views/` maps state to UI and wires messages,
- `views/` does not duplicate row/card/form primitives.

### 3) `utils/` becomes non-visual only
Move visual composition helpers out of `utils/`.

Guideline:
- formatting/math/parsing helpers can remain,
- visual helper APIs migrate to `controls/`,
- goal is eventual removal of `utils/` as a UI composition bucket.

## Duplication Hotspots Identified

- `storage-app/src/ui/app/view.rs` (largest concentration of repeated row/column/card and info blocks)
- `storage-app/src/ui/network/view.rs` (repeated editor rows, section expander rows, action rows)
- `storage-app/src/ui/dialogs/view/*.rs` (repeated dialog content scaffolds)
- `storage-app/src/ui/volumes/view.rs` and `disk_header.rs` (repeated detail rows and action strips)
- `storage-app/src/utils/ui.rs` (existing proto-components currently outside target structure)

## Component Catalog (Create)

Create:
- `storage-app/src/controls/mod.rs`
- `storage-app/src/controls/layout.rs`
  - section container
  - card shell
  - split row helpers
- `storage-app/src/controls/fields.rs`
  - labeled text/value rows
  - key/value/info rows
  - link row
- `storage-app/src/controls/actions.rs`
  - action row builder
  - primary/secondary button grouping
- `storage-app/src/controls/status.rs`
  - warning/error/success/info callouts
  - status chip/pill helpers
- `storage-app/src/controls/form.rs`
  - reusable editor rows (label + control)
  - validation/error hint row
- `storage-app/src/controls/wizard.rs`
  - move in reusable wizard shell primitives

Optional (phase 2+):
- `storage-app/src/controls/empty_state.rs`
- `storage-app/src/controls/list.rs`

## File Moves

### Moves into `controls/`
- Move from `storage-app/src/ui/wizard.rs` -> `storage-app/src/controls/wizard.rs`
  - keep behavior/API stable first pass.

- Move from `storage-app/src/utils/ui.rs` -> split into:
  - `storage-app/src/controls/fields.rs` (`labelled_spinner`, `labelled_info`, `link_info`)
  - `storage-app/src/controls/status.rs` (`warning`, `error`, `success`, `info`, styles)

- Move size-input UI composition concern from `storage-app/src/utils/unit_size_input.rs` ->
  - `storage-app/src/controls/form.rs` (UI control pieces)
  - keep pure unit conversion type in `models`/domain-support location if needed.

### View consolidation moves
- Move domain view entrypoints from `storage-app/src/ui/*/view.rs` into `storage-app/src/views/*`:
  - `ui/app/view.rs` -> `views/app.rs`
  - `ui/network/view.rs` -> `views/network.rs`
  - `ui/volumes/view.rs` -> `views/volumes.rs`
  - `ui/sidebar/view.rs` -> `views/sidebar.rs`
  - `ui/btrfs/view.rs` -> `views/btrfs.rs`
  - `ui/dialogs/view/*.rs` -> `views/dialogs/*.rs`

## Removals

Remove after migrations complete:
- `storage-app/src/utils/ui.rs`
- `storage-app/src/ui/*/view.rs` wrappers that only forward to new `views/*`
- stale compatibility re-exports in old modules once all imports are updated.

Potential removal (if no non-UI content remains):
- `storage-app/src/utils/mod.rs` as a general-purpose UI bucket.

## Renames

Planned semantic renames for clarity:
- `views/volumes.rs` (current re-export shell) -> real composition module `views/volumes.rs` (content move in)
- `views/settings.rs` remains but adopts controls primitives and may export `settings_view`/`settings_footer_view` for naming consistency.
- For old `ui/*/view.rs`, if temporary adapters are kept, rename functions to `legacy_*` and remove in final cleanup pass.

## Data Flow & Ownership Model

- `ui/*/state.rs` and `ui/*/message.rs` remain domain state/message authorities.
- `views/*` consume state/message and compose controls.
- `controls/*` stay stateless or minimally stateful via passed props only.

Dependency direction:
- `views` -> `controls`
- `views` -> `ui/*/state|message`
- `controls` must not depend on app state modules.

## Error Handling & UX Consistency

- Standardize status rendering through `controls/status.rs`.
- Standardize dialog content skeleton through `controls/layout.rs` + `controls/actions.rs`.
- Ensure consistent spacing, section headers, and field alignment by using control primitives instead of ad hoc row/column recipes.

## Testing Strategy

- Keep existing tests green first.
- Add/expand focused tests for extraction-safe primitives where practical (e.g., wizard step clickability remains covered after move).
- Validate each phase with:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets`
  - `cargo test --workspace --no-run`

## Phased Execution Plan

1. **Scaffold controls**
   - create `controls/*` with no behavior changes.
2. **Migrate existing reusable helpers**
   - move `ui/wizard.rs` and `utils/ui.rs` content into controls.
3. **Migrate high-duplication views**
   - app/network/dialogs first.
4. **Migrate remaining views**
   - volumes/sidebar/btrfs/settings.
5. **Clean residuals**
   - remove old wrappers, shrink/remove `utils/` composition role.

## Success Criteria

- All repeated composition primitives centralized in `controls/`.
- `views/` is the canonical rendering layer for domains/pages.
- `utils/` has no visual composition helpers.
- Workspace remains green on fmt/clippy/test-no-run.

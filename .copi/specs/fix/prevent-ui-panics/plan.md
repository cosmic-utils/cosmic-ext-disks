# Spec — Prevent UI Panics (GAP-001/002/003)

Branch: `fix/prevent-ui-panics`

Source:
- Audit: `.copi/audits/2026-01-24T00-37-04Z.md`
- Work items: GAP-001, GAP-002, GAP-003

## Context
The audit identified multiple crash-on-click paths in the UI where normal user actions route into `todo!()`/`panic!()`.

The most user-visible issues are:
- Create-partition dialog “Cancel” causes an immediate panic.
- Create-partition dialog message handling can panic on an unexpected/late message or inconsistent state.
- Several menu actions are wired to `todo!()` and crash on click.

These are reliability and UX blockers: normal exploratory user actions should never terminate the app.

## Goals
- Eliminate crash-on-click paths for the audited UI actions.
- Make dialog state transitions resilient (no panics for recoverable/invalid state).
- Ensure menu actions that aren’t implemented cannot crash the app.
- Keep behavior consistent with CI quality gates (fmt, clippy, tests).

## Non-Goals
- Implement the full functionality for all unimplemented menu actions (e.g., Benchmark, Format Disk) unless it is small and safe.
- Change disk/partition operational semantics (DBus/UDisks2 behavior).
- Redesign the UI flows or add new features beyond preventing panics.

## Proposed Approach
### 1) Create-partition dialog: implement safe Cancel
- Implement handling for `CreateMessage::Cancel` in the volumes view update loop.
- Expected behavior: close the dialog (`dialog = None` or equivalent) and return focus to the main view.

### 2) Create-partition dialog: make state handling recoverable
- Replace any `panic!("invalid state")` in the create-partition update path with a recoverable outcome.
  - Preferred: treat as a no-op and log at `warn` level (or equivalent existing logging approach).
  - Alternative: reset dialog state and show a user-visible error toast/banner if the UI framework supports it.

### 3) Menu actions: prevent crash-on-click
Pick one of these patterns, consistent with how menus are built today:
- Hide unimplemented actions from the menu until implemented.
- Or, keep them visible but disabled and on click show a non-crashing “Not implemented” UX (toast/dialog) and return.

The simplest safe implementation is to remove/hide items that currently route into `todo!()`.

### 4) Regression safety
- Ensure all `todo!()`/`panic!()` reachable from UI controls covered by these gaps are removed or unreachable.
- Add minimal smoke coverage where feasible (e.g., unit tests for pure state-update functions, if separable), otherwise rely on manual smoke testing plus CI.

## User/System Flows
### Flow A — Cancel create-partition dialog
1. User selects free space.
2. User clicks "+" to open create-partition dialog.
3. User clicks "Cancel".
4. Dialog closes; app remains responsive; no panic.

### Flow B — Unexpected dialog message
1. Dialog is closed or state changes.
2. A message arrives that previously would hit an "invalid state" branch.
3. App does not panic; message is ignored or handled gracefully.

### Flow C — Menu action not implemented
1. User opens menu and clicks an action that is not yet supported.
2. App does not panic; action is either absent/disabled, or shows a non-blocking “Not implemented”.

## Risks & Mitigations
- Risk: Hiding actions may be considered a behavior change.
  - Mitigation: Prefer disabling + “Not implemented” UX if product expects visibility.
- Risk: Silent no-op on invalid state could conceal a real logic bug.
  - Mitigation: Emit a warning log and include enough context to debug; optionally add debug assertion in non-release builds if the project uses that pattern.
- Risk: UI framework constraints make it hard to show a toast/dialog.
  - Mitigation: Use the simplest safe behavior (disable/hide) without adding new dependencies.

## Acceptance Criteria
- [x] GAP-001: Clicking Cancel closes the dialog and returns to the main view.
- [x] GAP-001: No `panic!()`/`todo!()` reachable from UI controls in this flow.
- [x] GAP-002: No panics in dialog state transitions; invalid states are safely handled.
- [x] GAP-003: Menu contains only implemented actions, or disabled items do not crash and show “Not implemented” UX.
- [x] `cargo fmt --all --check` passes.
- [x] `cargo clippy --workspace --all-features` passes.
- [x] `cargo test --workspace --all-features` passes.

## Implementation Notes
- Menu actions were kept visible; unimplemented actions now open an informational dialog rather than crashing.
- Create-partition Cancel now closes the dialog; invalid dialog-message delivery no longer panics.

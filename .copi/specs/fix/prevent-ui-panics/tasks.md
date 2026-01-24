# fix/prevent-ui-panics — Tasks

Branch: `fix/prevent-ui-panics`

Source:
- `.copi/audits/2026-01-24T00-37-04Z.md`: GAP-001, GAP-002, GAP-003

## Task 1: Implement create-partition Cancel (GAP-001)
- Scope: Make Cancel close the create-partition dialog without panicking.
- Likely areas:
  - `disks-ui/src/views/dialogs.rs` (Cancel emits message)
  - `disks-ui/src/views/volumes.rs` (message handler currently `todo!()`)
- Steps:
  - Locate the `CreateMessage::Cancel` handler.
  - Replace `todo!()` with state update that closes the dialog.
  - Ensure no other code path assumes the dialog remains present after Cancel.
- Test plan:
  - Manual: open create-partition dialog → click Cancel → verify no crash and dialog closes.
  - CI: run `cargo fmt --all --check`, `cargo clippy --workspace --all-features`, `cargo test --workspace --all-features`.
- Done when:
  - [ ] Cancel closes dialog.
  - [ ] No panic/todo reachable in this path.

## Task 2: Remove panic on invalid dialog state (GAP-002)
- Scope: Replace panic-based state checking in dialog update path with recoverable behavior.
- Likely areas:
  - `disks-ui/src/views/volumes.rs` (audit references `panic!("invalid state")`)
- Steps:
  - Identify the invalid-state branch.
  - Convert to safe handling:
    - ignore message, or
    - reset dialog state, or
    - close dialog + surface error (if UX primitives exist).
  - Add warning logging consistent with the codebase’s logging approach.
- Test plan:
  - Manual: open dialog → quickly open/close and interact (try to provoke late messages) → confirm no crash.
  - CI: `cargo fmt`, `cargo clippy`, `cargo test`.
- Done when:
  - [ ] No panics in dialog state transitions.
  - [ ] Invalid state paths are handled deterministically.

## Task 3: Prevent menu crash-on-click for unimplemented actions (GAP-003)
- Scope: Ensure menu items do not route into `todo!()`.
- Likely areas:
  - `disks-ui/src/views/menu.rs` (menu construction)
  - `disks-ui/src/app.rs` (update handler contains unimplemented actions)
- Steps:
  - Enumerate menu actions that are currently `todo!()`.
  - Choose one pattern and apply consistently:
    - Hide/remove actions from menu until implemented, or
    - Disable actions and show non-crashing “Not implemented” UX on click.
  - Ensure keyboard shortcuts (if any) also cannot trigger a panic.
- Test plan:
  - Manual: click each menu item, including previously crashing ones → verify no crash.
  - CI: `cargo fmt`, `cargo clippy`, `cargo test`.
- Done when:
  - [ ] No menu click can crash the app.
  - [ ] Unimplemented actions are either absent/disabled or show safe messaging.

## Task 4: CI/quality gate verification
- Scope: Make sure the changes satisfy repo quality gates.
- Steps:
  - Run `cargo fmt --all --check`.
  - Run `cargo clippy --workspace --all-features`.
  - Run `cargo test --workspace --all-features`.
- Done when:
  - [ ] All commands pass locally and in CI.

## Notes / Dependencies
- If the UI layer lacks a toast/dialog primitive for “Not implemented”, prefer disabling or hiding items over introducing new dependencies.
- Keep changes focused to prevent panics; avoid implementing disk-destructive actions as part of this work item.

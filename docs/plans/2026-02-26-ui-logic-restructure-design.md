# UI Logic Restructure Design

## Summary

Adopt an aggressive, non-compat migration from `storage-app/src/ui/**` to a new canonical structure centered on:

- `storage-app/src/message/{feature}.rs`
- `storage-app/src/state/{feature}.rs`
- `storage-app/src/update/{feature}/...`

and reduce/remove `ui/` usage entirely.

This follows the explicit direction: move fast, no legacy shims, no intermediate wrappers.

## Goals

1. Replace feature-scoped `ui/<feature>/{message,state,update}` with layer-scoped `message/state/update`.
2. Keep `views/{view_name}.rs` as canonical rendering layer.
3. Eliminate duplicated action-button/style patterns via shared controls.
4. Remove `ui/` as an architectural bucket (or leave only minimal transitory files for one commit max).

## Non-Goals

- No behavior redesign of storage/network/dialog business flows.
- No service-contract/API behavior changes.
- No UI feature additions.

## Chosen Approach

Use a **single aggressive migration wave** with direct file moves and immediate import rewrites.

Why:
- aligns with "move fast and break things",
- avoids long-lived mixed architecture,
- minimizes time spent on compatibility glue.

## Current Pain Points

1. Layering mismatch:
   - rendering is now `views/*`, but logic still in `ui/*`.
2. Import churn and coupling:
   - many files import `crate::ui::<feature>::message/state/...`.
3. Duplicated visual patterns remain:
   - custom action button assembly,
   - repeated button style closures,
   - repeated row/card style containers.

## Target Architecture

## 1) Layer-first module layout

```text
storage-app/src/
  message/
    mod.rs
    app.rs
    dialogs.rs
    network.rs
    volumes.rs
  state/
    mod.rs
    app.rs
    btrfs.rs
    dialogs.rs
    network.rs
    sidebar.rs
    volumes.rs
  update/
    mod.rs
    btrfs.rs
    drive.rs
    image.rs
    image/dialogs.rs
    image/ops.rs
    nav.rs
    network.rs
    smart.rs
    volumes/
      mod.rs
      btrfs.rs
      create.rs
      encryption.rs
      filesystem.rs
      mount.rs
      mount_options.rs
      partition.rs
      selection.rs
  views/
    ... (already canonical)
```

## 2) Feature helper placement (non-message/state/update)

To finish reducing `ui/` usage, move remaining helper modules to feature-root locations:

- `ui/network/icons.rs` -> `network/icons.rs`
- `ui/volumes/helpers.rs` -> `volumes/helpers.rs`
- `ui/volumes/disk_header.rs` -> `volumes/disk_header.rs`
- `ui/volumes/usage_pie.rs` -> `volumes/usage_pie.rs`
- `ui/app/subscriptions.rs` -> `subscriptions/app.rs`
- `ui/error.rs` -> `errors/ui.rs`

(Exact destination names are selected for low ambiguity and import readability.)

## 3) Views contract

`views/*` imports only from:
- `message::*`
- `state::*`
- `controls::*`
- feature helper modules (`network::*`, `volumes::*`, etc.)

and never from `ui::*`.

## Duplication Reduction Design (Action Buttons + Styling)

## Action Buttons

Centralize shared action controls in `controls/actions.rs`:
- icon button with tooltip and disabled behavior,
- grouped/trailing action row builders,
- shared style variants for icon-only action strips.

Apply these builders consistently in:
- `views/network.rs`
- `views/sidebar.rs`
- `views/app.rs` (tab/action strips where duplicated)
- `views/volumes.rs` (segment/child action strips where possible).

## Style Primitives

Centralize recurring style closures in `controls/layout.rs`:
- row container emphasis/de-emphasis,
- transparent/soft action button class helpers,
- card shell spacing defaults where repeated.

## Data Flow After Restructure

- `state/*` owns UI state structs and enums.
- `message/*` owns message enums.
- `update/*` owns transition/update logic and side effects.
- `views/*` maps `state + messages` to widgets.
- `controls/*` provides reusable composition primitives.

Dependency direction:

- `views` -> `state`, `message`, `controls`
- `update` -> `state`, `message`, clients/services
- `state` and `message` do not depend on `views`

## Risk Profile

Primary risk is import breakage due to broad moves.

Mitigation:
1. Move files in coherent buckets (message/state/update).
2. Immediately run `cargo clippy --workspace --all-targets` after each bucket.
3. Keep commits small and thematic.
4. Use grep-driven rewrites for `crate::ui::...` imports.

## Success Criteria

1. No `storage-app/src/ui/**` feature logic remains.
2. All state/message/update imports resolve from `state/message/update`.
3. Views do not import from `ui::*`.
4. Shared action/style duplication is reduced through controls.
5. Verification passes:
   - `cargo clippy --workspace --all-targets`
   - `cargo test --workspace --no-run`

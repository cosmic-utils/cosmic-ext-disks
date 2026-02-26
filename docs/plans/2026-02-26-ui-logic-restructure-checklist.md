# UI Logic Restructure Exhaustive Checklist

Branch: `069-polish`
Mode: aggressive, no compatibility wrappers

## A. Pre-flight

- [x] Confirm working tree clean (`git status --short`)
- [x] Confirm baseline green (`cargo clippy --workspace --all-targets`)
- [x] Confirm baseline tests compile (`cargo test --workspace --no-run`)

## B. Module Skeleton

- [x] Create `storage-app/src/message/mod.rs`
- [x] Create `storage-app/src/state/mod.rs`
- [x] Create `storage-app/src/update/mod.rs`
- [x] Create `storage-app/src/update/volumes/mod.rs`
- [x] Wire modules in `storage-app/src/main.rs`

## C. File Move Matrix (Exhaustive)

## C1. Messages

- [x] `storage-app/src/ui/app/message.rs` -> `storage-app/src/message/app.rs`
- [x] `storage-app/src/ui/dialogs/message.rs` -> `storage-app/src/message/dialogs.rs`
- [x] `storage-app/src/ui/network/message.rs` -> `storage-app/src/message/network.rs`
- [x] `storage-app/src/ui/volumes/message.rs` -> `storage-app/src/message/volumes.rs`

## C2. State

- [x] `storage-app/src/ui/app/state.rs` -> `storage-app/src/state/app.rs`
- [x] `storage-app/src/ui/btrfs/state.rs` -> `storage-app/src/state/btrfs.rs`
- [x] `storage-app/src/ui/dialogs/state.rs` -> `storage-app/src/state/dialogs.rs`
- [x] `storage-app/src/ui/network/state.rs` -> `storage-app/src/state/network.rs`
- [x] `storage-app/src/ui/sidebar/state.rs` -> `storage-app/src/state/sidebar.rs`
- [x] `storage-app/src/ui/volumes/state.rs` -> `storage-app/src/state/volumes.rs`

## C3. Updates (App)

- [x] `storage-app/src/ui/app/update/mod.rs` -> `storage-app/src/update/mod.rs`
- [x] `storage-app/src/ui/app/update/btrfs.rs` -> `storage-app/src/update/btrfs.rs`
- [x] `storage-app/src/ui/app/update/drive.rs` -> `storage-app/src/update/drive.rs`
- [x] `storage-app/src/ui/app/update/image.rs` -> `storage-app/src/update/image.rs`
- [x] `storage-app/src/ui/app/update/nav.rs` -> `storage-app/src/update/nav.rs`
- [x] `storage-app/src/ui/app/update/network.rs` -> `storage-app/src/update/network.rs`
- [x] `storage-app/src/ui/app/update/smart.rs` -> `storage-app/src/update/smart.rs`
- [x] `storage-app/src/ui/app/update/image/dialogs.rs` -> `storage-app/src/update/image/dialogs.rs`
- [x] `storage-app/src/ui/app/update/image/ops.rs` -> `storage-app/src/update/image/ops.rs`

## C4. Updates (Volumes)

- [x] `storage-app/src/ui/volumes/update.rs` -> `storage-app/src/update/volumes/mod.rs`
- [x] `storage-app/src/ui/volumes/update/btrfs.rs` -> `storage-app/src/update/volumes/btrfs.rs`
- [x] `storage-app/src/ui/volumes/update/create.rs` -> `storage-app/src/update/volumes/create.rs`
- [x] `storage-app/src/ui/volumes/update/encryption.rs` -> `storage-app/src/update/volumes/encryption.rs`
- [x] `storage-app/src/ui/volumes/update/filesystem.rs` -> `storage-app/src/update/volumes/filesystem.rs`
- [x] `storage-app/src/ui/volumes/update/mount.rs` -> `storage-app/src/update/volumes/mount.rs`
- [x] `storage-app/src/ui/volumes/update/mount_options.rs` -> `storage-app/src/update/volumes/mount_options.rs`
- [x] `storage-app/src/ui/volumes/update/partition.rs` -> `storage-app/src/update/volumes/partition.rs`
- [x] `storage-app/src/ui/volumes/update/selection.rs` -> `storage-app/src/update/volumes/selection.rs`

## C5. Remaining `ui` Helper Modules

- [x] `storage-app/src/ui/network/icons.rs` -> `storage-app/src/network/icons.rs`
- [x] `storage-app/src/ui/volumes/helpers.rs` -> `storage-app/src/volumes/helpers.rs`
- [x] `storage-app/src/ui/volumes/disk_header.rs` -> `storage-app/src/volumes/disk_header.rs`
- [x] `storage-app/src/ui/volumes/usage_pie.rs` -> `storage-app/src/volumes/usage_pie.rs`
- [x] `storage-app/src/ui/app/subscriptions.rs` -> `storage-app/src/subscriptions/app.rs`
- [x] `storage-app/src/ui/error.rs` -> `storage-app/src/errors/ui.rs`

## D. Import Rewrite Sweep

- [x] Rewrite all `crate::ui::app::message::*` -> `crate::message::app::*`
- [x] Rewrite all `crate::ui::dialogs::message::*` -> `crate::message::dialogs::*`
- [x] Rewrite all `crate::ui::network::message::*` -> `crate::message::network::*`
- [x] Rewrite all `crate::ui::volumes::message::*` -> `crate::message::volumes::*`
- [x] Rewrite all `crate::ui::<feature>::state::*` -> `crate::state::<feature>::*`
- [x] Rewrite all `crate::ui::app::update::*` -> `crate::update::*`
- [x] Rewrite all `crate::ui::volumes::update::*` -> `crate::update::volumes::*`
- [x] Rewrite all moved helper imports (`network`, `volumes`, `subscriptions`, `errors`)

## E. App Entry and Re-exports

- [x] Update `storage-app/src/app.rs` re-exports:
  - [x] `AppModel` from `crate::state::app::AppModel`
  - [x] `Message` from `crate::message::app::Message`
- [x] Update `storage-app/src/views/*.rs` imports away from `crate::ui::*`
- [x] Update module wiring where `ui/*/mod.rs` was the previous gateway

## F. Controls Dedup Follow-up

- [x] Inventory repeated action button builders in `views/{app,network,sidebar,volumes}.rs`
- [x] Expand `storage-app/src/controls/actions.rs` helpers if needed
- [x] Expand `storage-app/src/controls/layout.rs` style primitives if needed
- [x] Replace duplicated inline style/button code in views

## G. ui Directory Removal

- [x] Remove obsolete `storage-app/src/ui/*/mod.rs` files
- [x] Remove empty `storage-app/src/ui/**` directories
- [x] Verify no `crate::ui::` references remain (`grep`)

## H. Verification Gates

- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets`
- [x] `cargo test --workspace --no-run`
- [x] `git status --short` is clean

## I. Commit Strategy

- [x] No intermediate commits during migration
- [x] One final commit after all verification gates pass

# UI Logic Restructure Exhaustive Checklist

Branch: `069-polish`
Mode: aggressive, no compatibility wrappers

## A. Pre-flight

- [ ] Confirm working tree clean (`git status --short`)
- [ ] Confirm baseline green (`cargo clippy --workspace --all-targets`)
- [ ] Confirm baseline tests compile (`cargo test --workspace --no-run`)

## B. Module Skeleton

- [ ] Create `storage-app/src/messages/mod.rs`
- [ ] Create `storage-app/src/state/mod.rs`
- [ ] Create `storage-app/src/updates/mod.rs`
- [ ] Create `storage-app/src/updates/volumes/mod.rs`
- [ ] Wire modules in `storage-app/src/main.rs`

## C. File Move Matrix (Exhaustive)

## C1. Messages

- [ ] `storage-app/src/ui/app/message.rs` -> `storage-app/src/messages/app.rs`
- [ ] `storage-app/src/ui/dialogs/message.rs` -> `storage-app/src/messages/dialogs.rs`
- [ ] `storage-app/src/ui/network/message.rs` -> `storage-app/src/messages/network.rs`
- [ ] `storage-app/src/ui/volumes/message.rs` -> `storage-app/src/messages/volumes.rs`

## C2. State

- [ ] `storage-app/src/ui/app/state.rs` -> `storage-app/src/state/app.rs`
- [ ] `storage-app/src/ui/btrfs/state.rs` -> `storage-app/src/state/btrfs.rs`
- [ ] `storage-app/src/ui/dialogs/state.rs` -> `storage-app/src/state/dialogs.rs`
- [ ] `storage-app/src/ui/network/state.rs` -> `storage-app/src/state/network.rs`
- [ ] `storage-app/src/ui/sidebar/state.rs` -> `storage-app/src/state/sidebar.rs`
- [ ] `storage-app/src/ui/volumes/state.rs` -> `storage-app/src/state/volumes.rs`

## C3. Updates (App)

- [ ] `storage-app/src/ui/app/update/mod.rs` -> `storage-app/src/updates/mod.rs`
- [ ] `storage-app/src/ui/app/update/btrfs.rs` -> `storage-app/src/updates/btrfs.rs`
- [ ] `storage-app/src/ui/app/update/drive.rs` -> `storage-app/src/updates/drive.rs`
- [ ] `storage-app/src/ui/app/update/image.rs` -> `storage-app/src/updates/image.rs`
- [ ] `storage-app/src/ui/app/update/nav.rs` -> `storage-app/src/updates/nav.rs`
- [ ] `storage-app/src/ui/app/update/network.rs` -> `storage-app/src/updates/network.rs`
- [ ] `storage-app/src/ui/app/update/smart.rs` -> `storage-app/src/updates/smart.rs`
- [ ] `storage-app/src/ui/app/update/image/dialogs.rs` -> `storage-app/src/updates/image/dialogs.rs`
- [ ] `storage-app/src/ui/app/update/image/ops.rs` -> `storage-app/src/updates/image/ops.rs`

## C4. Updates (Volumes)

- [ ] `storage-app/src/ui/volumes/update.rs` -> `storage-app/src/updates/volumes/mod.rs`
- [ ] `storage-app/src/ui/volumes/update/btrfs.rs` -> `storage-app/src/updates/volumes/btrfs.rs`
- [ ] `storage-app/src/ui/volumes/update/create.rs` -> `storage-app/src/updates/volumes/create.rs`
- [ ] `storage-app/src/ui/volumes/update/encryption.rs` -> `storage-app/src/updates/volumes/encryption.rs`
- [ ] `storage-app/src/ui/volumes/update/filesystem.rs` -> `storage-app/src/updates/volumes/filesystem.rs`
- [ ] `storage-app/src/ui/volumes/update/mount.rs` -> `storage-app/src/updates/volumes/mount.rs`
- [ ] `storage-app/src/ui/volumes/update/mount_options.rs` -> `storage-app/src/updates/volumes/mount_options.rs`
- [ ] `storage-app/src/ui/volumes/update/partition.rs` -> `storage-app/src/updates/volumes/partition.rs`
- [ ] `storage-app/src/ui/volumes/update/selection.rs` -> `storage-app/src/updates/volumes/selection.rs`

## C5. Remaining `ui` Helper Modules

- [ ] `storage-app/src/ui/network/icons.rs` -> `storage-app/src/network/icons.rs`
- [ ] `storage-app/src/ui/volumes/helpers.rs` -> `storage-app/src/volumes/helpers.rs`
- [ ] `storage-app/src/ui/volumes/disk_header.rs` -> `storage-app/src/volumes/disk_header.rs`
- [ ] `storage-app/src/ui/volumes/usage_pie.rs` -> `storage-app/src/volumes/usage_pie.rs`
- [ ] `storage-app/src/ui/app/subscriptions.rs` -> `storage-app/src/subscriptions/app.rs`
- [ ] `storage-app/src/ui/error.rs` -> `storage-app/src/errors/ui.rs`

## D. Import Rewrite Sweep

- [ ] Rewrite all `crate::ui::app::message::*` -> `crate::messages::app::*`
- [ ] Rewrite all `crate::ui::dialogs::message::*` -> `crate::messages::dialogs::*`
- [ ] Rewrite all `crate::ui::network::message::*` -> `crate::messages::network::*`
- [ ] Rewrite all `crate::ui::volumes::message::*` -> `crate::messages::volumes::*`
- [ ] Rewrite all `crate::ui::<feature>::state::*` -> `crate::state::<feature>::*`
- [ ] Rewrite all `crate::ui::app::update::*` -> `crate::updates::*`
- [ ] Rewrite all `crate::ui::volumes::update::*` -> `crate::updates::volumes::*`
- [ ] Rewrite all moved helper imports (`network`, `volumes`, `subscriptions`, `errors`)

## E. App Entry and Re-exports

- [ ] Update `storage-app/src/app.rs` re-exports:
  - [ ] `AppModel` from `crate::state::app::AppModel`
  - [ ] `Message` from `crate::messages::app::Message`
- [ ] Update `storage-app/src/views/*.rs` imports away from `crate::ui::*`
- [ ] Update module wiring where `ui/*/mod.rs` was the previous gateway

## F. Controls Dedup Follow-up

- [ ] Inventory repeated action button builders in `views/{app,network,sidebar,volumes}.rs`
- [ ] Expand `storage-app/src/controls/actions.rs` helpers if needed
- [ ] Expand `storage-app/src/controls/layout.rs` style primitives if needed
- [ ] Replace duplicated inline style/button code in views

## G. ui Directory Removal

- [ ] Remove obsolete `storage-app/src/ui/*/mod.rs` files
- [ ] Remove empty `storage-app/src/ui/**` directories
- [ ] Verify no `crate::ui::` references remain (`grep`)

## H. Verification Gates

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets`
- [ ] `cargo test --workspace --no-run`
- [ ] `git status --short` is clean

## I. Commit Cadence (Recommended)

- [ ] Commit 1: scaffold layer modules
- [ ] Commit 2: message moves + import rewrites
- [ ] Commit 3: state moves + import rewrites
- [ ] Commit 4: app updates migration
- [ ] Commit 5: volumes updates migration
- [ ] Commit 6: helper module moves out of ui
- [ ] Commit 7: controls dedup pass
- [ ] Commit 8: ui tree deletion + final verification

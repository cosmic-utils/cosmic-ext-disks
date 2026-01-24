# Implementation Log — feature/luks-logical-volumes

## 2026-01-24
- Implemented nested volume model in `disks-dbus` (new `VolumeNode` tree) and plumbed it through `DriveModel`.
- Added LUKS unlock/lock operations using UDisks2 `Encrypted` interface; UI exposes Unlock/Lock actions on containers.
- Enumerated contained filesystems and rendered them as nested child rows with mount/unmount only on children.
- Added best-effort LVM support (PV -> LVs) using `pvs`/`lvs` enumeration; mapped LV device paths back to UDisks block objects for probing/mounting.
- Ran `cargo clippy --workspace --all-features -- -D warnings` (clean).
- Ran `cargo test --workspace --all-features` (passing).

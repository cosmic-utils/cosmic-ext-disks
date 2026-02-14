# Implementation Log — feature/luks-logical-volumes

## 2026-01-24
- Implemented nested volume model in `storage-dbus` (new `VolumeNode` tree) and plumbed it through `DriveModel`.
- Added LUKS unlock/lock operations using UDisks2 `Encrypted` interface; UI exposes Unlock/Lock actions on containers.
- Enumerated contained filesystems and rendered them as nested child rows with mount/unmount only on children.
- Added best-effort LVM support (PV -> LVs) using `pvs`/`lvs` enumeration; mapped LV device paths back to UDisks block objects for probing/mounting.
- Ran `cargo clippy --workspace --all-features -- -D warnings` (clean).
- Ran `cargo test --workspace --all-features` (passing).
- Final UI tweaks:
	- Increased volumes bar and nested child row height.
	- Split container/children UI into top/bottom halves.
	- Made child filesystem nodes selectable and wired the details panel to show their info.
	- Renamed cleartext filesystem child title to “Filesystem” when unlabeled.

- Follow-up: fixed a `clippy` `unused_mut` warning in the child-tile renderer and re-verified:
	- Ran `cargo fmt --all --check` (clean).
	- Ran `cargo clippy --workspace --all-features -- -D warnings` (clean).
	- Ran `cargo test --workspace --all-features` (passing).

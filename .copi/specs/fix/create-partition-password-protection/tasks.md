# fix/create-partition-password-protection — Tasks

Branch: `fix/create-partition-password-protection`

## Task 1: Confirm UDisks2 encryption options for CreatePartitionAndFormat
- Scope: Determine the exact `a{sv}` keys/values needed to request LUKS when creating + formatting.
- Likely areas:
  - `storage-dbus/src/disks/ops.rs` (`RealDiskBackend::create_partition_and_format`)
- Steps:
  - Inspect UDisks2 docs / `udisksctl` / GNOME Disks D-Bus calls.
  - Decide the canonical values (e.g., `encrypt.type=luks2`, `encrypt.passphrase=<...>`).
  - Record the chosen keys in code comments (no passphrase examples).
- Test plan:
  - Manual: create encrypted partition on a test device and verify it appears as LUKS.
- Done when:
  - [x] Exact keys/values are known and documented in this spec or an implementation note.

## Task 2: Wire password-protected create-partition into DBus create call
- Scope: Ensure `CreatePartitionInfo.password_protected/password` affect the UDisks2 create request.
- Likely files/areas:
  - `storage-dbus/src/disks/ops.rs`
  - (maybe) `storage-dbus/src/disks/create_partition_info.rs`
- Steps:
  - Extend `CreatePartitionAndFormatArgs` to include encryption intent (type + passphrase), or compute format options directly from `CreatePartitionInfo`.
  - In `RealDiskBackend::create_partition_and_format`, add encryption options to `format_options` when requested.
  - Ensure passphrases never hit logs and are not included in `Debug` output.
- Test plan:
  - Unit: extend existing `build_create_partition_and_format_args` tests to cover encryption case.
  - Unit: ensure `drive_create_partition` surfaces a helpful error when encryption tooling is missing (if detectable).
- Done when:
  - [x] Encrypted create path passes encryption options to UDisks.
  - [x] Unencrypted path remains unchanged.

## Task 3: Add minimal UI-side validation for passphrase inputs
- Scope: Prevent “encrypted create” with empty/mismatched passphrase.
- Likely files/areas:
  - `storage-ui/src/ui/volumes/update/create.rs`
  - `storage-ui/src/ui/dialogs/state.rs` (may need an error field)
  - `storage-ui/src/ui/dialogs/view/partition.rs` (render error + disable button)
- Steps:
  - If `password_protected` is enabled:
    - require non-empty password
    - require `password == confirmed_password`
  - Show an error message in the dialog and do not start the create task when invalid.
- Test plan:
  - Manual: try mismatch/empty; verify the dialog blocks the operation and shows a clear error.
- Done when:
  - [x] Mismatched/empty passphrases cannot proceed.

## Task 4: Manual validation checklist (smoke test)
- Scope: Human validation on a disposable/test disk or loopback device.
- Steps:
  - Create a small partition with **Password protected** enabled.
  - Refresh: confirm it appears as “LUKS” / `CryptoContainer`.
  - Unlock via UI: confirm cleartext child appears and is mountable.
  - Create a partition without password: confirm unchanged behavior.
- Done when:
  - [x] All acceptance criteria in plan.md are met.

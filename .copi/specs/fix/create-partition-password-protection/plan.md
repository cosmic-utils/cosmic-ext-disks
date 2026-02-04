# Fix: Create partition “Password protected” actually creates LUKS

Branch: `fix/create-partition-password-protection`

Source: Brief (2026-02-04) — “Enabling Password protection on create partition doesnt actually setup a luks partition”

## Context
The UI exposes a **Password protected** checkbox and passphrase fields in the create-partition dialog, but the backend partition creation path ignores these values.

Evidence (current behavior):
- UI collects the flag + passphrase via `CreatePartitionInfo`: `password_protected`, `password`, `confirmed_password`.
  - `disks-ui/src/ui/dialogs/view/partition.rs`
  - `disks-ui/src/ui/volumes/update/create.rs`
  - `disks-dbus/src/disks/create_partition_info.rs`
- DBus/ops layer always calls UDisks2 `CreatePartitionAndFormat` without any encryption-related options.
  - `disks-dbus/src/disks/ops.rs` (`RealDiskBackend::create_partition_and_format`, `build_create_partition_and_format_args`)

## Goals
- When **Password protected** is enabled, creating a partition results in an encrypted LUKS container (discoverable as `VolumeKind::CryptoContainer` after refresh).
- The passphrase entered in the dialog is actually used during creation.
- The non-encrypted create-partition flow remains unchanged.

## Non-Goals
- Implementing keyfile storage, auto-unlock at boot, or other advanced encryption policies.
- Redesigning the dialog UX beyond minimal validation.

## Proposed Approach
1. **Plumb encryption intent into the DBus create call**
   - Extend `CreatePartitionAndFormatArgs` to carry encryption parameters (e.g., `encrypt_type`, `encrypt_passphrase`) OR derive them from `CreatePartitionInfo` when building the args.
   - In `RealDiskBackend::create_partition_and_format`, add required UDisks2 format options when encryption is requested.

2. **Use UDisks2-supported encryption options (confirm exact keys)**
   - Confirmed via UDisks2 API docs for `org.freedesktop.UDisks2.Block.Format`:
     - `encrypt.passphrase`: passphrase (type `s` or `ay`)
     - `encrypt.type`: encryption technology, one of `luks1`/`luks2`
   - Confirm by:
     - Checking UDisks2 docs or `udisksctl` behavior, or
     - Comparing with GNOME Disks’ `CreatePartitionAndFormat` invocation (via dbus-monitor).

## Implementation Notes
- Uses `format_options` for `CreatePartitionAndFormat` to pass `encrypt.passphrase` and `encrypt.type=luks2`.
- Passphrases are carried through the code in a `RedactedString` wrapper to avoid accidental Debug/log leaks.

3. **Validation and security**
   - Validate before making the DBus call:
     - If `password_protected`: require non-empty passphrase and `password == confirmed_password`.
     - If invalid: keep dialog open and show an actionable error.
   - Ensure passphrases are never logged (including via `Debug` prints). Avoid including the passphrase in error strings.

4. **Post-create expectations**
   - After creation, `DriveModel::get_drives()` refresh should show:
     - A new crypto container volume (locked by default), and
     - A cleartext child device once unlocked (filesystem inside should reflect selected filesystem type).

## User/System Flows
- Create partition (unencrypted)
  - User leaves **Password protected** off → partition is created + formatted as before.

- Create partition (encrypted)
  - User enables **Password protected**, enters passphrase + confirm, presses **Continue** → backend creates an encrypted LUKS container and formats the inner filesystem → UI refresh shows locked “LUKS” volume → user can unlock with the same passphrase.

## Risks & Mitigations
- **UDisks option key/value mismatch across distros**
  - Mitigation: probe/document the exact option names; prefer compatibility with common UDisks2 versions.
- **Missing system tooling (cryptsetup)**
  - Mitigation: surface a clear error/hint, similar to existing NTFS/exFAT hints.
- **Passphrase handling**
  - Mitigation: do not log passphrases; minimize cloning; avoid storing passphrase beyond the create call.

## Acceptance Criteria
- [x] When **Password protected** is enabled and the dialog completes, the created volume is an encrypted container (visible as `VolumeKind::CryptoContainer` after refresh).
- [x] Unlocking the new encrypted volume with the chosen passphrase works.
- [x] When **Password protected** is disabled, create-partition behavior is unchanged.
- [x] Passphrases are not logged and do not appear in error messages.
- [x] Add/adjust tests in the DBus layer to assert encryption options are wired when requested.

Manual validation: confirmed working on 2026-02-04.

# Implementation Log — fix/luks-delete-preflight

## 2026-01-25

## 2026-01-25
- Improved partition deletion errors to preserve D-Bus error name/message by using a raw `zbus::Proxy` call to `org.freedesktop.UDisks2.Partition.Delete`.
- Added best-effort context to delete failures (device path, partition number, table type/path, object path) to make errors like “Invalid argument” diagnosable.
- Added a short retry loop for transient `org.freedesktop.UDisks2.Error.DeviceBusy` races right after crypto teardown.

- Parity fix vs GNOME Disks: call `Partition.Delete` with empty options first (GNOME uses `a{sv}` empty) and keep `tear-down=true` only as a fallback.

- Added best-effort diagnostics to report any `/proc/self/fd` entries pointing at the target device during deletion failures (helps confirm whether our process is holding open block device FDs).

- After confirming the failing system was just in a stale kernel/userspace disk state (fixed via `wipefs -n` + `partprobe`/`partx`/`udevadm settle` and recreating the partition), simplified `Partition.Delete` again: removed retries and verbose debug context, keeping only preserved D-Bus error name/message plus minimal context (device + object path).

- `cargo test --workspace --all-features`
- `disks-ui/i18n/sv/cosmic_ext_disks.ftl`
- `.copi/specs/fix/luks-delete-preflight/tasks.md`

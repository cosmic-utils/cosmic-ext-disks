# GAP-007 — Spec: Mount state detection via UDisks2

Branch: `fix/gap-007-mount-state-detection`

Source:
- GAP: `GAP-007`
- Audit: `.copi/audits/2026-01-24T18-03-04Z.md`

## Context
The UI currently infers “mounted vs not mounted” by checking whether filesystem `usage` exists. That `usage` is sourced from parsing `df` output, which is not a reliable indicator of UDisks2’s mount state and can diverge (timing, permissions, transient mountpoints, unusual filesystems, etc.).

Research note: UDisks2 *does* provide mount state via `org.freedesktop.UDisks2.Filesystem.MountPoints` (empty ⇒ not mounted) and it provides sizes via `Block.Size` / `Filesystem.Size`, but it does **not** provide “bytes used / bytes available” for a mounted filesystem. That usage data must be computed locally (e.g., `statvfs`) if we want to remove `df` entirely.

This GAP will drive mount/unmount UI state from UDisks2-reported mount points, and it will replace `df` usage collection with `statvfs` (no external `df` invocation) for mounted filesystems.

## Goals
- Determine mount state from UDisks2 (not from `df`).
- Remove `df` parsing entirely; compute usage via `statvfs` for mounted filesystems.
- Keep usage data optional enrichment (space used/free), without affecting mount/unmount actions.
- Ensure mount/unmount actions and labels reflect the actual state reported by UDisks2.

## Non-Goals
- Changing the mount/unmount implementation itself (unless required for correctness).
- Implementing a full “device change signals” model (covered by GAP-008).
- Providing usage stats for unmounted filesystems (expected to remain unknown/absent).

## Proposed Approach
- DBus layer:
  - Extend the disk/partition model to expose a canonical mount-state view derived from UDisks2 properties (e.g., filesystem mountpoints).
  - Replace usage retrieval:
    - Read device path from UDisks2 `Block.PreferredDevice`/`Block.Device` (instead of inferring it from `df`).
    - For mounted filesystems, compute usage by running `statvfs` on a mountpoint path.
    - Keep the usage field optional and best-effort; failures must not flip mount state.
- UI layer:
  - Replace “usage exists” mount indicator with the new mount-state field.
  - Continue showing usage numbers when available.

Likely touched areas:
- `storage-dbus/src/disks/partition.rs` (or related model types)
- `storage-dbus/src/usage.rs` (replace `df` with `statvfs` helpers, or delete/repurpose)
- `storage-ui/src/views/volumes.rs` (mount/unmount button logic)

## User / System Flows
- **Volumes list rendering**
  - System fetches partitions via UDisks2.
  - For each filesystem partition, system reads mountpoints (or equivalent) from UDisks2.
  - UI renders “Mount” if no mountpoints; renders “Unmount” if mountpoints present.
  - UI optionally renders usage if `df` data is available.

- **Mount action**
  - User clicks “Mount”.
  - System invokes UDisks2 mount; on completion, mountpoints reflect mounted state.
  - UI updates based on mountpoints regardless of usage availability.

- **Unmount action**
  - User clicks “Unmount”.
  - System invokes UDisks2 unmount; on completion, mountpoints reflect unmounted state.
  - UI updates based on mountpoints regardless of usage availability.

## Risks & Mitigations
- **Different UDisks2 object types** (filesystem vs crypto vs loop): clarify which partitions should show mount actions.
  - Mitigation: define mount-state only for filesystem-capable partitions; gate UI accordingly.
- **Transient states / races** (action in progress vs refreshed properties).
  - Mitigation: keep existing in-flight UI state handling; refresh from UDisks2 after operations.
- **Mountpoints can be multiple** (bind mounts).
  - Mitigation: treat “non-empty mountpoints” as mounted; optionally show the first mountpoint for display.

## Acceptance Criteria
- [x] Mount/unmount button state is derived from UDisks2-reported mount state (e.g., mountpoints), not from `df`.
- [x] No runtime dependency on parsing `df` output (no `df` command execution).
- [x] Usage collection failures do not cause incorrect mount/unmount UI state.
- [x] UI behavior matches UDisks2 state after mount/unmount operations.

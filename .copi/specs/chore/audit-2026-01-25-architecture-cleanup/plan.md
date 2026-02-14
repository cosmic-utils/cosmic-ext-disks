# Implementation Spec — Audit 2026-01-25 Architecture Cleanup

Branch: `chore/audit-2026-01-25-architecture-cleanup`

Source audit: `.copi/audits/2026-01-25T23-24-44Z.md`

Scope focus: **Abstractions, naming, hierarchy, separation of concerns, file length/complexity**.

## Context

The repo is currently functional but concentrated into a few very large “god files” in both the UI and DBus crates:

- UI: `storage-ui/src/app.rs` (~1780 LOC), `storage-ui/src/views/volumes.rs` (~2750 LOC), `storage-ui/src/views/dialogs.rs` (~980 LOC)
- DBus: `storage-dbus/src/disks/drive.rs` (~1150 LOC), `storage-dbus/src/disks/partition.rs` (~988 LOC), `storage-dbus/src/disks/volume.rs` (~672 LOC), `storage-dbus/src/partition_type.rs` (~2070 LOC)

This spec proposes a refactor that **does not add new features** and is primarily about reducing coupling, clarifying ownership boundaries, and making changes safer.

## Goals

- Reduce the largest files into smaller, cohesive modules (target: < ~400 LOC per module, with rare justified exceptions).
- Make UI flows easier to extend by centralizing message routing and reducing “dialog-state switching” boilerplate.
- Align naming and vocabulary across layers (partition vs volume vs node).
- Deduplicate shared low-level helpers (bytestring decoding, mount-point decoding, byte formatting).
- Prepare `partition_type` for localization and maintainability.

## Non-goals

- Rewriting UI/UX or changing workflows.
- Changing the UDisks2 behavior or permissions model.
- Introducing new dependencies unless a clear “data only” parsing format for partition types is chosen.
- Broad “cleanup PR” (this is intentionally broken into small tasks).

## Overview (Object Tree — Old → New)

This section is the canonical “map” of what moves where. Each node lists:

- **Old name / location**
- **New name / location**
- **Notes** (ownership boundary + why)

### UI crate (`storage-ui`)

| Node | Old | New | Notes |
|---|---|---|---|
| App root | `AppModel` in `storage-ui/src/app.rs` | `ui/app/mod.rs` + submodules | Keep `ui/app/mod.rs` as glue; move state/messages/update/view out. |
| App messages | `enum Message` in `storage-ui/src/app.rs` | `storage-ui/src/ui/app/message.rs` | Conversions (`From<...>`) live next to message types. |
| App state | `AppModel` fields + dialog fields in `storage-ui/src/app.rs` | `storage-ui/src/ui/app/state.rs` | State-only; no async IO. |
| App update | `fn update(...)` in `storage-ui/src/app.rs` | `storage-ui/src/ui/app/update.rs` | `match Message` routing; delegates to feature modules. |
| App view | `fn view(...)` in `storage-ui/src/app.rs` | `storage-ui/src/ui/app/view.rs` | Must be “pure view”: no unwrap-based invariants; no disk IO. |
| App subscriptions | device + config subscriptions in `storage-ui/src/app.rs` | `storage-ui/src/ui/app/subscriptions.rs` | Encapsulate `DiskManager` wiring and config watch. |
| Dialog state | `ShowDialog` + dialog state structs in `storage-ui/src/app.rs` | `storage-ui/src/ui/dialogs/state.rs` | Dialog ownership becomes the dialogs module, not AppModel. |
| Dialog messages | multiple `*DialogMessage` enums in `storage-ui/src/app.rs` | `storage-ui/src/ui/dialogs/message.rs` | Moves dialog message types under dialogs; `Message` wraps them. |
| Dialog rendering | functions in `storage-ui/src/views/dialogs.rs` | `storage-ui/src/ui/dialogs/view/*.rs` | Split per dialog or per domain (image, format, encryption, etc). |
| Volumes view+logic | huge file `storage-ui/src/views/volumes.rs` | `storage-ui/src/ui/volumes/{mod.rs,state.rs,message.rs,update.rs,view.rs,actions.rs}` | Separate view/state/update/actions; make message handling composable. |
| Volumes control | `VolumesControl` in `storage-ui/src/views/volumes.rs` | `storage-ui/src/ui/volumes/state.rs` | Pure state + helpers; no direct `eprintln!`. |
| Volumes messages | `VolumesControlMessage` + submessages in `storage-ui/src/views/volumes.rs` | `storage-ui/src/ui/volumes/message.rs` | Fix naming typos during migration (see “Naming”). |
| Segmentation | `compute_disk_segments` in `storage-ui/src/utils/segments.rs` | unchanged (for now) | This is already reasonably cohesive; just update imports if module paths change. |
| Byte formatting | `storage-ui/src/utils/format.rs` (dup) | remove; import from `storage-dbus` | DBus crate remains canonical; no new shared/common crate. |

### DBus crate (`storage-dbus`)

| Node | Old | New | Notes |
|---|---|---|---|
| Disk module root | `storage-dbus/src/disks/mod.rs` re-exports many types | `storage-dbus/src/disks/mod.rs` with clearer submodule exports | Keep a stable public API, but stop re-exporting ambiguous aliases. |
| Drive model | `DriveModel` in `storage-dbus/src/disks/drive.rs` | `storage-dbus/src/disks/drive/{mod.rs,model.rs,discovery.rs,actions.rs,smart.rs,volume_tree.rs}` | Split by responsibility: discovery, actions, SMART, tree building. |
| Flat volume model (multi-role) | `VolumeModel` in `storage-dbus/src/disks/partition.rs` and alias `pub type PartitionModel = VolumeModel` | Keep **`VolumeModel`** (canonical name) but move to `storage-dbus/src/disks/volume_model.rs` (or `volume_model/mod.rs`); **remove the `PartitionModel` alias** | `VolumeModel` can represent container/partition/filesystem; the alias is what introduces ambiguity. Rename the file/module to match the type. |
| Volume tree node | `VolumeNode` in `storage-dbus/src/disks/volume.rs` | `storage-dbus/src/disks/volume_tree/node.rs` (name optional) | Separate “tree presentation + nested operations” from flat partition type. |
| Ops backend | `storage-dbus/src/disks/ops.rs` | `storage-dbus/src/disks/backend/{mod.rs,real.rs,trait.rs}` | Keep `DiskBackend` trait but move + name it as a backend boundary. |
| Bytestring helpers | duplicated in partition + volume modules | `storage-dbus/src/dbus/bytestring.rs` | Single helper for decode/encode/mountpoint decode; used everywhere. |
| Partition types catalog | `storage-dbus/src/partition_type.rs` giant array | `storage-dbus/src/partition_types/{mod.rs,gpt.rs,dos.rs}` (Rust modules) | Split by table type as an immediate size/maintainability win; keep a single API surface that merges both.
| Byte formatting | `storage-dbus/src/format.rs` + UI dup | keep DBus as canonical | No new shared/common crate; UI imports `disks_dbus::bytes_to_pretty` etc. |

### Naming + typos to fix as part of migration

| Old | New | Where |
|---|---|---|
| `PasswordProectedUpdate` | `PasswordProtectedUpdate` | UI volumes messages |
| `selected_partitition_type` | `selected_partition_type_index` (or `selected_partition_type`) | CreatePartitionInfo + call sites |
| `PartitionModel` alias | removed | DBus exports (use `VolumeModel` everywhere) |

## Proposed Approach

### Phase 1 — Carve out module boundaries without changing behavior

- UI: Introduce a `ui/` module tree and move code with minimal edits (mechanical refactor).
- DBus: Extract shared helpers and split “drive” into submodules while preserving public exports.

### Phase 2 — Fix naming and vocabulary

- Rename misspellings and ambiguous types; use `pub use` re-exports temporarily to avoid large diffs, then remove.

### Phase 3 — Reduce coupling and remove implicit invariants

- Replace `unwrap()` in UI view paths with safe rendering and explicit “no selection/loading” states.
- Reduce “dialog-state routing” by giving each dialog a dedicated update handler.

### Phase 4 — Partition type catalog maintainability

- First: split the large table into smaller Rust modules.
- Optional: move to a data file + parser/codegen if localization needs accelerate.

## User/System Flows

- **Open app → list drives → select drive → view segments**
  - Behavior unchanged; code moves from `app.rs` + `views/volumes.rs` into `ui/app/*` + `ui/volumes/*`.

- **Partition actions (mount/unmount/delete/create/format)**
  - Behavior unchanged; message routing becomes structured and dialog ownership becomes clear.

- **Device updates**
  - Behavior unchanged; subscription wiring moves into `ui/app/subscriptions.rs`.

## Risks & Mitigations

- **Risk: refactor breaks message wiring**
  - Mitigation: perform a series of small moves; keep compilation green per task.

- **Risk: public API break in `storage-dbus`**
  - Mitigation: staged rename with temporary `pub use` re-exports and clear removal task.

- **Risk: large diff makes PR review hard**
  - Mitigation: tasks are intentionally small and sequential.

## Acceptance Criteria (covers all audit gaps)

- [x] GAP-001 (UI god file): `storage-ui/src/app.rs` is reduced via `ui/app/*` split; no single module remains > ~400 LOC without justification.
- [x] GAP-002 (volumes god file): `views/volumes.rs` split into state/message/update/view/actions modules.
- [x] GAP-003 (dialogs coupling): dialog state + messages moved under `ui/dialogs/*`; dialogs do not depend on volumes message enums.
- [x] GAP-004 (DriveModel mixing): drive code split by domain (discovery/actions/smart/tree).
- [x] GAP-005 (partition types giant file): `partition_type.rs` split into `partition_types/` modules (GPT/DOS/APM) with stable API + tests.
- [x] GAP-006 (duplicated bytestring helpers): a shared helper module exists and is used by both flat + tree models.
- [x] GAP-007 (byte formatting duplication): single canonical formatting implementation is used by UI and DBus.
- [x] GAP-008 (typos): `rg -g'*.rs' "partitition|proected"` yields no matches.
- [x] GAP-009 (Volume/Partition naming confusion): remove `pub type PartitionModel = VolumeModel`; keep `VolumeModel` as the canonical multi-role model; rename files/modules accordingly.
- [x] GAP-010 (unwrap crashes): no unwraps in UI `view()` for nav/segment selection; safe fallbacks exist.
- [x] GAP-011 (logging layering): operational errors routed via `tracing` (and existing UI Info dialogs for actionable failures) rather than scattered `println!/eprintln!`.

## Addendum (2026-01-26) — Remaining UI module breakdown

The original scope is implemented, but two UI areas remain significantly oversized and are worth finishing to fully realize the “small cohesive modules” intent.

### Context

- `storage-ui/src/ui/volumes/update.rs` remains a large, monolithic `match` handler (~1655 LOC).
- Dialog rendering remains in a large legacy module `storage-ui/src/views/dialogs.rs` (~975 LOC) while dialog state/messages live under `storage-ui/src/ui/dialogs/`.

### Goals

- Split dialog rendering into `storage-ui/src/ui/dialogs/view/*` modules (grouped by domain) and reduce or retire `storage-ui/src/views/dialogs.rs`.
- Split `VolumesControl::update` handling into focused submodules under `storage-ui/src/ui/volumes/update/` (selection, mount/unmount, encryption, partition ops, dialogs, etc.).

### Non-goals

- No UX changes; only code organization and wiring.
- No behavior changes to disk operations; keep all async tasks and error surfaces equivalent.

### Updated acceptance criteria (addendum)

- [x] `storage-ui/src/ui/volumes/update.rs` becomes a thin dispatcher (or is reduced to < ~400 LOC) with domain-focused submodules.
- [x] Dialog rendering code lives under `storage-ui/src/ui/dialogs/view/` (or equivalent) and `storage-ui/src/views/dialogs.rs` is either removed or reduced to a compatibility shim.
- [x] App view continues to treat views as “pure view” (no IO, no panicking invariants) while importing dialogs from the `ui` hierarchy.

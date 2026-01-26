# Implementation Spec — Audit 2026-01-25 Architecture Cleanup

Branch: `chore/audit-2026-01-25-architecture-cleanup`

Source audit: `.copi/audits/2026-01-25T23-24-44Z.md`

Scope focus: **Abstractions, naming, hierarchy, separation of concerns, file length/complexity**.

## Context

The repo is currently functional but concentrated into a few very large “god files” in both the UI and DBus crates:

- UI: `disks-ui/src/app.rs` (~1780 LOC), `disks-ui/src/views/volumes.rs` (~2750 LOC), `disks-ui/src/views/dialogs.rs` (~980 LOC)
- DBus: `disks-dbus/src/disks/drive.rs` (~1150 LOC), `disks-dbus/src/disks/partition.rs` (~988 LOC), `disks-dbus/src/disks/volume.rs` (~672 LOC), `disks-dbus/src/partition_type.rs` (~2070 LOC)

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

### UI crate (`disks-ui`)

| Node | Old | New | Notes |
|---|---|---|---|
| App root | `AppModel` in `disks-ui/src/app.rs` | `ui/app/mod.rs` + submodules | Keep `ui/app/mod.rs` as glue; move state/messages/update/view out. |
| App messages | `enum Message` in `disks-ui/src/app.rs` | `disks-ui/src/ui/app/message.rs` | Conversions (`From<...>`) live next to message types. |
| App state | `AppModel` fields + dialog fields in `disks-ui/src/app.rs` | `disks-ui/src/ui/app/state.rs` | State-only; no async IO. |
| App update | `fn update(...)` in `disks-ui/src/app.rs` | `disks-ui/src/ui/app/update.rs` | `match Message` routing; delegates to feature modules. |
| App view | `fn view(...)` in `disks-ui/src/app.rs` | `disks-ui/src/ui/app/view.rs` | Must be “pure view”: no unwrap-based invariants; no disk IO. |
| App subscriptions | device + config subscriptions in `disks-ui/src/app.rs` | `disks-ui/src/ui/app/subscriptions.rs` | Encapsulate `DiskManager` wiring and config watch. |
| Dialog state | `ShowDialog` + dialog state structs in `disks-ui/src/app.rs` | `disks-ui/src/ui/dialogs/state.rs` | Dialog ownership becomes the dialogs module, not AppModel. |
| Dialog messages | multiple `*DialogMessage` enums in `disks-ui/src/app.rs` | `disks-ui/src/ui/dialogs/message.rs` | Moves dialog message types under dialogs; `Message` wraps them. |
| Dialog rendering | functions in `disks-ui/src/views/dialogs.rs` | `disks-ui/src/ui/dialogs/view/*.rs` | Split per dialog or per domain (image, format, encryption, etc). |
| Volumes view+logic | huge file `disks-ui/src/views/volumes.rs` | `disks-ui/src/ui/volumes/{mod.rs,state.rs,message.rs,update.rs,view.rs,actions.rs}` | Separate view/state/update/actions; make message handling composable. |
| Volumes control | `VolumesControl` in `disks-ui/src/views/volumes.rs` | `disks-ui/src/ui/volumes/state.rs` | Pure state + helpers; no direct `eprintln!`. |
| Volumes messages | `VolumesControlMessage` + submessages in `disks-ui/src/views/volumes.rs` | `disks-ui/src/ui/volumes/message.rs` | Fix naming typos during migration (see “Naming”). |
| Segmentation | `compute_disk_segments` in `disks-ui/src/utils/segments.rs` | unchanged (for now) | This is already reasonably cohesive; just update imports if module paths change. |
| Byte formatting | `disks-ui/src/utils/format.rs` (dup) | remove; import from `disks-dbus` | DBus crate remains canonical; no new shared/common crate. |

### DBus crate (`disks-dbus`)

| Node | Old | New | Notes |
|---|---|---|---|
| Disk module root | `disks-dbus/src/disks/mod.rs` re-exports many types | `disks-dbus/src/disks/mod.rs` with clearer submodule exports | Keep a stable public API, but stop re-exporting ambiguous aliases. |
| Drive model | `DriveModel` in `disks-dbus/src/disks/drive.rs` | `disks-dbus/src/disks/drive/{mod.rs,model.rs,discovery.rs,actions.rs,smart.rs,volume_tree.rs}` | Split by responsibility: discovery, actions, SMART, tree building. |
| Flat volume model (multi-role) | `VolumeModel` in `disks-dbus/src/disks/partition.rs` and alias `pub type PartitionModel = VolumeModel` | Keep **`VolumeModel`** (canonical name) but move to `disks-dbus/src/disks/volume_model.rs` (or `volume_model/mod.rs`); **remove the `PartitionModel` alias** | `VolumeModel` can represent container/partition/filesystem; the alias is what introduces ambiguity. Rename the file/module to match the type. |
| Volume tree node | `VolumeNode` in `disks-dbus/src/disks/volume.rs` | `disks-dbus/src/disks/volume_tree/node.rs` (name optional) | Separate “tree presentation + nested operations” from flat partition type. |
| Ops backend | `disks-dbus/src/disks/ops.rs` | `disks-dbus/src/disks/backend/{mod.rs,real.rs,trait.rs}` | Keep `DiskBackend` trait but move + name it as a backend boundary. |
| Bytestring helpers | duplicated in partition + volume modules | `disks-dbus/src/dbus/bytestring.rs` | Single helper for decode/encode/mountpoint decode; used everywhere. |
| Partition types catalog | `disks-dbus/src/partition_type.rs` giant array | `disks-dbus/src/partition_types/{mod.rs,gpt.rs,dos.rs}` (Rust modules) | Split by table type as an immediate size/maintainability win; keep a single API surface that merges both.
| Byte formatting | `disks-dbus/src/format.rs` + UI dup | keep DBus as canonical | No new shared/common crate; UI imports `disks_dbus::bytes_to_pretty` etc. |

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

- **Risk: public API break in `disks-dbus`**
  - Mitigation: staged rename with temporary `pub use` re-exports and clear removal task.

- **Risk: large diff makes PR review hard**
  - Mitigation: tasks are intentionally small and sequential.

## Acceptance Criteria (covers all audit gaps)

- [x] GAP-001 (UI god file): `disks-ui/src/app.rs` is reduced via `ui/app/*` split; no single module remains > ~400 LOC without justification.
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

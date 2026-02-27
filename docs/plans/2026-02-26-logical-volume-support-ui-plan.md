# Logical Volume Support UI Plan

**Date:** 2026-02-26  
**Status:** Approved for implementation planning  
**Companion:** `docs/plans/2026-02-26-logical-volume-support-design.md`

---

## 1. Current Source Baseline (What We Reuse)

### 1.1 Sidebar and Selection Infrastructure

- `storage-app/src/views/sidebar.rs` already has a `Logical` section enum but does not currently classify drives into it.
- Existing sidebar tree patterns to reuse:
  - section headers
  - expandable rows
  - selection state via `SidebarNodeKey`
- Existing network and image sections prove mixed section types can coexist.

### 1.2 Detail Surfaces

- `storage-app/src/views/disk.rs` (`disk_header`) is drive-specific and should not be reused for logical roots.
- `storage-app/src/views/volumes.rs` segment control is partition-layout-specific and should not be reused for logical roots.
- `storage-app/src/views/app.rs` detail-card/row patterns are reusable as the shell for logical entity detail pages.

### 1.3 State and Data Flow Patterns

- Existing app patterns to mirror:
  - state structs under `storage-app/src/state/*`
  - UI models in `storage-app/src/models/*`
  - message dispatch in `storage-app/src/message/*`
  - update handlers under `storage-app/src/update/*`
- Existing service calls for disks/volumes in `storage-app/src/client/disks.rs` establish JSON-over-DBus model for new logical client APIs.

---

## 2. Recommended V1 Feature Set

### 2.1 LVM

**Entity views:**
- Volume Group (root)
- Logical Volume (child)
- Physical Volume (member list)

**Read features:**
- VG size/free/used, PV count, LV count
- LV size/path/active state
- PV device assignment and capacity

**Management actions (v1):**
- VG: create/delete, add/remove PV
- LV: create/delete/resize, activate/deactivate
- Deep-link to physical members in disk section

### 2.2 MD RAID

**Entity views:**
- Array (root)
- Active members and spares

**Read features:**
- level, size, running/degraded, device count
- sync action/progress/rate/remaining time

**Management actions (v1):**
- create/delete array
- start/stop
- add/remove member
- request `check` / `repair` / `idle`

### 2.3 Multi-device BTRFS

**Entity views:**
- Filesystem root by UUID/label
- member devices list
- existing subvolume/snapshot management panel

**Read features:**
- label/uuid/num devices/used
- device membership and aggregate size

**Management actions (v1):**
- add/remove device
- resize
- set label
- set/get default subvolume
- reuse existing subvolume/snapshot actions

### 2.4 Nested LUKS Interactions

- LUKS remains a container layer, not a logical aggregate root.
- For logical entities containing encrypted members, expose lock/unlock/health context in member detail and route to existing LUKS flows.

---

## 3. Information Architecture and Layout

## 3.1 Sidebar IA

`Logical` section grouping:
- LVM
  - `VG <name>`
    - `LV <name>`
- RAID
  - `mdX (<level>)`
- BTRFS
  - `<label-or-uuid>`

Member devices are displayed inside detail pages, not as primary sidebar roots.

### 3.2 Logical Detail Page Layout

Use a shared page shell with four vertical zones:

1. **Title + Status Row**
   - icon, display name, type badge, health badge
2. **Summary Metrics Row**
   - size/used/free/count/progress cards
3. **Action Toolbar**
   - entity-specific actions, with disabled reasons
4. **Entity Body Tabs**
   - `Overview` (default)
   - `Members`
   - `Operations` (history/status)
   - `BTRFS` tab only when entity kind is BTRFS

### 3.3 Components and Structure Recommendation

**Reuse:**
- card containers and text hierarchy used in `views/app.rs`
- button/tooltip patterns from `views/disk.rs` and `views/app.rs`
- sidebar row and expander patterns from `views/sidebar.rs`

**Add:**
- `storage-app/src/views/logical.rs` as logical detail renderer
- logical-specific mini-components under `storage-app/src/controls/logical/*`:
  - `LogicalTopologyControl` (VolumesControl-like navigation for logical entities)
  - summary metric card
  - member table row
  - operation state pill

**Do not reuse:**
- `disk_header`
- partition segment control (`VolumesControl::view`)

### 3.4 LogicalTopologyControl Decision

Create a dedicated `LogicalTopologyControl` that mirrors the interaction style of `VolumesControl` (quick visual navigation and selection), but uses logical topology semantics:
- LVM: VG -> LV (+ PV/member context)
- MD RAID: array -> members/spares
- BTRFS: filesystem -> devices -> subvolumes

Rationale for not reusing `VolumesControl` internals:
- existing control is partition-segment based (offset/free/reserved/adjacent resize assumptions)
- action wiring is partition-centric (`create/resize/delete partition`)
- forcing logical entities into segment models would create misleading UI behavior and brittle code

Implementation note: reuse visual primitives/styles from `VolumesControl` where practical, while keeping state/messages/modeling independent.

---

## 4. Capability Matrix (Actions + Disabled Reasons)

| Entity | Action | Enabled when | Disabled reason text |
| --- | --- | --- | --- |
| LVM VG | Delete VG | No dependent active LVs and policy allows | `Volume group has active logical volumes` |
| LVM LV | Resize LV | LV supports resize and target is valid | `Logical volume cannot be resized in current state` |
| MD RAID | Repair | Array running and redundancy supports sync actions | `Array is not running or does not support repair` |
| MD RAID | Remove member | Member removable by current array policy | `Member removal would violate array integrity` |
| BTRFS FS | Remove device | Removal safe and space constraints met | `Insufficient redundancy/free space for device removal` |
| Any | Destructive op | Policy/auth passes and preflight passes | Policy/preflight error message from service |

---

## 5. UX Acceptance Criteria

1. `Logical` section is visibly populated when logical entities exist.
2. Selecting a logical root never renders disk-specific controls.
3. Action controls include clear disabled reasons without opening extra dialogs.
4. Member devices can navigate to physical disk context without duplicating ownership in sidebar hierarchy.
5. Existing physical disk/partition workflows remain unchanged.

---

## 6. Implementation Order for UI Workstream

1. Add logical state/client/message scaffolding.
2. Populate sidebar `Logical` with read-only entities.
3. Add shared logical detail shell and overview tab.
4. Add per-entity action toolbars and capability wiring.
5. Add members and operation-status views.
6. Integrate BTRFS management tab for multi-device logical entities.

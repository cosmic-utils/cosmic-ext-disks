# Logical Volume Support Design (LVM/RAID/Logical)

**Date:** 2026-02-26  
**Status:** Approved  
**Scope:** Add first-class logical storage topology and management across app/service/backends, including cross-device spanning entities.

---

## 1. Goals

1. Add a real `Logical` section in sidebar for non-physical aggregates.
2. Support full-management v1 for logical entities currently in practical scope.
3. Model cross-device spanning ownership explicitly (instead of inferring from physical trees).
4. Keep crate responsibilities strict:
   - UDisks-backed discovery only in `storage-udisks`
   - non-UDisks discovery/probing in `storage-sys`
   - orchestration/authz/API in `storage-service`
   - UI state and rendering in `storage-app`

## 2. Non-Goals (v1)

1. No Stratis management.
2. No ZFS pool/dataset management.
3. No dm-cache / dm-writecache management.
4. No VDO management.
5. No bcache/bcachefs management.

These remain future work and are documented in `README.md` under `Later`.

---

## 3. Architecture and Boundaries

### 3.1 Canonical Logical Domain

Add canonical logical topology models to `storage-types` for:
- LVM: volume groups, logical volumes, physical volumes
- MD RAID: arrays and members
- multi-device BTRFS filesystem entities and member devices

Logical entities are first-class roots; physical disks reference membership rather than owning logical topology.

### 3.2 Discovery Ownership

- `storage-udisks`: only UDisks2-backed discovery and operations.
- `storage-sys`: any non-UDisks data gathering, CLI probing, or fallback parsing.
- `storage-service`: merges both into one normalized logical view and exposes DBus API.

---

## 4. UI/UX Design

### 4.1 Sidebar

- Keep top-level `Logical` section.
- Populate with first-class logical roots (LVM VG, MD array, multi-device BTRFS FS).
- Selecting a logical root opens logical detail view.
- Member devices can deep-link to physical disk view, but are not re-owned there.

### 4.2 Existing Controls Reuse Decision

- `disk_header`: **do not reuse** for logical entities (disk-specific semantics/actions).
- `volumes_control` segment bar: **do not reuse** (partition-layout semantics do not apply).
- detail/info cards: **reuse as base shell**, extend with logical entity action groups and metrics.

### 4.3 UI Layout & Feature Planning Requirement

Implementation planning must include a dedicated UI planning workstream with:
- final logical detail page layout
- per-entity feature matrix (actions, state, health, progress)
- reuse-vs-new view/component decisions
- UX acceptance criteria per entity type

Detailed UI planning output is captured in:
- `docs/plans/2026-02-26-logical-volume-support-ui-plan.md`

---

## 5. Service API and Data Flow

### 5.1 New Logical Interface

Add a dedicated logical handler interface in `storage-service` with:
- list logical entities
- get logical entity details
- list member devices and member state
- expose capability flags and blocked reasons

### 5.2 App Consumption

`storage-app` consumes:
- existing physical APIs (`Disks`)
- new logical APIs (`Logical`)

Selection and navigation map between both via stable IDs/device paths.

### 5.3 Operations Surface (Full-management v1)

- LVM: create/delete VG, create/delete/resize LV, add/remove PV, activate/deactivate.
- MD RAID: create/delete arrays, add/remove members, start/stop, check/repair sync actions.
- BTRFS (multi-device): add/remove device, resize, label/default-subvolume updates, existing subvolume/snapshot actions.

---

## 6. Logical Type Coverage Matrix

### 6.1 In-scope (v1)

- LVM (VG/LV/PV)
- MD RAID arrays
- multi-device BTRFS filesystem entities
- existing LUKS container interactions where nested within logical stacks

### 6.2 Already Present Partially

- LVM and LUKS are partially represented already.
- BTRFS management is partially implemented (subvolume/snapshot-oriented).

### 6.3 Explicitly Out of v1

- Stratis
- ZFS
- dm-cache / dm-writecache
- VDO
- bcache / bcachefs

---

## 7. Error Handling and Safety

1. Every logical entity reports capability flags and blocked reasons.
2. UI disables inapplicable operations with explicit reason text.
3. Service returns typed operation errors with actionable guidance.
4. Destructive operations require clear confirmation and preflight checks.

---

## 8. Rollout Strategy

1. Phase 1: logical topology read-path + sidebar/detail rendering.
2. Phase 2: safe management actions (activation/mount/check/repair paths).
3. Phase 3: destructive/structural management actions (create/delete/reshape/add-remove).

---

## 9. Validation Plan

1. Unit tests for normalization, mapping, and capability derivation.
2. Integration tests for logical APIs across representative topologies.
3. UI tests for sidebar grouping, routing, and action enablement/disablement.
4. Manual scenario matrix including nested/stacked combinations (e.g., LUKS on LV).

---

## 10. Acceptance Criteria

1. `Logical` sidebar section is populated with first-class logical roots.
2. Cross-device spanning entities are represented as aggregates, not inferred physical children.
3. Crate boundary rule is enforced (`udisks` only for UDisks-backed discovery; non-UDisks in `sys`).
4. Full-management v1 operation set is available for in-scope logical classes.
5. Out-of-v1 logical classes are documented in README as future scope.

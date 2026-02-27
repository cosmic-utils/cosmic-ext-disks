# Logical Volume Support Migration Checklist

**Date:** 2026-02-26  
**Scope:** Detailed actionable checklist for the approved LVM/RAID/Logical support plan, including UI planning, data model changes, service contracts, and app integration.

---

## A. Preparation

- [x] Confirm branch scope includes logical topology + management work only (no unrelated feature bundling).
- [x] Run baseline verification `just check`.
- [x] Record baseline verification timestamp and outcome in planning docs.
- [x] Create approved design doc for logical support.
- [x] Create approved implementation plan for logical support.
- [x] Create dedicated UI planning doc for logical layout + feature matrix.
- [x] Confirm out-of-v1 logical classes are explicitly documented in `README.md` under `Later`.

## B. Global Guardrails

- [x] Preserve strict crate boundaries:
  - `storage-udisks`: UDisks-backed discovery/operations only
  - `storage-sys`: non-UDisks probing/fallbacks only
  - `storage-service`: merge/orchestrate/authz/expose DBus
  - `storage-app`: UI state, navigation, and interactions
- [x] Keep physical disk/partition workflows behavior-compatible while logical flows are added.
- [x] Do not force logical entities into partition-segment models.
- [x] Reuse app visual primitives where practical; avoid copy-paste control logic.
- [x] Keep APIs explicit and typed; avoid ad-hoc stringly contracts where canonical types exist.

## C. Documentation and Planning Tasks

## C1. Baseline and Planning Artifacts

- [x] Add/validate `docs/plans/2026-02-26-logical-volume-support-design.md`.
- [x] Add/validate `docs/plans/2026-02-26-logical-volume-support-ui-plan.md`.
- [x] Add/validate `docs/plans/2026-02-26-logical-volume-support-implementation-plan.md`.

## C2. Out-of-v1 Scope Documentation

- [x] Add explicit out-of-v1 logical classes in `README.md`:
  - Stratis
  - ZFS
  - dm-cache / dm-writecache
  - VDO
  - bcache / bcachefs

## D. storage-types Tasks

## D1. Canonical Logical Domain Models

- [x] Create `storage-types/src/logical.rs`.
- [x] Define top-level logical entity identity type(s) with stable IDs.
- [x] Define entity-kind enum covering:
  - LVM VG/LV/PV
  - MD RAID array/member
  - multi-device BTRFS filesystem/device/subvolume context
- [x] Define per-entity payload structures for health, size, progress, and status metadata.
- [x] Define capability/blocked-reason structures for operation gating.
- [x] Add serde derives and ensure external JSON stability.

## D2. Exports and API Surface

- [x] Wire `pub mod logical;` in `storage-types/src/lib.rs`.
- [x] Add explicit re-exports for logical model types in `storage-types/src/lib.rs`.
- [x] Keep existing public exports intact (no unrelated API churn).

## D3. Tests

- [x] Add unit tests for serde roundtrip on logical models.
- [x] Add unit tests for entity-kind and capability serialization consistency.
- [x] Add unit tests for member-link and aggregate-summary helper correctness.

## E. storage-udisks Tasks (UDisks-only Discovery)

## E1. New Logical Discovery Module

- [x] Create `storage-udisks/src/logical/mod.rs`.
- [x] Create `storage-udisks/src/logical/lvm_udisks.rs`.
- [x] Create `storage-udisks/src/logical/mdraid_udisks.rs`.
- [x] Create `storage-udisks/src/logical/btrfs_udisks.rs`.
- [x] Export public entry points from `storage-udisks/src/lib.rs`.

## E2. UDisks Mapping Implementation

- [x] Map UDisks LVM interfaces to canonical LVM logical models.
- [x] Map UDisks MD RAID interfaces/properties to canonical RAID models.
- [x] Map UDisks BTRFS filesystem interfaces to multi-device BTRFS models.
- [x] Normalize object-path/device-path references for stable identity.

## E3. Boundary Validation

- [x] Verify no CLI probing is introduced in `storage-udisks/src/logical/**`.
- [x] Confirm no direct `Command::new` usage inside new UDisks logical modules.

## E4. Tests

- [x] Add fixture-driven mapper tests for each logical family (LVM/RAID/BTRFS).
- [x] Add tests for missing/partial property handling.
- [x] Add tests for deterministic ordering of entity/member lists.

## F. storage-sys Tasks (Non-UDisks Discovery)

## F1. New Fallback/Tooling Module

- [x] Create `storage-sys/src/logical/mod.rs`.
- [x] Create `storage-sys/src/logical/lvm_tools.rs`.
- [x] Create `storage-sys/src/logical/mdadm_tools.rs`.
- [x] Create `storage-sys/src/logical/btrfs_tools.rs`.
- [x] Export module façade from `storage-sys/src/lib.rs`.

## F2. Non-UDisks Probing

- [x] Implement parser/normalizer for LVM command output fallbacks.
- [x] Implement parser/normalizer for mdadm/system RAID fallbacks.
- [x] Implement parser/normalizer for BTRFS multi-device fallbacks.
- [x] Convert fallback data into canonical `storage-types::logical` models.

## F3. Tests

- [x] Add fixture-based parser tests for each tool output shape.
- [x] Add tests for degraded/error-path parsing resilience.
- [x] Add tests for malformed output handling with safe errors.

## G. storage-service Tasks

## G1. Policies and Handler Wiring

- [x] Create `storage-service/src/policies/logical.rs`.
- [x] Register policy module in `storage-service/src/policies/mod.rs`.
- [x] Create `storage-service/src/handlers/logical.rs`.
- [x] Register handler in `storage-service/src/handlers/mod.rs`.
- [x] Serve handler in `storage-service/src/main.rs` at a dedicated DBus path.

## G2. Read APIs

- [x] Implement list logical entities method.
- [x] Implement get logical entity detail method.
- [x] Implement list/query members and state method.
- [x] Return capability flags + blocked-reasons with all relevant entities.

## G3. Management APIs (v1)

- [x] LVM ops:
  - create/delete VG
  - add/remove PV
  - create/delete/resize LV
  - activate/deactivate LV
- [x] MD RAID ops:
  - create/delete array
  - start/stop array
  - add/remove member
  - request check/repair/idle sync actions
- [x] BTRFS multi-device ops:
  - add/remove device
  - resize
  - set label
  - set/get default subvolume

## G4. Orchestration and Error Model

- [x] Merge UDisks and sys discovery into one normalized logical topology response.
- [x] Ensure typed error mapping with actionable messages.
- [x] Ensure policy denial reasons are surfaced for disabled UI states.
- [x] Add/update signals for logical topology/operation changes as needed.

## G5. Tests

- [x] Add handler tests for list/get logical APIs.
- [x] Add handler tests for operation authz/denial paths.
- [x] Add handler tests for operation success/error normalization.

## H. storage-app Tasks

## H1. Client Layer

- [x] Create `storage-app/src/client/logical.rs` for DBus logical API.
- [x] Register client module in `storage-app/src/client/mod.rs`.
- [x] Add typed parse/deserialize handling aligned with `storage-types::logical`.

## H2. State and Messages

- [x] Create `storage-app/src/state/logical.rs`.
- [x] Register logical state in `storage-app/src/state/mod.rs`.
- [x] Add logical messages in `storage-app/src/message/**`.
- [x] Add logical update handlers under `storage-app/src/update/**`.

## H3. Sidebar Integration

- [x] Populate `Logical` section in `storage-app/src/views/sidebar.rs` with logical roots.
- [x] Preserve ordering with existing Internal/External/Network/Images sections.
- [x] Implement expand/collapse and selection behavior for logical nodes.

## H4. Logical Detail Views

- [x] Create `storage-app/src/views/logical.rs`.
- [x] Route selected logical nodes to logical detail views from `storage-app/src/views/app.rs`.
- [x] Implement recommended layout zones:
  - title/status row
  - summary metrics row
  - action toolbar
  - tabbed body (`Overview`, `Members`, `Operations`, conditional `BTRFS`)

## H5. LogicalTopologyControl (New Control)

- [x] Create `LogicalTopologyControl` under `storage-app/src/controls/logical/*`.
- [x] Mirror VolumesControl interaction style for navigation, without segment semantics.
- [x] Support logical hierarchies:
  - VG -> LV (+ member context)
  - mdX -> members/spares
  - BTRFS FS -> devices -> subvolume context
- [x] Reuse visual primitives/styles from existing controls where practical.

## H6. Action Wiring and Disabled Reasons

- [x] Wire action buttons to logical client/service APIs.
- [x] Show disabled reason text for inapplicable operations.
- [x] Add confirmation/prefight UI for destructive actions.
- [x] Ensure success/error status is visible in the logical detail surface.

## H7. Compatibility Constraints

- [x] Keep current disk/partition selection and operation flows working unchanged.
- [x] Avoid coupling logical selection state to partition-segment assumptions.

## H8. Tests

- [x] Add state tests for logical selection/refresh transitions.
- [x] Add view tests for logical sidebar grouping/rendering.
- [x] Add update tests for logical operation dispatch and status handling.

## I. Cross-Crate Type Coverage Validation

- [x] Validate first-class support coverage includes:
  - LVM VG/LV/PV
  - MD RAID arrays/members
  - multi-device BTRFS filesystems/devices
- [x] Validate LUKS treatment remains container-layer (not aggregate root) while nested interactions remain supported.
- [x] Validate out-of-v1 classes are not silently partially implemented as unstable UX.

## J. Verification Tasks

- [x] Run targeted crate tests as each crate section completes.
- [x] Run `just check` after major integration waves.
- [x] Validate no boundary violations (`udisks` doing non-UDisks probing, `sys` duplicating UDisks mapping responsibilities).
- [x] Validate no regressions in existing physical storage workflows.
- [x] Run final `just check` with no new warnings/errors introduced by this work.

## K. PR Hygiene

- [x] Keep commits focused by task wave (docs/types/udisks/sys/service/app/verification).
- [x] Avoid unrelated refactors while executing logical support plan.
- [x] Update planning docs status notes before final merge request.

## L. Done Definition

- [x] `Logical` sidebar section shows first-class logical roots.
- [x] Logical details and navigation are powered by `LogicalTopologyControl`, not partition-segment control semantics.
- [x] Service exposes logical read/manage APIs with capability/blocked-reason coverage.
- [x] UDisks-only vs non-UDisks discovery boundaries are enforced by crate.
- [x] In-scope v1 logical families are manageable end-to-end.
- [x] Out-of-v1 classes remain clearly documented as future scope.
- [x] Workspace passes verification (`just check`).

## M. Approach B Review Plan (Dialog/Wizard Completion)

## M1. Locked UX Decisions (No Branching)

- [x] Approve Approach B as the final app-side path: operation dialogs/wizards using existing `ShowDialog` architecture.
- [x] Approve the exact operation UX matrix:
  - **Wizard flows (configuration operations):**
    - LVM: create VG, delete VG, add PV, remove PV, create LV, delete LV, resize LV
    - MD RAID: create array, delete array, add member, remove member
    - BTRFS: add device, remove device, resize, set label, set default subvolume
  - **Single-step dialogs (control operations):**
    - LVM: activate LV, deactivate LV
    - MD RAID: start array, stop array, sync action (`check`, `repair`, `idle`)
- [x] Approve destructive actions always require explicit confirmation step in the same flow.

## M2. Exact File Edit Plan (In Order)

- [x] Update `storage-app/src/state/dialogs.rs`:
  - Add `ShowDialog` variants for logical wizard/single-step flows.
  - Add state structs for each logical flow family (`LogicalLvmWizardDialog`, `LogicalMdRaidWizardDialog`, `LogicalBtrfsWizardDialog`, `LogicalControlDialog`).
  - Add step enums for each wizard family with numbered steps.
- [x] Update `storage-app/src/message/dialogs.rs`:
  - Add logical dialog message enums with explicit step navigation and field updates.
  - Add confirm/cancel/submit variants per logical flow family.
- [x] Update `storage-app/src/message/app.rs`:
  - Add explicit messages to open each logical dialog/wizard flow.
  - Add explicit messages to route logical dialog sub-messages.
- [x] Update `storage-app/src/views/dialogs/mod.rs`:
  - Export new logical dialog renderer module(s).
- [x] Create `storage-app/src/views/dialogs/logical.rs`:
  - Implement wizard UIs using existing wizard primitives.
  - Implement single-step control dialogs for activate/deactivate/start/stop/sync-action.
- [x] Update `storage-app/src/views/app.rs`:
  - Route new logical dialog variants through `dialog()`.
- [x] Update `storage-app/src/views/logical.rs`:
  - Replace direct action prompt buttons with explicit dialog open messages.
  - Keep operation availability/disabled reasons visible at action entry points.
- [x] Update `storage-app/src/client/logical.rs`:
  - Ensure full wrapper coverage for all in-scope v1 service methods used by flows.
- [x] Update `storage-app/src/update/mod.rs` and/or `storage-app/src/update/logical.rs`:
  - Add logical dialog open handlers.
  - Add logical dialog state reducers.
  - Add operation submit handlers that map dialog payloads to exact client calls.
  - Keep post-operation behavior fixed: clear dialog, set status message, reload logical entities.
- [x] Update `storage-app/src/update/nav.rs`:
  - Include logical dialog running-state coverage so navigation disable behavior remains consistent.

## M3. Operation-to-Method Mapping Checklist (Mandatory)

- [x] LVM mappings:
  - `create VG` → `lvm_create_volume_group(vg_name, devices_json)`
  - `delete VG` → `lvm_delete_volume_group(vg_name)`
  - `add PV` → `lvm_add_physical_volume(vg_name, pv_device)`
  - `remove PV` → `lvm_remove_physical_volume(vg_name, pv_device)`
  - `create LV` → `lvm_create_logical_volume(vg_name, lv_name, size_bytes)`
  - `delete LV` → `lvm_delete_logical_volume(lv_path)`
  - `resize LV` → `lvm_resize_logical_volume(lv_path, size_bytes)`
  - `activate LV` → `lvm_activate_logical_volume(lv_path)`
  - `deactivate LV` → `lvm_deactivate_logical_volume(lv_path)`
- [x] MD RAID mappings:
  - `create array` → `mdraid_create_array(array_device, level, devices_json)`
  - `delete array` → `mdraid_delete_array(array_device)`
  - `start array` → `mdraid_start_array(array_device)`
  - `stop array` → `mdraid_stop_array(array_device)`
  - `add member` → `mdraid_add_member(array_device, member_device)`
  - `remove member` → `mdraid_remove_member(array_device, member_device)`
  - `sync action` → `mdraid_request_sync_action(md_name, action)`
- [x] BTRFS mappings:
  - `add device` → `btrfs_add_device(member_device, mount_point)`
  - `remove device` → `btrfs_remove_device(member_device, mount_point)`
  - `resize` → `btrfs_resize(size_spec, mount_point)`
  - `set label` → `btrfs_set_label(mount_point, label)`
  - `set default subvolume` → `btrfs_set_default_subvolume(subvolume_id, mount_point)`

## M4. Wizard Field Contracts (Mandatory)

- [x] LVM wizard fields:
  - VG create: `vg_name`, `devices[]`
  - VG delete: `vg_name` + confirmation
  - PV add/remove: `vg_name`, `pv_device`
  - LV create: `vg_name`, `lv_name`, `size_bytes`
  - LV delete: `lv_path` + confirmation
  - LV resize: `lv_path`, `size_bytes`
- [x] MD RAID wizard fields:
  - Create: `array_device`, `level`, `devices[]`
  - Delete: `array_device` + confirmation
  - Add/remove member: `array_device`, `member_device`
- [x] BTRFS wizard fields:
  - Add/remove device: `member_device`, `mount_point`
  - Resize: `size_spec`, `mount_point`
  - Set label: `mount_point`, `label`
  - Set default subvolume: `subvolume_id`, `mount_point`
- [x] Single-step control dialog fields:
  - Activate/deactivate LV: `lv_path`
  - Start/stop array: `array_device`
  - Sync action: `md_name`, `action` (one of `check|repair|idle`)

## M5. Validation + Error Surface Requirements

- [x] Add deterministic client-side validation before submit for all required fields.
- [x] Show per-field validation errors in dialog body before submit.
- [x] Surface service/policy errors in dialog error area without swallowing message text.
- [x] Keep disabled reason visibility in logical detail actions for blocked operations.

## M6. Test Execution Plan (Exact Commands)

- [x] Add/extend tests in:
  - `storage-app/src/state/logical.rs` (state transitions for dialog open/close/reset)
  - `storage-app/src/views/logical.rs` and `storage-app/src/views/dialogs/logical.rs` (render + action availability)
  - `storage-app/src/update/mod.rs` and/or `storage-app/src/update/logical.rs` (dispatch-to-client mapping)
- [x] Run targeted command:
  - `cargo test -p cosmic-ext-storage logical -- --nocapture`
- [x] Run full workspace verification command:
  - `just check`

## M7. Completion Gate (Strict)

- [x] Mark this section complete only after M1-M6 are complete and both verification commands pass.
- [x] Mark `L. In-scope v1 logical families are manageable end-to-end` complete immediately after M7 passes.
- [x] Record timestamped verification evidence in this checklist with exact command outputs summary.

### Verification Evidence (2026-02-26)

- `cargo test -p cosmic-ext-storage logical -- --nocapture`: passed (6 tests).
- `just check`: passed (`cargo clippy --workspace --all-targets`, `cargo fmt --all -- --check`, `cargo test --workspace --no-run`).

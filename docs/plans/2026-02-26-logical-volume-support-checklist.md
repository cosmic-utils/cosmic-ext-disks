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

- [ ] Preserve strict crate boundaries:
  - `storage-udisks`: UDisks-backed discovery/operations only
  - `storage-sys`: non-UDisks probing/fallbacks only
  - `storage-service`: merge/orchestrate/authz/expose DBus
  - `storage-app`: UI state, navigation, and interactions
- [ ] Keep physical disk/partition workflows behavior-compatible while logical flows are added.
- [ ] Do not force logical entities into partition-segment models.
- [ ] Reuse app visual primitives where practical; avoid copy-paste control logic.
- [ ] Keep APIs explicit and typed; avoid ad-hoc stringly contracts where canonical types exist.

## C. Documentation and Planning Tasks

## C1. Baseline and Planning Artifacts

- [x] Record baseline pass in `docs/plans/2026-02-26-layered-crate-structure-checklist.md`.
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

- [ ] Create `storage-types/src/logical.rs`.
- [ ] Define top-level logical entity identity type(s) with stable IDs.
- [ ] Define entity-kind enum covering:
  - LVM VG/LV/PV
  - MD RAID array/member
  - multi-device BTRFS filesystem/device/subvolume context
- [ ] Define per-entity payload structures for health, size, progress, and status metadata.
- [ ] Define capability/blocked-reason structures for operation gating.
- [ ] Add serde derives and ensure external JSON stability.

## D2. Exports and API Surface

- [ ] Wire `pub mod logical;` in `storage-types/src/lib.rs`.
- [ ] Add explicit re-exports for logical model types in `storage-types/src/lib.rs`.
- [ ] Keep existing public exports intact (no unrelated API churn).

## D3. Tests

- [ ] Add unit tests for serde roundtrip on logical models.
- [ ] Add unit tests for entity-kind and capability serialization consistency.
- [ ] Add unit tests for member-link and aggregate-summary helper correctness.

## E. storage-udisks Tasks (UDisks-only Discovery)

## E1. New Logical Discovery Module

- [ ] Create `storage-udisks/src/logical/mod.rs`.
- [ ] Create `storage-udisks/src/logical/lvm_udisks.rs`.
- [ ] Create `storage-udisks/src/logical/mdraid_udisks.rs`.
- [ ] Create `storage-udisks/src/logical/btrfs_udisks.rs`.
- [ ] Export public entry points from `storage-udisks/src/lib.rs`.

## E2. UDisks Mapping Implementation

- [ ] Map UDisks LVM interfaces to canonical LVM logical models.
- [ ] Map UDisks MD RAID interfaces/properties to canonical RAID models.
- [ ] Map UDisks BTRFS filesystem interfaces to multi-device BTRFS models.
- [ ] Normalize object-path/device-path references for stable identity.

## E3. Boundary Validation

- [ ] Verify no CLI probing is introduced in `storage-udisks/src/logical/**`.
- [ ] Confirm no direct `Command::new` usage inside new UDisks logical modules.

## E4. Tests

- [ ] Add fixture-driven mapper tests for each logical family (LVM/RAID/BTRFS).
- [ ] Add tests for missing/partial property handling.
- [ ] Add tests for deterministic ordering of entity/member lists.

## F. storage-sys Tasks (Non-UDisks Discovery)

## F1. New Fallback/Tooling Module

- [ ] Create `storage-sys/src/logical/mod.rs`.
- [ ] Create `storage-sys/src/logical/lvm_tools.rs`.
- [ ] Create `storage-sys/src/logical/mdadm_tools.rs`.
- [ ] Create `storage-sys/src/logical/btrfs_tools.rs`.
- [ ] Export module façade from `storage-sys/src/lib.rs`.

## F2. Non-UDisks Probing

- [ ] Implement parser/normalizer for LVM command output fallbacks.
- [ ] Implement parser/normalizer for mdadm/system RAID fallbacks.
- [ ] Implement parser/normalizer for BTRFS multi-device fallbacks.
- [ ] Convert fallback data into canonical `storage-types::logical` models.

## F3. Tests

- [ ] Add fixture-based parser tests for each tool output shape.
- [ ] Add tests for degraded/error-path parsing resilience.
- [ ] Add tests for malformed output handling with safe errors.

## G. storage-service Tasks

## G1. Policies and Handler Wiring

- [ ] Create `storage-service/src/policies/logical.rs`.
- [ ] Register policy module in `storage-service/src/policies/mod.rs`.
- [ ] Create `storage-service/src/handlers/logical.rs`.
- [ ] Register handler in `storage-service/src/handlers/mod.rs`.
- [ ] Serve handler in `storage-service/src/main.rs` at a dedicated DBus path.

## G2. Read APIs

- [ ] Implement list logical entities method.
- [ ] Implement get logical entity detail method.
- [ ] Implement list/query members and state method.
- [ ] Return capability flags + blocked-reasons with all relevant entities.

## G3. Management APIs (v1)

- [ ] LVM ops:
  - create/delete VG
  - add/remove PV
  - create/delete/resize LV
  - activate/deactivate LV
- [ ] MD RAID ops:
  - create/delete array
  - start/stop array
  - add/remove member
  - request check/repair/idle sync actions
- [ ] BTRFS multi-device ops:
  - add/remove device
  - resize
  - set label
  - set/get default subvolume

## G4. Orchestration and Error Model

- [ ] Merge UDisks and sys discovery into one normalized logical topology response.
- [ ] Ensure typed error mapping with actionable messages.
- [ ] Ensure policy denial reasons are surfaced for disabled UI states.
- [ ] Add/update signals for logical topology/operation changes as needed.

## G5. Tests

- [ ] Add handler tests for list/get logical APIs.
- [ ] Add handler tests for operation authz/denial paths.
- [ ] Add handler tests for operation success/error normalization.

## H. storage-app Tasks

## H1. Client Layer

- [ ] Create `storage-app/src/client/logical.rs` for DBus logical API.
- [ ] Register client module in `storage-app/src/client/mod.rs`.
- [ ] Add typed parse/deserialize handling aligned with `storage-types::logical`.

## H2. State and Messages

- [ ] Create `storage-app/src/state/logical.rs`.
- [ ] Register logical state in `storage-app/src/state/mod.rs`.
- [ ] Add logical messages in `storage-app/src/message/**`.
- [ ] Add logical update handlers under `storage-app/src/update/**`.

## H3. Sidebar Integration

- [ ] Populate `Logical` section in `storage-app/src/views/sidebar.rs` with logical roots.
- [ ] Preserve ordering with existing Internal/External/Network/Images sections.
- [ ] Implement expand/collapse and selection behavior for logical nodes.

## H4. Logical Detail Views

- [ ] Create `storage-app/src/views/logical.rs`.
- [ ] Route selected logical nodes to logical detail views from `storage-app/src/views/app.rs`.
- [ ] Implement recommended layout zones:
  - title/status row
  - summary metrics row
  - action toolbar
  - tabbed body (`Overview`, `Members`, `Operations`, conditional `BTRFS`)

## H5. LogicalTopologyControl (New Control)

- [ ] Create `LogicalTopologyControl` under `storage-app/src/controls/logical/*`.
- [ ] Mirror VolumesControl interaction style for navigation, without segment semantics.
- [ ] Support logical hierarchies:
  - VG -> LV (+ member context)
  - mdX -> members/spares
  - BTRFS FS -> devices -> subvolume context
- [ ] Reuse visual primitives/styles from existing controls where practical.

## H6. Action Wiring and Disabled Reasons

- [ ] Wire action buttons to logical client/service APIs.
- [ ] Show disabled reason text for inapplicable operations.
- [ ] Add confirmation/prefight UI for destructive actions.
- [ ] Ensure success/error status is visible in the logical detail surface.

## H7. Compatibility Constraints

- [ ] Keep current disk/partition selection and operation flows working unchanged.
- [ ] Avoid coupling logical selection state to partition-segment assumptions.

## H8. Tests

- [ ] Add state tests for logical selection/refresh transitions.
- [ ] Add view tests for logical sidebar grouping/rendering.
- [ ] Add update tests for logical operation dispatch and status handling.

## I. Cross-Crate Type Coverage Validation

- [ ] Validate first-class support coverage includes:
  - LVM VG/LV/PV
  - MD RAID arrays/members
  - multi-device BTRFS filesystems/devices
- [ ] Validate LUKS treatment remains container-layer (not aggregate root) while nested interactions remain supported.
- [ ] Validate out-of-v1 classes are not silently partially implemented as unstable UX.

## J. Verification Tasks

- [ ] Run targeted crate tests as each crate section completes.
- [ ] Run `just check` after major integration waves.
- [ ] Validate no boundary violations (`udisks` doing non-UDisks probing, `sys` duplicating UDisks mapping responsibilities).
- [ ] Validate no regressions in existing physical storage workflows.
- [ ] Run final `just check` with no new warnings/errors introduced by this work.

## K. PR Hygiene

- [ ] Keep commits focused by task wave (docs/types/udisks/sys/service/app/verification).
- [ ] Avoid unrelated refactors while executing logical support plan.
- [ ] Update planning docs status notes before final merge request.

## L. Done Definition

- [ ] `Logical` sidebar section shows first-class logical roots.
- [ ] Logical details and navigation are powered by `LogicalTopologyControl`, not partition-segment control semantics.
- [ ] Service exposes logical read/manage APIs with capability/blocked-reason coverage.
- [ ] UDisks-only vs non-UDisks discovery boundaries are enforced by crate.
- [ ] In-scope v1 logical families are manageable end-to-end.
- [ ] Out-of-v1 classes remain clearly documented as future scope.
- [ ] Workspace passes verification (`just check`).

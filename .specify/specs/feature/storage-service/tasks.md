# Tasks: Audit 2026-02-14 Gap Remediation

**Input**: Design documents from `.copi/specs/fix/audit-2026-02-14-gaps/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US7)

---

## Phase 1: Critical Fixes - US1 Eliminate Panics (Priority: P1)

**Purpose**: Fix all panic paths in production code

### GAP-002: Required Connection in VolumeNode/VolumeModel

- [ ] T001 [US1] Make Connection required (not Option) in `storage-dbus/src/disks/volume.rs:44`
- [ ] T002 [US1] Update VolumeNode constructor to require Connection parameter
- [ ] T003 [P] [US1] Fix volume_model/partition.rs: Remove Option<Connection>, update methods
- [ ] T004 [P] [US1] Fix volume_model/filesystem.rs: Remove Option<Connection>, update methods
- [ ] T005 [US1] Update all callers passing None to pass actual Connection
- [ ] T006 [US1] Remove all `self.connection.as_ref().unwrap()` patterns

### GAP-003: Remove Blocking Runtime in Clone

- [ ] T007 [US1] Create ClientPool struct in `storage-ui/src/client/pool.rs`
- [ ] T008 [US1] Implement ClientPool::new() with all client types
- [ ] T009 [US1] Add `clients: Arc<ClientPool>` to AppModel in `storage-ui/src/app.rs`
- [ ] T010 [US1] Initialize ClientPool in AppModel startup
- [ ] T011 [US1] Update UiDrive to store `Arc<ClientPool>` instead of owned clients
- [ ] T012 [US1] Remove custom Clone impl from UiDrive, derive or simplify
- [ ] T013 [US1] Remove Runtime::new()/block_on() from `storage-ui/src/models/helpers.rs:93-94`
- [ ] T014 [US1] Update helpers.rs to receive Arc<ClientPool> as parameter

### GAP-005 (Partial): Fix Unwrap in Hot Paths

- [ ] T015 [P] [US1] Fix string parsing unwrap in `storage-models/src/common.rs:119,122`
- [ ] T016 [P] [US1] Fix path manipulation unwrap in `storage-btrfs/src/subvolume.rs:68`

### GAP-011: Handle Mutex Poisoning

- [ ] T017 [US1] Replace `.lock().unwrap()` with `.lock().expect()` in `storage-dbus/src/disks/ops.rs`
- [ ] T018 [US1] Add context messages to all 30+ mutex lock sites

**Checkpoint**: `cargo clippy --workspace` passes, no unwrap warnings in hot paths

---

## Phase 2: High Priority - US2 Volume Hierarchy (Priority: P1)

**Purpose**: Fix volume tree display for LUKS/LVM/BTRFS

### GAP-004: Parent Path Population

- [ ] T019 [US2] Add parent_device parameter to flatten_volumes function
- [ ] T020 [US2] Set vol_info.parent_path in flatten_volumes before recursion
- [ ] T021 [US2] Update flatten_volumes callers to pass None initially
- [ ] T022 [US2] Remove TODO comment from `storage-dbus/src/disks/volume.rs:651`
- [ ] T023 [US2] Test: LUKS cleartext shows parent_path = encrypted container
- [ ] T024 [US2] Test: LVM LVs show parent_path = PV device

**Checkpoint**: Volume tree displays correctly for all volume types

---

## Phase 3: High Priority - US3/US4 Clear Errors & Architecture (Priority: P2)

**Purpose**: Improve error messages and client architecture

### GAP-008: Input Validation

- [ ] T025 [P] [US3] Add device path validation (must start with /dev/)
- [ ] T026 [P] [US3] Add size > 0 validation
- [ ] T027 [P] [US3] Add offset alignment validation
- [ ] T028 [US3] Add type_id format validation (GUID for GPT, hex for DOS)
- [ ] T029 [US3] Add offset + size <= disk capacity validation
- [ ] T030 [US3] Return InvalidArgs FDO error type with clear messages

### GAP-012: Error Context Preservation

- [ ] T031 [P] [US3] Update ClientError variants to include dbus_name field
- [ ] T032 [US3] Update From<zbus::Error> to preserve error names
- [ ] T033 [US3] Add source context to Connection error variant

### GAP-016: Refresh Strategy Documentation

- [ ] T034 [US4] Add RefreshResult enum with Updated/NotFound/Failed variants
- [ ] T035 [US4] Update refresh_volume to return RefreshResult
- [ ] T036 [US4] Document refresh strategy in module doc comment
- [ ] T037 [US4] Log warning when atomic refresh returns NotFound

**Checkpoint**: Clear error messages for all invalid inputs, documented refresh strategy

---

## Phase 4: Medium Priority - US6 Integration Tests (Priority: P2)

**Purpose**: Prevent serialization regressions

### GAP-015: Integration Test Scaffolding

- [ ] T038 [US6] Create `tests/integration/` directory structure
- [ ] T039 [P] [US6] Create serialization round-trip test for DiskInfo
- [ ] T040 [P] [US6] Create serialization round-trip test for VolumeInfo
- [ ] T041 [P] [US6] Create serialization round-trip test for PartitionInfo
- [ ] T042 [US6] Create client error mapping test for PermissionDenied
- [ ] T043 [US6] Create client error mapping test for ServiceNotAvailable

### GAP-007: Serialization Tests

- [ ] T044 [US6] Document JSON-over-D-Bus decision in architecture.md

**Checkpoint**: Integration tests pass, serialization contracts verified

---

## Phase 5: Polish - US7 Tech Debt (Priority: P3)

**Purpose**: Clean up remaining technical debt

### GAP-009: Delete Conversions Module

- [ ] T045 [US7] Verify all storage-dbus APIs return storage-models types
- [ ] T046 [US7] Delete `storage-service/src/conversions.rs`
- [ ] T047 [US7] Remove conversions module import from storage-service

### GAP-010: TODO Cleanup

- [ ] T048 [P] [US7] Link partition.rs:195 TODO to GitHub issue
- [ ] T049 [P] [US7] Link encryption.rs:126 TODO to GitHub issue
- [ ] T050 [P] [US7] Link encryption.rs:301 TODO to GitHub issue

### GAP-013: Timeouts and Progress

- [ ] T051 [US5] Add format_with_progress method to FilesystemsClient
- [ ] T052 [US5] Subscribe to FormatProgress signal in format_with_progress
- [ ] T053 [US5] Wrap format call in timeout (10 min default)
- [ ] T054 [US5] Add Timeout variant to ClientError

### GAP-014: Service Already Running Check

- [ ] T055 [US7] Add is_service_already_running() function to main.rs
- [ ] T056 [US7] Call check before ConnectionBuilder, exit with clear message if running

**Checkpoint**: Clean codebase, no orphaned TODOs

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Critical) ──► Phase 2 (Hierarchy) ──► Phase 3 (Errors/Arch)
                              │
                              ▼
                       Phase 4 (Tests) ──► Phase 5 (Polish)
```

### Within Phase 1

```
T007-T010 (ClientPool) ──► T011-T014 (UiDrive refactor)
        │
        └──► T001-T006 (Connection) [can run in parallel with ClientPool]
```

### Parallel Opportunities

- T003, T004: Different files, can run together
- T015, T016: Different crates, can run together
- T025-T027: Independent validations, can run together
- T039-T041: Independent test files, can run together
- T048-T050: Independent TODO fixes, can run together

---

## Acceptance Criteria Summary

| Criteria | Verification |
|----------|--------------|
| No unwrap in hot paths | `grep -r "unwrap()" storage-{models,dbus,service,ui}/src/*.rs` shows <20 hits |
| No blocking runtime | `grep -r "Runtime::new\|block_on" storage-ui/src/` shows 0 hits |
| Parent path populated | All VolumeInfo objects have correct parent_path |
| Clear error messages | Invalid partition input returns actionable error |
| Integration tests pass | `cargo test --workspace --all-features` passes |
| ClientPool pattern | All UiDrive instances use Arc<ClientPool> |

---

## Notes

- T001-T006 may require updating test code to pass mock connections
- T011-T014 is dependent on T007-T010 (ClientPool must exist first)
- T045-T047 should only be done if Phase 3A migration is confirmed complete
- GAP-013 (timeouts) is P3; can be deferred if time-constrained

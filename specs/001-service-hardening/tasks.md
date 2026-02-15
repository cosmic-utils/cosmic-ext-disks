# Tasks: Service Hardening

**Input**: Design documents from `/specs/001-service-hardening/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace structure**: `storage-ui/src/`, `storage-service/src/`, `storage-dbus/src/`, `storage-common/src/`
- Based on plan.md project structure

---

## Phase 1: Setup

**Purpose**: Verify build environment and understand current codebase

- [ ] T001 Verify workspace builds cleanly with `cargo build --workspace`
- [ ] T002 Run existing tests with `cargo test --workspace --all-features` to establish baseline
- [ ] T003 [P] Review current D-Bus connection patterns in `storage-ui/src/client/*.rs`
- [ ] T004 [P] Review current discovery function in `storage-dbus/src/disk/discovery.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared types needed by multiple user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 Add `FilesystemToolInfo` struct to `storage-common/src/lib.rs` with fs_type, fs_name, command, package_hint, available fields

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 4 - Efficient Service-to-UDisks2 Communication (Priority: P1) 🎯 MVP

**Goal**: Cache D-Bus connection in DiskManager to eliminate 9+ redundant connections per operation

**Independent Test**: Measure disk enumeration time before/after; verify no new connections created after first call

### Implementation for User Story 4

- [ ] T006 [US4] Add `connection: Arc<Connection>` field to DiskManager in `storage-dbus/src/disk/manager.rs`
- [ ] T007 [US4] Update `DiskManager::new()` to establish and cache connection in `storage-dbus/src/disk/manager.rs`
- [ ] T008 [US4] Add `pub fn connection(&self) -> &Arc<Connection>` method to DiskManager in `storage-dbus/src/disk/manager.rs`
- [ ] T009 [US4] Update `get_disks_with_volumes()` signature to accept `&DiskManager` parameter in `storage-dbus/src/disk/discovery.rs`
- [ ] T010 [US4] Replace `Connection::system()` with `manager.connection()` in `storage-dbus/src/disk/discovery.rs`
- [ ] T011 [US4] Update all call sites in `storage-service/src/disks.rs` to pass manager reference to `get_disks_with_volumes()`
- [ ] T012 [US4] Verify build with `cargo build --workspace`

**Checkpoint**: Layer 2 connection caching complete - service operations should be noticeably faster

---

## Phase 4: User Story 1 - Fast Application Startup and Responsive UI (Priority: P1)

**Goal**: Cache D-Bus connection in UI layer to eliminate multiple connections during startup

**Independent Test**: Measure app startup time before/after; verify single connection used across all clients

### Implementation for User Story 1

- [ ] T013 [US1] Create `storage-ui/src/client/connection.rs` with `OnceLock<Connection>` and `shared_connection()` function
- [ ] T014 [US1] Export connection module in `storage-ui/src/client/mod.rs`
- [ ] T015 [P] [US1] Update `DisksClient::new()` to use `shared_connection()` in `storage-ui/src/client/disks.rs`
- [ ] T016 [P] [US1] Update `FilesystemsClient::new()` to use `shared_connection()` in `storage-ui/src/client/filesystems.rs`
- [ ] T017 [P] [US1] Update `PartitionsClient::new()` to use `shared_connection()` in `storage-ui/src/client/partitions.rs`
- [ ] T018 [P] [US1] Update `LuksClient::new()` to use `shared_connection()` in `storage-ui/src/client/luks.rs` (if exists)
- [ ] T019 [P] [US1] Update `LvmClient::new()` to use `shared_connection()` in `storage-ui/src/client/lvm.rs` (if exists)
- [ ] T020 [P] [US1] Update `BtrfsClient::new()` to use `shared_connection()` in `storage-ui/src/client/btrfs.rs` (if exists)
- [ ] T021 [US1] Verify build with `cargo build --workspace`

**Checkpoint**: Layer 1 connection sharing complete - app startup should be faster

---

## Phase 5: User Story 2 - Protection Against Accidental System Unmount (Priority: P1)

**Goal**: Prevent users from killing processes on critical system paths during unmount

**Independent Test**: Attempt unmount with kill_processes=true on protected path; verify error returned and displayed

### Implementation for User Story 2

- [ ] T022 [US2] Create `storage-service/src/protected_paths.rs` with `PROTECTED_SYSTEM_PATHS` constant array
- [ ] T023 [US2] Add `is_protected_path()` function with canonical path matching in `storage-service/src/protected_paths.rs`
- [ ] T024 [US2] Add `mod protected_paths;` declaration in `storage-service/src/main.rs`
- [ ] T025 [US2] Import and use `is_protected_path()` in unmount handler in `storage-service/src/filesystems.rs`
- [ ] T026 [US2] Add protected path check before kill_processes logic, return error in `UnmountResult` format in `storage-service/src/filesystems.rs`
- [ ] T027 [US2] Add tracing log for protected path rejection in `storage-service/src/filesystems.rs`
- [ ] T028 [US2] Verify build with `cargo build --workspace`

**Checkpoint**: System path protection complete - critical paths are now protected

---

## Phase 6: User Story 3 - Centralized Filesystem Tool Detection (Priority: P2)

**Goal**: Move filesystem tool detection to service, expose via D-Bus for UI feature enablement

**Independent Test**: Query service's get_filesystem_tools() method; verify accurate reflection of installed tools

### Implementation for User Story 3

- [ ] T029 [US3] Add `filesystem_tools: Vec<FilesystemToolInfo>` field to FilesystemsHandler in `storage-service/src/filesystems.rs`
- [ ] T030 [US3] Implement `detect_all_filesystem_tools()` function in `storage-service/src/filesystems.rs`
- [ ] T031 [US3] Update `FilesystemsHandler::new()` to call detection and populate fields in `storage-service/src/filesystems.rs`
- [ ] T032 [US3] Add `get_filesystem_tools()` D-Bus method returning JSON in `storage-service/src/filesystems.rs`
- [ ] T033 [US3] Add `get_filesystem_tools()` client method in `storage-ui/src/client/filesystems.rs`
- [ ] T034 [US3] Mark `storage-ui/src/utils/fs_tools.rs` as deprecated with comment (if it exists)
- [ ] T035 [US3] Verify build with `cargo build --workspace`

**Checkpoint**: FSTools consolidation complete - UI can query service for capabilities

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates and final verification

- [ ] T036 Run `cargo fmt --all --check` and fix any formatting issues
- [ ] T037 Run `cargo clippy --workspace --all-features` and fix any warnings
- [ ] T038 Run `cargo test --workspace --all-features` and verify all tests pass
- [ ] T039 Manual test: Start service, launch UI, verify startup time improvement
- [ ] T040 Manual test: Attempt unmount with kill_processes on protected path, verify error displayed
- [ ] T041 Manual test: Query get_filesystem_tools(), verify accurate tool detection

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US4 (Layer 2 caching) can start after Phase 2
  - US1 (Layer 1 caching) can start after Phase 2 (independent of US4)
  - US2 (Protected Paths) can start after Phase 2 (independent of US1/US4)
  - US3 (FSTools) can start after Phase 2 (independent of US1/US2/US4)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US4 (P1)**: No dependencies - can start after Foundational
- **US1 (P1)**: No dependencies - can start after Foundational (parallel with US4)
- **US2 (P1)**: No dependencies - can start after Foundational (parallel with US1/US4)
- **US3 (P2)**: No dependencies - can start after Foundational

### Within Each User Story

- Tasks generally sequential within a story (same files modified)
- Tasks marked [P] within a story can run in parallel (different files)

### Parallel Opportunities

- T003, T004 can run in parallel (different files, exploration only)
- T015, T016, T017, T018, T019, T020 can run in parallel (different client files)
- US1, US2, US4 can be worked on in parallel by different developers after Phase 2

---

## Parallel Example: User Story 1 (Client Updates)

```bash
# All client updates can be done in parallel (different files):
Task: "Update DisksClient::new() to use shared_connection() in storage-ui/src/client/disks.rs"
Task: "Update FilesystemsClient::new() to use shared_connection() in storage-ui/src/client/filesystems.rs"
Task: "Update PartitionsClient::new() to use shared_connection() in storage-ui/src/client/partitions.rs"
Task: "Update LuksClient::new() to use shared_connection() in storage-ui/src/client/luks.rs"
Task: "Update LvmClient::new() to use shared_connection() in storage-ui/src/client/lvm.rs"
Task: "Update BtrfsClient::new() to use shared_connection() in storage-ui/src/client/btrfs.rs"
```

---

## Implementation Strategy

### MVP First (User Story 4 Only - Highest Impact)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 4 (Layer 2 caching - biggest performance win)
4. **STOP and VALIDATE**: Measure disk enumeration time improvement
5. Deploy/demo if ready

### Recommended Order (All P1 Stories)

1. Complete Setup + Foundational → Foundation ready
2. Complete US4 (Layer 2 caching) → Service operations faster
3. Complete US1 (Layer 1 caching) → UI startup faster
4. Complete US2 (Protected Paths) → Safety feature
5. Complete US3 (FSTools) → Maintainability improvement
6. Complete Polish → Quality gates

### Parallel Team Strategy

With multiple developers after Phase 2:

- Developer A: User Story 4 (storage-dbus changes)
- Developer B: User Story 1 (storage-ui client changes)
- Developer C: User Story 2 (storage-service protection)

---

## Summary

| Metric | Count |
|--------|-------|
| Total Tasks | 41 |
| Setup Tasks | 4 |
| Foundational Tasks | 1 |
| US4 (Layer 2 Caching) | 7 |
| US1 (Layer 1 Caching) | 9 |
| US2 (Protected Paths) | 7 |
| US3 (FSTools) | 7 |
| Polish Tasks | 6 |
| Parallel Opportunities | 10 tasks marked [P] |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Constitution requires: `cargo test`, `cargo clippy`, `cargo fmt` all pass before merge

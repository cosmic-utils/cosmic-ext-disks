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

- [x] T001 Verify workspace builds cleanly with `cargo build --workspace`
- [x] T002 Run existing tests with `cargo test --workspace --all-features` to establish baseline
- [x] T003 [P] Review current D-Bus connection patterns in `storage-ui/src/client/*.rs`
- [x] T004 [P] Review current discovery function in `storage-dbus/src/disk/discovery.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared types needed by multiple user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Add `FilesystemToolInfo` struct to `storage-common/src/lib.rs` with fs_type, fs_name, command, package_hint, available fields

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 4 - Efficient Service-to-UDisks2 Communication (Priority: P1) 🎯 MVP

**Goal**: Cache D-Bus connection in DiskManager to eliminate 9+ redundant connections per operation

**Independent Test**: Measure disk enumeration time before/after; verify no new connections created after first call

### Implementation for User Story 4

- [x] T006 [US4] Add `connection: Arc<Connection>` field to DiskManager in `storage-dbus/src/disk/manager.rs`
- [x] T007 [US4] Update `DiskManager::new()` to establish and cache connection in `storage-dbus/src/disk/manager.rs`
- [x] T008 [US4] Add `pub fn connection(&self) -> &Arc<Connection>` method to DiskManager in `storage-dbus/src/disk/manager.rs`
- [x] T009 [US4] Update `get_disks_with_volumes()` signature to accept `&DiskManager` parameter in `storage-dbus/src/disk/discovery.rs`
- [x] T010 [US4] Replace `Connection::system()` with `manager.connection()` in `storage-dbus/src/disk/discovery.rs`
- [x] T011 [US4] Update all call sites in `storage-service/src/disks.rs` to pass manager reference to `get_disks_with_volumes()`
- [x] T012 [US4] Verify build with `cargo build --workspace`

**Checkpoint**: Layer 2 connection caching complete - service operations should be noticeably faster

---

## Phase 4: User Story 1 - Fast Application Startup and Responsive UI (Priority: P1)

**Goal**: Cache D-Bus connection in UI layer to eliminate multiple connections during startup

**Independent Test**: Measure app startup time before/after; verify single connection used across all clients

### Implementation for User Story 1

- [x] T013 [US1] Create `storage-ui/src/client/connection.rs` with `OnceLock<Connection>` and `shared_connection()` function
- [x] T014 [US1] Export connection module in `storage-ui/src/client/mod.rs`
- [x] T015 [P] [US1] Update `DisksClient::new()` to use `shared_connection()` in `storage-ui/src/client/disks.rs`
- [x] T016 [P] [US1] Update `FilesystemsClient::new()` to use `shared_connection()` in `storage-ui/src/client/filesystems.rs`
- [x] T017 [P] [US1] Update `PartitionsClient::new()` to use `shared_connection()` in `storage-ui/src/client/partitions.rs`
- [x] T018 [P] [US1] Update `LuksClient::new()` to use `shared_connection()` in `storage-ui/src/client/luks.rs` (if exists)
- [x] T019 [P] [US1] Update `LvmClient::new()` to use `shared_connection()` in `storage-ui/src/client/lvm.rs` (if exists)
- [x] T020 [P] [US1] Update `BtrfsClient::new()` to use `shared_connection()` in `storage-ui/src/client/btrfs.rs` (if exists)
- [x] T021 [US1] Verify build with `cargo build --workspace`

**Checkpoint**: Layer 1 connection sharing complete - app startup should be faster

---

## Phase 5: User Story 2 - Protection Against Accidental System Unmount (Priority: P1)

**Goal**: Prevent users from killing processes on critical system paths during unmount

**Independent Test**: Attempt unmount with kill_processes=true on protected path; verify error returned and displayed

### Implementation for User Story 2

- [x] T022 [US2] Create `storage-service/src/protected_paths.rs` with `PROTECTED_SYSTEM_PATHS` constant array
- [x] T023 [US2] Add `is_protected_path()` function with canonical path matching in `storage-service/src/protected_paths.rs`
- [x] T024 [US2] Add `mod protected_paths;` declaration in `storage-service/src/main.rs`
- [x] T025 [US2] Import and use `is_protected_path()` in unmount handler in `storage-service/src/filesystems.rs`
- [x] T026 [US2] Add protected path check before kill_processes logic, return error in `UnmountResult` format in `storage-service/src/filesystems.rs`
- [x] T027 [US2] Add tracing log for protected path rejection in `storage-service/src/filesystems.rs`
- [x] T028 [US2] Verify build with `cargo build --workspace`

**Checkpoint**: System path protection complete - critical paths are now protected

---

## Phase 6: User Story 3 - Centralized Filesystem Tool Detection (Priority: P2)

**Goal**: Move filesystem tool detection to service, expose via D-Bus for UI feature enablement

**Independent Test**: Query service's get_filesystem_tools() method; verify accurate reflection of installed tools

### Implementation for User Story 3

- [x] T029 [US3] Add `filesystem_tools: Vec<FilesystemToolInfo>` field to FilesystemsHandler in `storage-service/src/filesystems.rs`
- [x] T030 [US3] Implement `detect_all_filesystem_tools()` function in `storage-service/src/filesystems.rs`
- [x] T031 [US3] Update `FilesystemsHandler::new()` to call detection and populate fields in `storage-service/src/filesystems.rs`
- [x] T032 [US3] Add `get_filesystem_tools()` D-Bus method returning JSON in `storage-service/src/filesystems.rs`
- [x] T033 [US3] Add `get_filesystem_tools()` client method in `storage-ui/src/client/filesystems.rs`
- [x] T034 [US3] Mark `storage-ui/src/utils/fs_tools.rs` as deprecated with comment (if it exists)
- [x] T035 [US3] Verify build with `cargo build --workspace`

**Checkpoint**: FSTools consolidation complete - UI can query service for capabilities

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates and final verification

- [x] T036 Run `cargo fmt --all --check` and fix any formatting issues
- [x] T037 Run `cargo clippy --workspace --all-features` and fix any warnings
- [x] T038 Run `cargo test --workspace --all-features` and verify all tests pass
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

---

## APPENDIX B: Security Critical Tasks (US5 & US6)

*Added during implementation: Security audit revealed critical vulnerabilities requiring new user stories.*

### Background

Two critical security issues were discovered:
1. **Polkit Bypass (US5)**: All authorization checks validate against root (always passes) instead of actual caller
2. **User Context Loss (US6)**: Mount operations create root-owned paths/files inaccessible to users

These are **P1 CRITICAL** and should be addressed before other work.

---

## Phase 8: Foundational - Authorization Types (CRITICAL PREREQUISITE)

**Purpose**: Types needed for authorization macro and user context passthrough

**⚠️ CRITICAL**: Must complete before US5/US6

- [X] T042 Create `storage-service-macros` crate directory at `storage-service-macros/`
- [X] T043 Create `storage-service-macros/Cargo.toml` with `proc-macro = true` and dependencies (syn, quote, proc-macro2)
- [X] T044 Create `storage-service-macros/src/lib.rs` with crate structure
- [X] T045 [P] Create `CallerInfo` struct in `storage-common/src/caller.rs` with `uid: u32`, `username: Option<String>`, `sender: String` fields
- [X] T046 [P] Export `CallerInfo` from `storage-common/src/lib.rs`
- [X] T047 Add `storage-service-macros` dependency to `storage-service/Cargo.toml`
- [X] T048 Implement `#[authorized_interface]` procedural macro in `storage-service-macros/src/lib.rs` that wraps `#[zbus::interface]` with Polkit auth

**Checkpoint**: Authorization infrastructure ready

---

## Phase 9: User Story 5 - Proper Polkit Authorization (Priority: P1) 🔒 SECURITY CRITICAL

**Goal**: Fix complete security bypass - all destructive operations currently bypass authorization

**Independent Test**: Unprivileged user attempts format/delete - should see Polkit password prompt

### Implementation for User Story 5

**Filesystems (P1 - Most Critical)**

- [X] T049 [US5] Migrate `mount` method to `#[authorized_interface(action = "org.cosmic.ext.storage-service.mount")]` in `storage-service/src/filesystems.rs`
- [X] T050 [US5] Migrate `unmount` method to `#[authorized_interface]` in `storage-service/src/filesystems.rs`
- [X] T051 [US5] Migrate `format` method to `#[authorized_interface]` in `storage-service/src/filesystems.rs`
- [X] T052 [US5] Migrate `set_label` method to `#[authorized_interface]` in `storage-service/src/filesystems.rs`
- [X] T053 [US5] Migrate `take_ownership` method to `#[authorized_interface]` in `storage-service/src/filesystems.rs`
- [X] T054 [US5] Migrate `check` method to `#[authorized_interface]` in `storage-service/src/filesystems.rs`
- [X] T055 [US5] Migrate `repair` method to `#[authorized_interface]` in `storage-service/src/filesystems.rs`

**Partitions (P1)**

- [ ] T056 [P] [US5] Migrate `create_partition` method to `#[authorized_interface]` in `storage-service/src/partitions.rs`
- [ ] T057 [P] [US5] Migrate `delete_partition` method to `#[authorized_interface]` in `storage-service/src/partitions.rs`
- [ ] T058 [P] [US5] Migrate `resize_partition` method to `#[authorized_interface]` in `storage-service/src/partitions.rs`
- [ ] T059 [P] [US5] Migrate `set_partition_type` method to `#[authorized_interface]` in `storage-service/src/partitions.rs`
- [ ] T060 [P] [US5] Migrate `set_partition_name` method to `#[authorized_interface]` in `storage-service/src/partitions.rs`
- [ ] T061 [P] [US5] Migrate `set_partition_flags` method to `#[authorized_interface]` in `storage-service/src/partitions.rs`

**LUKS (P1)**

- [ ] T062 [P] [US5] Migrate `unlock_luks` method to `#[authorized_interface]` in `storage-service/src/luks.rs`
- [ ] T063 [P] [US5] Migrate `lock_luks` method to `#[authorized_interface]` in `storage-service/src/luks.rs`
- [ ] T064 [P] [US5] Migrate `format_luks` method to `#[authorized_interface]` in `storage-service/src/luks.rs`
- [ ] T065 [P] [US5] Migrate `change_passphrase` method to `#[authorized_interface]` in `storage-service/src/luks.rs`

**Btrfs (P2)**

- [ ] T066 [P] [US5] Migrate `create_subvolume` method to `#[authorized_interface]` in `storage-service/src/btrfs.rs`
- [ ] T067 [P] [US5] Migrate `delete_subvolume` method to `#[authorized_interface]` in `storage-service/src/btrfs.rs`
- [ ] T068 [P] [US5] Migrate `create_snapshot` method to `#[authorized_interface]` in `storage-service/src/btrfs.rs`

**Zram (P2)**

- [ ] T069 [P] [US5] Migrate `create_zram` method to `#[authorized_interface]` in `storage-service/src/zram.rs`
- [ ] T070 [P] [US5] Migrate `destroy_zram` method to `#[authorized_interface]` in `storage-service/src/zram.rs`

**Disks (P2)**

- [ ] T071 [P] [US5] Migrate `eject` method to `#[authorized_interface]` in `storage-service/src/disks.rs`
- [ ] T072 [P] [US5] Migrate `power_off` method to `#[authorized_interface]` in `storage-service/src/disks.rs`

**Cleanup**

- [ ] T073 [US5] Deprecate or remove `check_polkit_auth()` function in `storage-service/src/auth.rs`
- [ ] T074 [US5] Add deprecation warning to functions using `connection.unique_name()` for caller identity in `storage-service/src/auth.rs`

**Checkpoint**: All destructive operations now properly check Polkit against actual caller

---

## Phase 10: User Story 6 - User-Owned Mount Points (Priority: P1)

**Goal**: Mount operations create user-accessible paths (`/run/media/<username>/`) with user-owned files

**Independent Test**: Non-root user mounts USB - verify path under their username and file ownership

### Implementation for User Story 6

- [ ] T075 [US6] Add `caller_uid: Option<u32>` parameter to `mount_filesystem()` in `storage-dbus/src/filesystem/mount.rs`
- [ ] T076 [US6] Implement `get_username_from_uid()` helper using `libc::getpwuid` in `storage-dbus/src/filesystem/mount.rs`
- [ ] T077 [US6] Add `as-user` UDisks2 option with resolved username when `caller_uid` is provided in `storage-dbus/src/filesystem/mount.rs`
- [ ] T078 [US6] Add `uid` UDisks2 option for FAT/NTFS/exFAT file ownership in `storage-dbus/src/filesystem/mount.rs`
- [ ] T079 [US6] Update `mount` method in `storage-service/src/filesystems.rs` to pass `caller.uid` to `mount_filesystem()`

**Checkpoint**: Mount operations now create user-accessible mount points

---

## Phase 11: Security Polish & Validation

**Purpose**: Verify security fixes work correctly

- [ ] T080 Run `cargo build --workspace` and verify compilation
- [ ] T081 Run `cargo clippy --workspace --all-features` and fix any warnings
- [ ] T082 Run `cargo test --workspace --all-features` and verify tests pass
- [ ] T083 Manual test: Unprivileged user attempts format operation - verify Polkit prompt appears
- [ ] T084 Manual test: Cancel Polkit prompt - verify operation is denied
- [ ] T085 Manual test: Non-root user mounts USB drive - verify mount path is `/run/media/<username>/`
- [ ] T086 Manual test: Non-root user accesses mounted FAT/NTFS files - verify read/write works
- [ ] T087 Verify no code uses `connection.unique_name()` for caller identification

---

## Updated Dependencies (Including Security)

```
Phase 8 (Auth Types) ─────► Phase 9 (US5 Polkit) ─────► Phase 10 (US6 User Mount)
                                    │
                                    ▼
                           Phase 11 (Security Polish)
```

**Critical Path for Security Fix**:
1. Phase 8 (Auth Types) - MUST complete first
2. Phase 9 (US5 Polkit) - CRITICAL SECURITY
3. Phase 10 (US6 User Mount) - Dependent on US5 for CallerInfo
4. Phase 11 (Security Polish) - Validation

---

## Updated Summary

| Metric | Original | Security Addition | Total |
|--------|----------|-------------------|-------|
| Total Tasks | 41 | 46 | 87 |
| Setup/Foundational | 5 | 7 | 12 |
| US4 (Layer 2 Caching) | 7 | - | 7 |
| US1 (Layer 1 Caching) | 9 | - | 9 |
| US2 (Protected Paths) | 7 | - | 7 |
| US3 (FSTools) | 7 | - | 7 |
| US5 (Polkit Authorization) | - | 26 | 26 |
| US6 (User Mount) | - | 5 | 5 |
| Polish Tasks | 6 | 8 | 14 |
| Parallel Opportunities | 10 | 18 | 28 |

---

## Recommended Implementation Order

### Immediate (Security Critical)

1. **Phase 8**: Auth Types (T042-T048)
2. **Phase 9**: US5 Polkit (T049-T074) - Deploy immediately after completion
3. **Phase 10**: US6 User Mount (T075-T079)
4. **Phase 11**: Security Polish (T080-T087)

### Already Complete (Performance & Safety)

- Phase 1-7: Connection caching, protected paths, FSTools

---

## Security Task Notes

- US5 tasks are SECURITY CRITICAL - any user can currently perform any operation
- The `#[authorized_interface]` macro replaces manual `check_polkit_auth()` calls
- All methods must use `header.sender()` to get actual caller, NOT `connection.unique_name()`
- US6 depends on US5 because it needs `CallerInfo.uid` from authorized methods
- Test with unprivileged user to verify Polkit prompts appear

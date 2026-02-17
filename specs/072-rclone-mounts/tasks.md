# Tasks: RClone Mount Management

**Input**: Design documents from `/specs/072-rclone-mounts/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/rclone-api.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add polkit actions and update crate dependencies

- [x] T001 Add 4 RClone polkit actions to data/polkit-1/actions/org.cosmic.ext.storage-service.policy (rclone-read, rclone-test, rclone-mount, rclone-config)
- [x] T002 [P] Add `ini` crate dependency to storage-sys/Cargo.toml for rclone.conf parsing
- [x] T003 [P] Add `which` crate dependency to storage-sys/Cargo.toml for rclone binary detection

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data models and low-level CLI operations that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 [P] Create ConfigScope enum in storage-common/src/rclone.rs
- [x] T005 [P] Create MountStatus enum in storage-common/src/rclone.rs
- [x] T006 [P] Create MountType enum in storage-common/src/rclone.rs
- [x] T007 [P] Create RemoteConfig struct in storage-common/src/rclone.rs
- [x] T008 [P] Create NetworkMount struct in storage-common/src/rclone.rs
- [x] T009 [P] Create RemoteConfigList struct in storage-common/src/rclone.rs
- [x] T010 Create TestResult struct in storage-common/src/rclone.rs (for test_remote return)
- [x] T011 Create MountStatusResult struct in storage-common/src/rclone.rs (for get_mount_status return)
- [x] T012 Add `pub mod rclone;` and re-exports to storage-common/src/lib.rs
- [x] T013 [P] Add RCloneError variants to storage-sys/src/error.rs
- [x] T014 Create rclone module stub in storage-sys/src/lib.rs
- [x] T015 Create RCloneCli struct with `find_rclone_binary()` in storage-sys/src/rclone.rs
- [x] T016 Implement `list_remotes()` CLI wrapper in storage-sys/src/rclone.rs
- [x] T017 Implement `get_config_path()` for user/system scope in storage-sys/src/rclone.rs
- [x] T018 Implement `read_config()` parser in storage-sys/src/rclone.rs
- [x] T019 Implement `get_mount_point()` for user/system scope in storage-sys/src/rclone.rs
- [x] T020 Implement `is_mounted()` using mountpoint -q in storage-sys/src/rclone.rs
- [x] T021 [P] Create RcloneInterface D-Bus proxy trait in storage-ui/src/client/rclone.rs
- [x] T022 Create RcloneClient struct with all D-Bus methods in storage-ui/src/client/rclone.rs
- [x] T023 Add `pub mod rclone;` and `pub use RcloneClient` to storage-ui/src/client/mod.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - View Network Mounts (Priority: P1)

**Goal**: Display all configured RClone remotes in a "Network" section of the sidebar with status and scope indicator

**Independent Test**: Configure an RClone remote externally, open the application, verify the mount appears under the Network section with correct status and scope badge

### Implementation for User Story 1

- [x] T024 [US1] Create RcloneHandler struct in storage-service/src/rclone.rs
- [x] T025 [US1] Implement `list_remotes` D-Bus method in storage-service/src/rclone.rs with `rclone-read` polkit action
- [x] T026 [US1] Implement `get_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-read` polkit action
- [x] T027 [US1] Implement `get_mount_status` D-Bus method in storage-service/src/rclone.rs with `rclone-read` polkit action
- [x] T028 [US1] Add `mount_changed` signal definition to RcloneHandler in storage-service/src/rclone.rs
- [x] T029 [US1] Implement `supported_remote_types` property in storage-service/src/rclone.rs
- [x] T030 [US1] Register RcloneHandler at /org/cosmic/ext/StorageService/rclone in storage-service/src/main.rs
- [x] T031 [US1] Add "rclone" to supported_features in storage-service/src/service.rs
- [ ] T032 [P] [US1] Create NetworkMountItem component in storage-ui/src/components/network_mount_item.rs (displays name, status, scope badge)
- [ ] T033 [US1] Create NetworkSection sidebar component in storage-ui/src/sidebar/network_section.rs
- [ ] T034 [US1] Connect NetworkSection to RcloneClient in storage-ui
- [ ] T035 [US1] Add Network section to main sidebar in storage-ui

**Checkpoint**: User Story 1 complete - can view all RClone remotes with status and scope indicator

---

## Phase 4: User Story 2 - Control Mount Daemon (Priority: P2)

**Goal**: Allow users to start, stop, and restart individual RClone mounts

**Independent Test**: Select a configured remote, verify start/stop/restart actions correctly change the mount state

### Implementation for User Story 2

- [ ] T036 [US2] Implement `mount()` CLI wrapper using `rclone mount --daemon` in storage-sys/src/rclone.rs
- [ ] T037 [US2] Implement `unmount()` using `fusermount -u` in storage-sys/src/rclone.rs
- [ ] T038 [US2] Implement `mount` D-Bus method in storage-service/src/rclone.rs (conditional polkit for system scope)
- [ ] T039 [US2] Implement `unmount` D-Bus method in storage-service/src/rclone.rs (conditional polkit for system scope)
- [ ] T040 [US2] Add `mount()` and `unmount()` methods to RcloneClient in storage-ui/src/client/rclone.rs
- [ ] T041 [US2] Add mount/unmount buttons to NetworkMountItem in storage-ui/src/components/network_mount_item.rs
- [ ] T042 [US2] Wire mount/unmount buttons to RcloneClient calls in storage-ui
- [ ] T043 [US2] Add loading indicator during mount operations in storage-ui
- [ ] T044 [US2] Implement restart as unmount+mount sequence in storage-ui or storage-service

**Checkpoint**: User Story 2 complete - can control mount state from UI

---

## Phase 5: User Story 3 - Test Remote Configuration (Priority: P3)

**Goal**: Validate RClone remote configuration before mounting

**Independent Test**: Select a remote, invoke test action, verify result matches actual configuration validity

### Implementation for User Story 3

- [ ] T045 [US3] Implement `test_remote()` CLI wrapper using `rclone ls --max-depth 1` in storage-sys/src/rclone.rs
- [ ] T046 [US3] Implement `test_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-test` polkit action
- [ ] T047 [US3] Add `test_remote()` method to RcloneClient in storage-ui/src/client/rclone.rs
- [ ] T048 [US3] Add "Test Configuration" button to NetworkMountItem in storage-ui/src/components/network_mount_item.rs
- [ ] T049 [US3] Wire test button to RcloneClient call in storage-ui
- [ ] T050 [US3] Display test result dialog (success/failure with message) in storage-ui

**Checkpoint**: User Story 3 complete - can test remote connectivity

---

## Phase 6: User Story 4 - Manage Remote Configuration (Priority: P4)

**Goal**: Create, edit, and delete RClone remote configurations through the UI

**Independent Test**: Create a new remote configuration through the interface, verify it appears in the app and in rclone.conf

### Implementation for User Story 4

- [ ] T051 [US4] Implement `write_config()` to update rclone.conf in storage-sys/src/rclone.rs
- [ ] T052 [US4] Implement `create_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-config` polkit action
- [ ] T053 [US4] Implement `update_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-config` polkit action
- [ ] T054 [US4] Implement `delete_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-config` polkit action
- [ ] T055 [US4] Add CRUD methods to RcloneClient in storage-ui/src/client/rclone.rs
- [ ] T056 [P] [US4] Create RemoteConfigDialog component in storage-ui/src/components/remote_config_dialog.rs
- [ ] T057 [US4] Add "Add Remote" button to NetworkSection in storage-ui/src/sidebar/network_section.rs
- [ ] T058 [US4] Add "Edit" and "Delete" context menu to NetworkMountItem in storage-ui
- [ ] T059 [US4] Wire CRUD operations to RcloneClient calls in storage-ui
- [ ] T060 [US4] Add confirmation dialog for delete operation in storage-ui

**Checkpoint**: User Story 4 complete - full CRUD for remote configurations

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Error handling, edge cases, and finalization

- [ ] T061 [P] Add error handling for missing rclone binary in storage-sys/src/rclone.rs
- [ ] T062 [P] Add error handling for malformed rclone.conf in storage-sys/src/rclone.rs
- [ ] T063 [P] Add user-friendly error messages in storage-ui for common failures
- [ ] T064 Handle concurrent mount/unmount requests gracefully in storage-service/src/rclone.rs
- [ ] T065 Add empty state message when no remotes configured in storage-ui/src/sidebar/network_section.rs
- [ ] T066 Run `cargo clippy --workspace --all-features` and fix warnings
- [ ] T067 Run `cargo fmt --all --check` and fix issues
- [ ] T068 Run `cargo test --workspace --all-features` and ensure all tests pass
- [ ] T069 Validate quickstart.md scenarios manually

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can proceed in parallel (if staffed) or sequentially (P1 → P2 → P3 → P4)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational - Uses D-Bus infrastructure from US1
- **User Story 3 (P3)**: Can start after Foundational - Independent, can parallel with US2
- **User Story 4 (P4)**: Can start after Foundational - Independent, can parallel with US2/US3

### Parallel Opportunities

**Phase 1 (all can run in parallel)**:
```bash
# These tasks touch different files
Task T001: Polkit policy
Task T002: storage-sys Cargo.toml
Task T003: storage-sys Cargo.toml
```

**Phase 2 (models and client can run in parallel)**:
```bash
# All entity creation tasks are independent
Task T004-T011: Create data model structs
Task T013: Error variants (separate file)
Task T021: D-Bus proxy trait (separate file from client)
```

**User Stories can run in parallel after Phase 2**:
```bash
# Different developers can work on different stories
Developer A: US1 (T024-T035)
Developer B: US2 (T036-T044)
Developer C: US3 (T045-T050)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T023)
3. Complete Phase 3: User Story 1 (T024-T035)
4. **STOP and VALIDATE**: Test viewing remotes in UI
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → View remotes (MVP!)
3. Add User Story 2 → Control mounts
4. Add User Story 3 → Test configurations
5. Add User Story 4 → Manage configurations (Full feature)
6. Polish → Production ready

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| Phase 1: Setup | T001-T003 (3) | Polkit actions, dependencies |
| Phase 2: Foundational | T004-T023 (20) | Data models, CLI wrappers, D-Bus client |
| Phase 3: US1 (P1) | T024-T035 (12) | View Network Mounts |
| Phase 4: US2 (P2) | T036-T044 (9) | Control Mount Daemon |
| Phase 5: US3 (P3) | T045-T050 (6) | Test Configuration |
| Phase 6: US4 (P4) | T051-T060 (10) | Manage Configuration |
| Phase 7: Polish | T061-T069 (9) | Error handling, validation |
| **Total** | **69 tasks** | |

---

## Notes

- [P] tasks touch different files and have no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently

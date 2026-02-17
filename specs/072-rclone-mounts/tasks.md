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

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - View Network Mounts (Priority: P1)

**Goal**: Display all configured RClone remotes in a "Network" section of the sidebar with status and scope indicator

**Independent Test**: Configure an RClone remote externally, open the application, verify the mount appears under the Network section with correct status and scope badge

### Implementation for User Story 1

- [x] T021 [US1] Create RcloneHandler struct in storage-service/src/rclone.rs
- [x] T022 [US1] Implement `list_remotes` D-Bus method in storage-service/src/rclone.rs with `rclone-read` polkit action
- [x] T023 [US1] Implement `get_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-read` polkit action
- [x] T024 [US1] Implement `get_mount_status` D-Bus method in storage-service/src/rclone.rs with `rclone-read` polkit action
- [x] T025 [US1] Add `mount_changed` signal definition to RcloneHandler in storage-service/src/rclone.rs
- [x] T026 [US1] Implement `supported_remote_types` property in storage-service/src/rclone.rs
- [x] T027 [US1] Register RcloneHandler at /org/cosmic/ext/StorageService/rclone in storage-service/src/main.rs
- [x] T028 [US1] Add "rclone" to supported_features in storage-service/src/service.rs
- [x] T029 [P] [US1] Create NetworkMountItem component in storage-ui/src/ui/network/view.rs (displays name, status, scope badge)
- [x] T030 [US1] Create NetworkSection sidebar component in storage-ui/src/ui/network/view.rs
- [x] T031 [US1] Connect NetworkSection to D-Bus list_remotes in storage-ui
- [x] T032 [US1] Add Network section to main sidebar in storage-ui

**Checkpoint**: User Story 1 complete - can view all RClone remotes with status and scope indicator

---

## Phase 4: User Story 2 - Control Mount Daemon (Priority: P2)

**Goal**: Allow users to start, stop, and restart individual RClone mounts

**Independent Test**: Select a configured remote, verify start/stop/restart actions correctly change the mount state

### Implementation for User Story 2

- [x] T033 [US2] Implement `mount()` CLI wrapper using `rclone mount --daemon` in storage-sys/src/rclone.rs
- [x] T034 [US2] Implement `unmount()` using `fusermount -u` in storage-sys/src/rclone.rs
- [x] T035 [US2] Implement `mount` D-Bus method in storage-service/src/rclone.rs (conditional polkit for system scope)
- [x] T036 [US2] Implement `unmount` D-Bus method in storage-service/src/rclone.rs (conditional polkit for system scope)
- [x] T037 [US2] Add mount/unmount buttons to NetworkMountItem in storage-ui/src/ui/network/view.rs
- [x] T038 [US2] Wire mount/unmount buttons to D-Bus calls in storage-ui
- [x] T039 [US2] Add loading indicator during mount operations in storage-ui
- [x] T040 [US2] Implement restart as unmount+mount sequence in storage-ui or storage-service

**Checkpoint**: User Story 2 complete - can control mount state from UI

---

## Phase 5: User Story 3 - Test Remote Configuration (Priority: P3)

**Goal**: Validate RClone remote configuration before mounting

**Independent Test**: Select a remote, invoke test action, verify result matches actual configuration validity

### Implementation for User Story 3

- [x] T041 [US3] Implement `test_remote()` CLI wrapper using `rclone ls --max-depth 1` in storage-sys/src/rclone.rs
- [x] T042 [US3] Implement `test_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-test` polkit action
- [x] T043 [US3] Add "Test Configuration" button to NetworkMountItem in storage-ui/src/ui/network/view.rs
- [x] T044 [US3] Wire test button to D-Bus call in storage-ui
- [ ] T045 [US3] Display test result dialog (success/failure with message) in storage-ui

**Checkpoint**: User Story 3 complete - can test remote connectivity

---

## Phase 6: User Story 4 - Manage Remote Configuration (Priority: P4)

**Goal**: Create, edit, and delete RClone remote configurations through the UI

**Independent Test**: Create a new remote configuration through the interface, verify it appears in the app and in rclone.conf

### Implementation for User Story 4

- [x] T046 [US4] Implement `write_config()` to update rclone.conf in storage-sys/src/rclone.rs
- [x] T047 [US4] Implement `create_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-config` polkit action
- [x] T048 [US4] Implement `update_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-config` polkit action
- [x] T049 [US4] Implement `delete_remote` D-Bus method in storage-service/src/rclone.rs with `rclone-config` polkit action
- [ ] T050 [P] [US4] Create RemoteConfigDialog component in storage-ui/src/components/remote_config_dialog.rs
- [ ] T051 [US4] Add "Add Remote" button to NetworkSection in storage-ui/src/sidebar/network_section.rs. This should be a plus icon on the same row as the section header.
- [ ] T052 [US4] Add "Edit" and "Delete" context menu to NetworkMountItem in storage-ui
- [ ] T053 [US4] Wire CRUD operations to D-Bus calls in storage-ui
- [ ] T054 [US4] Add confirmation dialog for delete operation in storage-ui

**Checkpoint**: User Story 4 complete - full CRUD for remote configurations

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Error handling, edge cases, and finalization

- [ ] T055 [P] Add error handling for missing rclone binary in storage-sys/src/rclone.rs
- [ ] T056 [P] Add error handling for malformed rclone.conf in storage-sys/src/rclone.rs
- [ ] T057 [P] Add user-friendly error messages in storage-ui for common failures
- [ ] T058 Handle concurrent mount/unmount requests gracefully in storage-service/src/rclone.rs
- [x] T059 Add empty state message when no remotes configured in storage-ui/src/sidebar/network_section.rs
- [x] T060 Run `cargo clippy --workspace --all-features` and fix warnings
- [x] T061 Run `cargo fmt --all --check` and fix issues
- [x] T062 Run `cargo test --workspace --all-features` and ensure all tests pass
- [ ] T063 Validate quickstart.md scenarios manually

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

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T020)
3. Complete Phase 3: User Story 1 (T021-T032)
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

| Phase | Tasks | Completed | Description |
|-------|-------|-----------|-------------|
| Phase 1: Setup | T001-T003 (3) | 3 | Polkit actions, dependencies |
| Phase 2: Foundational | T004-T020 (17) | 17 | Data models, CLI wrappers |
| Phase 3: US1 (P1) | T021-T032 (12) | 12 | View Network Mounts |
| Phase 4: US2 (P2) | T033-T040 (8) | 8 | Control Mount Daemon |
| Phase 5: US3 (P3) | T041-T045 (5) | 4 | Test Configuration |
| Phase 6: US4 (P4) | T046-T054 (9) | 4 | Manage Configuration |
| Phase 7: Polish | T055-T063 (9) | 4 | Error handling, validation |
| **Total** | **63 tasks** | **52** | **82% complete** |

---

## Notes

- [P] tasks touch different files and have no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently

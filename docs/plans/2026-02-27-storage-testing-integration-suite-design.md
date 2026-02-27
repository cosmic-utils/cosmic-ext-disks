# Storage-Testing Integration Suite (Spec-Only) Design

**Date:** 2026-02-27  
**Status:** Spec-only (no implementation in this document)  
**Scope:** Remaining architecture and test specification work for replacing smoke tests with full integration coverage in `storage-testing/tests`.

---

## 1) Guardrails and Decisions

### 1.1 Fixed decisions from this spec

1. `storage-testing/tests` becomes a real integration suite (smoke tests removed from default path).
2. All tests must declare required lab spec metadata.
3. Harness runs all tests by default.
4. Parallelism is by required lab spec group.
5. Assertions must use `storage-contracts::client` D-Bus clients.

### 1.2 No cross-binary subprocess rule

`harness` must **not** execute `lab` via external command calls (`cargo run ... lab ...`, shelling into another binary, etc.).

Instead:
- Shared orchestration logic is implemented in library modules.
- Both `src/bin/lab.rs` and `src/bin/harness.rs` call the same in-process APIs.

This is mandatory for deterministic behavior, testability, and avoiding binary-to-binary coupling.

---

## 2) Required Internal Architecture

## 2.1 Shared orchestration modules

- `storage-testing/src/lab_orchestrator.rs`
   - canonical APIs for create/prepare/attach/mount/unmount/detach/destroy/cleanup
   - no clap or stdout formatting dependencies
- `storage-testing/src/harness_orchestrator.rs`
   - test discovery, grouping, parallel scheduler, result aggregation
- `storage-testing/src/test_registry.rs`
   - typed test metadata and registration catalog
- existing support modules remain (`spec`, `ledger`, `runtime`, `artifacts`, `errors`, `cmd`)

## 2.2 Binary boundaries

- `src/bin/lab.rs`
   - parse CLI args
   - map args to `lab_orchestrator` API call
   - render result
- `src/bin/harness.rs`
   - parse CLI args
   - call `harness_orchestrator` which uses `lab_orchestrator` directly
   - render summary and exit code

No orchestration logic is duplicated in binaries.

---

## 3) Test Contract

Define a registry trait:

- `id() -> &'static str`
- `suite() -> &'static str` (e.g. `disk`, `filesystem`, `partition`, `luks`, `logical`, `image`, `rclone`)
- `required_spec() -> &'static str` (mandatory; e.g. `2disk`, `3disk`)
- `exclusive() -> bool` (default false)
- `run(ctx: &HarnessContext) -> Result<()>`

`HarnessContext` includes:
- artifact writers
- timeout policy
- typed clients from `storage-contracts::client`
- spec-scoped lab state handles

---

## 4) Execution Model

1. Discover tests from static registry.
2. Apply selectors (if any); default is all tests.
3. Group selected tests by `required_spec()`.
4. For each group:
    - in-process lab setup via `lab_orchestrator`
    - execute tests (sequential by default)
    - always run teardown
5. Run groups in parallel with bounded concurrency (`max_parallel_groups`).
6. `exclusive()` tests run in isolated lane within group (no overlap).

Failure policy:
- setup failure: mark group failed/skipped with explicit reason
- test failure: mark failed but continue per-group policy
- teardown failure: record separately and keep original test failure cause

---

## 5) Full Functional Test Catalog (Required Coverage)

The following test IDs are the minimum required set for "all functionality" coverage through current client surface.

## 5.0 Planned `tests/` folder structure

```text
storage-testing/tests/
   common/
      mod.rs
      assertions.rs
      fixtures.rs
      registration.rs
   disk/
      list_disks.rs
      list_volumes_schema_integrity.rs
      get_disk_info_for_known_device.rs
   filesystem/
      unmount_roundtrip.rs
      check_readonly_path.rs
      usage_scan_basic.rs
      mount_options_roundtrip.rs
   partition/
      list_partitions_expected_from_spec.rs
      create_delete_roundtrip.rs
      set_name_type_flags_roundtrip.rs
   luks/
      unlock_lock_roundtrip.rs
      options_roundtrip.rs
   btrfs/
      subvolume_create_delete_roundtrip.rs
      snapshot_create_delete_roundtrip.rs
      default_subvolume_set_get.rs
   logical/
      list_entities_schema_integrity.rs
      lvm_create_resize_delete_lv.rs
      mdraid_create_start_stop_delete.rs
      btrfs_add_remove_member.rs
   image/
      loop_setup_valid_image.rs
      backup_restore_drive_smoke.rs
   rclone/
      list_remotes_basic.rs
      mount_status_query.rs
```

`harness_smoke.rs` and `lab_smoke.rs` are removed from default execution.

## 5.1 Disk suite

- `disk.list_disks.non_empty_or_empty_ok` (`2disk`)
   - file: `storage-testing/tests/disk/list_disks.rs`
   - call `DisksClient::list_disks`
   - assert parse success and stable schema fields
- `disk.list_volumes.schema_integrity` (`2disk`)
   - file: `storage-testing/tests/disk/list_volumes_schema_integrity.rs`
   - call `DisksClient::list_volumes`
   - assert parent-child references are internally consistent
- `disk.get_disk_info.for_known_device` (`2disk`)
   - file: `storage-testing/tests/disk/get_disk_info_for_known_device.rs`
   - pick known disk from spec-attached devices
   - assert model fields (device path, size, transport optionality)

## 5.2 Filesystem suite

- `filesystem.mount.unmount.roundtrip` (`2disk`, exclusive)
   - file: `storage-testing/tests/filesystem/unmount_roundtrip.rs`
   - create/mount filesystem target from spec
   - assert mounted state then unmounted state via service queries
- `filesystem.check.readonly_path` (`2disk`)
   - file: `storage-testing/tests/filesystem/check_readonly_path.rs`
   - call check without repair
   - assert command completion and structured response
- `filesystem.usage_scan.basic` (`2disk`)
   - file: `storage-testing/tests/filesystem/usage_scan_basic.rs`
   - call usage scan on known mount point
   - assert categories returned and bytes are non-negative
- `filesystem.mount_options.read_write_roundtrip` (`2disk`)
   - file: `storage-testing/tests/filesystem/mount_options_roundtrip.rs`
   - set mount options, read back, reset defaults

## 5.3 Partition suite

- `partition.list_partitions.expected_from_spec` (`2disk`)
   - file: `storage-testing/tests/partition/list_partitions_expected_from_spec.rs`
   - verify partition table reflects spec plan
- `partition.create_delete.roundtrip` (`2disk`, exclusive)
   - file: `storage-testing/tests/partition/create_delete_roundtrip.rs`
   - create temporary partition, verify exists, delete, verify removed
- `partition.set_name_type_flags.roundtrip` (`2disk`, exclusive)
   - file: `storage-testing/tests/partition/set_name_type_flags_roundtrip.rs`
   - set GPT name/type/flags where applicable and verify no error

## 5.4 LUKS suite

- `luks.unlock_lock.roundtrip` (`2disk`, exclusive)
   - file: `storage-testing/tests/luks/unlock_lock_roundtrip.rs`
   - use preconfigured encrypted member from spec fixture
   - unlock then lock via service path
- `luks.options.read_write_roundtrip` (`2disk`)
   - file: `storage-testing/tests/luks/options_roundtrip.rs`
   - set/get/default encryption options

## 5.5 Btrfs suite

- `btrfs.subvolume.create_delete.roundtrip` (`2disk`, exclusive)
   - file: `storage-testing/tests/btrfs/subvolume_create_delete_roundtrip.rs`
   - create subvolume, list confirms, delete, list confirms removal
- `btrfs.snapshot.create_delete.roundtrip` (`2disk`, exclusive)
   - file: `storage-testing/tests/btrfs/snapshot_create_delete_roundtrip.rs`
   - create snapshot from source subvolume, assert presence, cleanup
- `btrfs.default_subvolume.set_get` (`2disk`)
   - file: `storage-testing/tests/btrfs/default_subvolume_set_get.rs`
   - set default then get default and verify expected id/path

## 5.6 Logical suite (LVM/MDRAID/Btrfs logical)

- `logical.list_entities.schema_integrity` (`3disk`)
   - file: `storage-testing/tests/logical/list_entities_schema_integrity.rs`
   - assert every returned entity has consistent metadata
- `logical.lvm.create_resize_delete_lv` (`3disk`, exclusive)
   - file: `storage-testing/tests/logical/lvm_create_resize_delete_lv.rs`
   - create VG/LV (or use fixture VG), resize LV, delete LV
- `logical.mdraid.create_start_stop_delete` (`3disk`, exclusive)
   - file: `storage-testing/tests/logical/mdraid_create_start_stop_delete.rs`
   - create test array, start/stop lifecycle, delete array
- `logical.btrfs.add_remove_member` (`3disk`, exclusive)
   - file: `storage-testing/tests/logical/btrfs_add_remove_member.rs`
   - add member device then remove member and verify state transitions

## 5.7 Image suite

- `image.loop_setup.valid_image` (`2disk`)
   - file: `storage-testing/tests/image/loop_setup_valid_image.rs`
   - call loop setup on controlled image file and verify loop path
- `image.backup_restore.drive_smoke` (`2disk`, exclusive)
   - file: `storage-testing/tests/image/backup_restore_drive_smoke.rs`
   - start backup op, poll status, verify completion event
   - run restore on disposable target and verify completion event

## 5.8 Rclone suite (feature-gated)

- `rclone.list_remotes.basic` (`2disk`)
   - file: `storage-testing/tests/rclone/list_remotes_basic.rs`
   - if runtime has provider config, assert list call semantics
- `rclone.mount_status.query` (`2disk`)
   - file: `storage-testing/tests/rclone/mount_status_query.rs`
   - query mount status for configured test remote

If rclone prerequisites are unavailable in CI, mark suite as conditional skip with explicit reason.

---

## 6) CI Behavior for Remaining Work

CI harness run must:
- run default all-tests path
- emit per-suite and per-test summary
- upload artifacts on failure (`index.json`, group logs, test logs, service logs)

---

## 7) Non-Goals

- UI screenshot/snapshot testing
- backend-direct assertions that bypass D-Bus clients
- unbounded parallel scheduler

---

## 8) Acceptance Criteria

1. No smoke placeholders remain in default `storage-testing/tests` path.
2. Every integration test declares a required lab spec.
3. Harness executes all tests by default.
4. Group orchestration uses direct in-process `lab_orchestrator` API calls.
5. Coverage includes disk/filesystem/partition/luks/btrfs/logical/image (and conditional rclone).
6. CI publishes failure artifacts for triage.

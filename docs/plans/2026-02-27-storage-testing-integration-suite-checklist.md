# Storage-Testing Integration Suite Checklist (Spec-Only, Action-Level)

**Date:** 2026-02-27  
**Purpose:** Execution-ready, high-LOD checklist for completing remaining integration-suite work.

---

## A. Preflight

- [ ] Run baseline `just check` and record timestamp.
- [ ] Confirm uncommitted scope is limited to `storage-testing` integration framework and docs.
- [ ] Confirm target replacement files currently include `storage-testing/tests/harness_smoke.rs` and `storage-testing/tests/lab_smoke.rs`.

## B. Shared Orchestrator Refactor (Mandatory)

## B1. Extract lab execution logic into shared module

- [ ] Create `storage-testing/src/lab_orchestrator.rs`.
- [ ] Move image lifecycle execution from clap command handlers into pure methods:
	- [ ] `create`
	- [ ] `prepare`
	- [ ] `attach`
	- [ ] `mount`
	- [ ] `unmount`
	- [ ] `detach`
	- [ ] `destroy`
	- [ ] `cleanup`
- [ ] Keep method inputs/outputs typed (no clap types in API signatures).

## B2. Enforce no cross-binary subprocess calls

- [ ] Verify `harness` does not invoke `lab` via `cargo run`, shell, or process exec.
- [ ] Wire `harness` to call `lab_orchestrator` methods directly.
- [ ] Add regression unit test asserting no `lab` command invocation path in harness planner/executor.

## C. Test Contract and Registry

- [ ] Create `storage-testing/src/test_registry.rs`.
- [ ] Define `HarnessTest` trait with required methods:
	- [ ] `id`
	- [ ] `suite`
	- [ ] `required_spec`
	- [ ] `exclusive`
	- [ ] `run`
- [ ] Add registry API:
	- [ ] `all_tests`
	- [ ] `filter_tests`
	- [ ] `group_by_spec`
	- [ ] `group_by_suite`
- [ ] Add deterministic ordering test for `group_by_spec` output.
- [ ] Add filtering tests for suite/id selectors.

## D. Harness Scheduler and Group Lifecycle

- [ ] Add `storage-testing/src/harness_orchestrator.rs`.
- [ ] Implement default run-all selection when no filter flags are provided.
- [ ] Implement bounded group parallelism (`max_parallel_groups`).
- [ ] Implement in-group sequential execution policy by default.
- [ ] Implement `exclusive()` isolated lane behavior.
- [ ] Implement per-group setup using direct `lab_orchestrator` calls:
	- [ ] create
	- [ ] prepare
	- [ ] attach
	- [ ] mount
- [ ] Implement per-group teardown (always attempt):
	- [ ] unmount
	- [ ] detach
	- [ ] destroy
- [ ] Ensure teardown failures are reported without overwriting root test failures.

## E. Replace Smoke Tests with Full Functional Suites

## E1. Remove smoke placeholders from default path

- [ ] Remove `harness_smoke` and `lab_smoke` from default integration registry.
- [ ] If retained for diagnostics, move behind non-default opt-in suite.

## E2. Implement required real integration files

- [ ] Add shared helper folder:
	- [ ] `storage-testing/tests/common/mod.rs`
	- [ ] `storage-testing/tests/common/assertions.rs`
	- [ ] `storage-testing/tests/common/fixtures.rs`
	- [ ] `storage-testing/tests/common/registration.rs`
- [ ] Add disk tests folder: `storage-testing/tests/disk/`
- [ ] Add filesystem tests folder: `storage-testing/tests/filesystem/`
- [ ] Add partition tests folder: `storage-testing/tests/partition/`
- [ ] Add luks tests folder: `storage-testing/tests/luks/`
- [ ] Add btrfs tests folder: `storage-testing/tests/btrfs/`
- [ ] Add logical tests folder: `storage-testing/tests/logical/`
- [ ] Add image tests folder: `storage-testing/tests/image/`
- [ ] Add rclone tests folder: `storage-testing/tests/rclone/` (conditional execution allowed).

## E3. Implement concrete required test cases

### Disk suite
- [ ] `disk.list_disks.non_empty_or_empty_ok` (spec: `2disk`, file: `storage-testing/tests/disk/list_disks.rs`)
- [ ] `disk.list_volumes.schema_integrity` (spec: `2disk`, file: `storage-testing/tests/disk/list_volumes_schema_integrity.rs`)
- [ ] `disk.get_disk_info.for_known_device` (spec: `2disk`, file: `storage-testing/tests/disk/get_disk_info_for_known_device.rs`)

### Filesystem suite
- [ ] `filesystem.mount.unmount.roundtrip` (spec: `2disk`, exclusive, file: `storage-testing/tests/filesystem/unmount_roundtrip.rs`)
- [ ] `filesystem.check.readonly_path` (spec: `2disk`, file: `storage-testing/tests/filesystem/check_readonly_path.rs`)
- [ ] `filesystem.usage_scan.basic` (spec: `2disk`, file: `storage-testing/tests/filesystem/usage_scan_basic.rs`)
- [ ] `filesystem.mount_options.read_write_roundtrip` (spec: `2disk`, file: `storage-testing/tests/filesystem/mount_options_roundtrip.rs`)

### Partition suite
- [ ] `partition.list_partitions.expected_from_spec` (spec: `2disk`, file: `storage-testing/tests/partition/list_partitions_expected_from_spec.rs`)
- [ ] `partition.create_delete.roundtrip` (spec: `2disk`, exclusive, file: `storage-testing/tests/partition/create_delete_roundtrip.rs`)
- [ ] `partition.set_name_type_flags.roundtrip` (spec: `2disk`, exclusive, file: `storage-testing/tests/partition/set_name_type_flags_roundtrip.rs`)

### LUKS suite
- [ ] `luks.unlock_lock.roundtrip` (spec: `2disk`, exclusive, file: `storage-testing/tests/luks/unlock_lock_roundtrip.rs`)
- [ ] `luks.options.read_write_roundtrip` (spec: `2disk`, file: `storage-testing/tests/luks/options_roundtrip.rs`)

### Btrfs suite
- [ ] `btrfs.subvolume.create_delete.roundtrip` (spec: `2disk`, exclusive, file: `storage-testing/tests/btrfs/subvolume_create_delete_roundtrip.rs`)
- [ ] `btrfs.snapshot.create_delete.roundtrip` (spec: `2disk`, exclusive, file: `storage-testing/tests/btrfs/snapshot_create_delete_roundtrip.rs`)
- [ ] `btrfs.default_subvolume.set_get` (spec: `2disk`, file: `storage-testing/tests/btrfs/default_subvolume_set_get.rs`)

### Logical suite
- [ ] `logical.list_entities.schema_integrity` (spec: `3disk`, file: `storage-testing/tests/logical/list_entities_schema_integrity.rs`)
- [ ] `logical.lvm.create_resize_delete_lv` (spec: `3disk`, exclusive, file: `storage-testing/tests/logical/lvm_create_resize_delete_lv.rs`)
- [ ] `logical.mdraid.create_start_stop_delete` (spec: `3disk`, exclusive, file: `storage-testing/tests/logical/mdraid_create_start_stop_delete.rs`)
- [ ] `logical.btrfs.add_remove_member` (spec: `3disk`, exclusive, file: `storage-testing/tests/logical/btrfs_add_remove_member.rs`)

### Image suite
- [ ] `image.loop_setup.valid_image` (spec: `2disk`, file: `storage-testing/tests/image/loop_setup_valid_image.rs`)
- [ ] `image.backup_restore.drive_smoke` (spec: `2disk`, exclusive, file: `storage-testing/tests/image/backup_restore_drive_smoke.rs`)

### Rclone suite (conditional)
- [ ] `rclone.list_remotes.basic` (spec: `2disk`, file: `storage-testing/tests/rclone/list_remotes_basic.rs`)
- [ ] `rclone.mount_status.query` (spec: `2disk`, file: `storage-testing/tests/rclone/mount_status_query.rs`)

## E4. Per-test action template (apply to each case)

- [ ] Register test in catalog with `id`, `suite`, `required_spec`, `exclusive`.
- [ ] Implement setup preconditions within test body (no global hidden state).
- [ ] Execute operation through `storage-contracts::client` wrapper.
- [ ] Assert success path semantics and output shape.
- [ ] Assert post-condition state via follow-up service query.
- [ ] Write test-scoped artifact log entry with operation/result summary.

## F. D-Bus Path Integrity Gate

- [ ] Search `storage-testing/tests/**` to ensure no backend-direct crate calls (outside allowed setup helpers).
- [ ] Confirm primary assertions are never shell-output-only.
- [ ] Confirm every suite imports clients from `storage-contracts::client`.

## G. Reporting and Artifacts

- [ ] Add per-test result record (`pass`/`fail`/`skip`/`error`).
- [ ] Add per-group setup/test/teardown summaries.
- [ ] Add run aggregate summary with total counts and non-zero-on-failure behavior.
- [ ] Persist artifacts:
	- [ ] `index.json`
	- [ ] `group-<spec>.log`
	- [ ] `test-<id>.log`
	- [ ] `service.log`

## H. CI Completion

- [ ] Ensure harness job runs default all-tests path.
- [ ] Add artifact upload for harness outputs on failure.
- [ ] Add concise summary section in CI log output.
- [ ] Validate with one intentional failing test on a temporary branch.

## I. Verification Commands

- [ ] `cargo test -p storage-testing -- --nocapture`
- [ ] `cargo clippy -p storage-testing --all-targets`
- [ ] `cargo run -p storage-testing --bin harness -- run --runtime <runtime>`
- [ ] `just check`

## J. Completion Gate

- [ ] No default smoke-only tests remain.
- [ ] Every integration test has explicit required spec metadata.
- [ ] Harness default path executes full catalog and groups by spec.
- [ ] Harness-to-lab orchestration is in-process (no external binary command calls).
- [ ] All required suites in section E3 pass (or documented conditional skip policy for rclone).
- [ ] CI publishes actionable artifacts for failures.

## K. Verification Evidence (fill during execution)

- [ ] Baseline `just check` timestamp/result:
- [ ] `cargo test -p storage-testing -- --nocapture` timestamp/result:
- [ ] `cargo clippy -p storage-testing --all-targets` timestamp/result:
- [ ] `harness run` timestamp/result:
- [ ] CI run URL and artifact verification note:

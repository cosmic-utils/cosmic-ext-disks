# Storage Service Adapter Strip-Out Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove adapter and routing indirection from `storage-service` so handlers call `storage-udisks` directly, while preserving D-Bus behavior and existing policy checks.

**Architecture:** Replace `handler -> adapter trait -> udisks impl` with `handler -> storage-udisks` direct calls. Keep `handlers` and `policies` layers. Remove `adapters` and `routing` modules once call sites are migrated.

**Tech Stack:** Rust, zbus, storage-udisks, storage-sys, disks_btrfs, cargo/just verification.

---

### Task 1: Baseline and usage inventory

**Files:**
- Modify: none
- Inspect: `storage-service/src/main.rs`, `storage-service/src/handlers/*.rs`, `storage-service/src/adapters/udisks.rs`, `storage-service/src/routing.rs`

**Step 1: Write the failing check (inventory command)**

Run:
`rg "DiskQueryAdapter|DiskOpsAdapter|PartitionOpsAdapter|FilesystemOpsAdapter|LuksOpsAdapter|ImageOpsAdapter|AdapterRegistry|Concern" storage-service/src -n`

**Step 2: Run inventory command**

Expected: matches in handlers + `main.rs` + adapter/routing modules.

**Step 3: Save inventory note (optional local scratch)**

Map each handler method to target `storage-udisks` function currently used by adapter impl.

**Step 4: Commit**

No commit required for inventory-only task.

### Task 2: Refactor `DisksHandler` to direct `storage-udisks`

**Files:**
- Modify: `storage-service/src/handlers/disks.rs`
- Reference: `storage-service/src/adapters/udisks.rs`

**Step 1: Write failing change**

Remove adapter trait fields/constructor args before implementing replacements.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL on missing fields/calls.

**Step 3: Write minimal implementation**

- Remove `DiskQueryAdapter`/`DiskOpsAdapter` trait usage.
- Add direct calls to equivalent `storage-udisks` functions currently used by adapter impls.
- Keep domain/policy usage unchanged.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS for `DisksHandler`-related changes.

**Step 5: Commit**

```bash
git add storage-service/src/handlers/disks.rs
git commit -m "refactor(storage-service): call storage-udisks directly in disks handler"
```

### Task 3: Refactor `PartitionsHandler` to direct `storage-udisks`

**Files:**
- Modify: `storage-service/src/handlers/partitions.rs`
- Reference: `storage-service/src/adapters/udisks.rs`

**Step 1: Write failing change**

Remove `PartitionOpsAdapter` field/constructor arg before replacement.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL on missing partition ops references.

**Step 3: Write minimal implementation**

Replace trait-method calls with corresponding `storage-udisks` direct calls.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/handlers/partitions.rs
git commit -m "refactor(storage-service): call storage-udisks directly in partitions handler"
```

### Task 4: Refactor `FilesystemsHandler` to direct `storage-udisks`

**Files:**
- Modify: `storage-service/src/handlers/filesystems.rs`
- Reference: `storage-service/src/adapters/udisks.rs`

**Step 1: Write failing change**

Remove `FilesystemOpsAdapter` field/constructor arg before replacement.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL on filesystem ops references.

**Step 3: Write minimal implementation**

Switch to direct `storage-udisks` calls for filesystem operations currently routed through adapter.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/handlers/filesystems.rs
git commit -m "refactor(storage-service): call storage-udisks directly in filesystems handler"
```

### Task 5: Refactor `LuksHandler` to direct `storage-udisks`

**Files:**
- Modify: `storage-service/src/handlers/luks.rs`
- Reference: `storage-service/src/adapters/udisks.rs`

**Step 1: Write failing change**

Remove `LuksOpsAdapter` field/constructor arg before replacement.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL on luks ops references.

**Step 3: Write minimal implementation**

Replace adapter-trait calls with direct `storage-udisks` equivalents.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/handlers/luks.rs
git commit -m "refactor(storage-service): call storage-udisks directly in luks handler"
```

### Task 6: Refactor `ImageHandler` to direct `storage-udisks`

**Files:**
- Modify: `storage-service/src/handlers/image.rs`
- Reference: `storage-service/src/adapters/udisks.rs`

**Step 1: Write failing change**

Remove `ImageOpsAdapter` field/constructor arg before replacement.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL on image ops references.

**Step 3: Write minimal implementation**

Use direct `storage-udisks` calls for loop/image operations currently behind adapter.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/handlers/image.rs
git commit -m "refactor(storage-service): call storage-udisks directly in image handler"
```

### Task 7: Simplify `main.rs` and remove routing dependency

**Files:**
- Modify: `storage-service/src/main.rs`
- Modify/Delete references: `storage-service/src/routing.rs`

**Step 1: Write failing change**

Remove `AdapterRegistry` usage/imports from `main.rs` before constructor updates.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL on handler constructors/imports.

**Step 3: Write minimal implementation**

- Construct handlers with no adapter trait args.
- Remove route logging tied to registry.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/main.rs
git commit -m "refactor(storage-service): remove adapter registry wiring"
```

### Task 8: Remove adapters/routing modules

**Files:**
- Delete: `storage-service/src/adapters/mod.rs`
- Delete: `storage-service/src/adapters/udisks.rs`
- Delete: `storage-service/src/routing.rs`
- Modify: `storage-service/src/main.rs` (remove `mod adapters; mod routing;`)

**Step 1: Write failing change**

Delete modules before removing all declarations/usages.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL on unresolved module declarations/usages.

**Step 3: Write minimal implementation**

Remove stale module declarations/imports and compile errors.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 5: Commit**

```bash
git add -A storage-service/src
git commit -m "refactor(storage-service): remove adapters and routing modules"
```

### Task 9: Optional `storage-contracts` cleanup

**Files:**
- Modify: `storage-contracts/src/traits/*.rs`
- Modify: `storage-contracts/src/traits/mod.rs`
- Modify: `storage-contracts/src/lib.rs`

**Step 1: Write discovery check**

Run: `rg "DiskQueryAdapter|DiskOpsAdapter|PartitionOpsAdapter|FilesystemOpsAdapter|LuksOpsAdapter|ImageOpsAdapter" --glob '!target/**'`

**Step 2: Decide cleanup scope**

If traits are unused workspace-wide, remove exports and files; if used elsewhere, keep them.

**Step 3: Run compile checks**

Run: `cargo check --workspace`
Expected: PASS.

**Step 4: Commit**

```bash
git add storage-contracts
git commit -m "refactor(storage-contracts): remove unused adapter trait contracts"
```

### Task 10: Verification gates and final pass

**Files:**
- Modify: none expected

**Step 1: Crate-level verification**

Run:
- `cargo check -p storage-service`
- `cargo test -p storage-service --no-run`

Expected: PASS.

**Step 2: Workspace verification**

Run: `just verify`
Expected: PASS.

**Step 3: Commit (if needed)**

```bash
git add -A
git commit -m "chore: finalize adapter strip-out verification"
```

## Notes

- Keep D-Bus API surfaces unchanged.
- Preserve existing error messages as much as practical.
- Prefer small commits by concern for easier review and rollback.

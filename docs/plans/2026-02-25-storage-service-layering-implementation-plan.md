# Storage Service Layering Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor `storage-service` to explicit `transport`, `domain`, and `adapters` layers with dependency direction `transport -> domain -> adapters`, preserving runtime behavior and D-Bus compatibility.

**Architecture:** Move D-Bus-facing handlers into `transport`, move current domain modules from `service/domain` into top-level `domain`, and keep adapters in place. Perform a structure-first pass with no logic changes, then tighten boundaries and optionally rename confusing `Default*Domain` types.

**Tech Stack:** Rust, zbus D-Bus service interfaces, cargo workspace tooling, justfile verification flow.

---

### Task 1: Introduce top-level transport/domain modules

**Files:**
- Create: `storage-service/src/transport/mod.rs`
- Create: `storage-service/src/domain/mod.rs`
- Modify: `storage-service/src/main.rs`
- Modify: `storage-service/src/service.rs`

**Step 1: Write the failing test/check**

Temporarily update `main.rs` to declare `mod transport; mod domain;` without creating files.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL with module-not-found errors for `transport` and `domain`.

**Step 3: Write minimal implementation**

Create `transport/mod.rs` and `domain/mod.rs` with minimal `pub mod` declarations for all concerns (`btrfs`, `disks`, `filesystems`, `image`, `luks`, `lvm`, `partitions`, `rclone`, plus `service` under transport).

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS or only expected downstream path errors to be fixed in next tasks.

**Step 5: Commit**

```bash
git add storage-service/src/main.rs storage-service/src/service.rs storage-service/src/transport/mod.rs storage-service/src/domain/mod.rs
git commit -m "refactor(storage-service): scaffold transport and domain modules"
```

### Task 2: Move D-Bus handlers into transport

**Files:**
- Move: `storage-service/src/btrfs.rs` -> `storage-service/src/transport/btrfs.rs`
- Move: `storage-service/src/disks.rs` -> `storage-service/src/transport/disks.rs`
- Move: `storage-service/src/filesystems.rs` -> `storage-service/src/transport/filesystems.rs`
- Move: `storage-service/src/image.rs` -> `storage-service/src/transport/image.rs`
- Move: `storage-service/src/luks.rs` -> `storage-service/src/transport/luks.rs`
- Move: `storage-service/src/lvm.rs` -> `storage-service/src/transport/lvm.rs`
- Move: `storage-service/src/partitions.rs` -> `storage-service/src/transport/partitions.rs`
- Move: `storage-service/src/rclone.rs` -> `storage-service/src/transport/rclone.rs`
- Move: `storage-service/src/service.rs` -> `storage-service/src/transport/service.rs`
- Modify: `storage-service/src/main.rs`

**Step 1: Write the failing test/check**

Move a single file first (e.g., `btrfs.rs`) and do not update imports yet.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL with unresolved module/use paths.

**Step 3: Write minimal implementation**

Move remaining files and update `main.rs` imports to `use transport::{...}` equivalents.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS or only expected unresolved domain path errors to fix in Task 3.

**Step 5: Commit**

```bash
git add storage-service/src/main.rs storage-service/src/transport
git commit -m "refactor(storage-service): move dbus handlers to transport layer"
```

### Task 3: Move domain modules to top-level domain

**Files:**
- Move: `storage-service/src/service/domain/btrfs.rs` -> `storage-service/src/domain/btrfs.rs`
- Move: `storage-service/src/service/domain/disks.rs` -> `storage-service/src/domain/disks.rs`
- Move: `storage-service/src/service/domain/filesystems.rs` -> `storage-service/src/domain/filesystems.rs`
- Move: `storage-service/src/service/domain/image.rs` -> `storage-service/src/domain/image.rs`
- Move: `storage-service/src/service/domain/luks.rs` -> `storage-service/src/domain/luks.rs`
- Move: `storage-service/src/service/domain/lvm.rs` -> `storage-service/src/domain/lvm.rs`
- Move: `storage-service/src/service/domain/partitions.rs` -> `storage-service/src/domain/partitions.rs`
- Move: `storage-service/src/service/domain/rclone.rs` -> `storage-service/src/domain/rclone.rs`
- Delete: `storage-service/src/service/domain/mod.rs`
- Delete: `storage-service/src/service/` (if empty)
- Modify: `storage-service/src/transport/*.rs` (all imports currently using `crate::service::domain::...`)

**Step 1: Write the failing test/check**

Move one domain file and keep old import path in a transport module.

**Step 2: Run check to verify it fails**

Run: `cargo check -p storage-service`
Expected: FAIL with unresolved import `crate::service::domain::...`.

**Step 3: Write minimal implementation**

Update all transport imports from `crate::service::domain::...` to `crate::domain::...`.

**Step 4: Run check to verify it passes**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/domain storage-service/src/transport storage-service/src/main.rs
git commit -m "refactor(storage-service): move domain modules to top-level domain"
```

### Task 4: Enforce boundary direction and remove stale references

**Files:**
- Modify: `storage-service/src/main.rs`
- Modify: `storage-service/src/transport/*.rs`
- Modify: `storage-service/src/domain/*.rs`
- Modify: `storage-service/src/routing.rs` (if import paths changed)

**Step 1: Write the failing check**

Run a grep expecting old path references to still exist.

Run: `rg "service::domain|crate::service::domain|mod btrfs;|mod disks;|mod filesystems;|mod image;|mod luks;|mod lvm;|mod partitions;|mod rclone;" storage-service/src`
Expected: Non-empty results (before cleanup).

**Step 2: Apply minimal implementation**

Remove stale root-level module declarations and update any remaining imports to the new module graph.

**Step 3: Re-run check to verify cleanup**

Run: `rg "service::domain|crate::service::domain" storage-service/src`
Expected: No matches.

**Step 4: Compile verification**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/main.rs storage-service/src/transport storage-service/src/domain storage-service/src/routing.rs
git commit -m "refactor(storage-service): enforce transport-domain-adapter boundaries"
```

### Task 5: Full verification gates

**Files:**
- Modify: none expected

**Step 1: Run crate-local checks**

Run: `cargo test -p storage-service --no-run`
Expected: PASS.

**Step 2: Run workspace verification**

Run: `just verify`
Expected: PASS (`fmt --check`, `clippy`, `test --no-run`).

**Step 3: Commit (if verification-only changes appear)**

```bash
git add -A
git commit -m "chore(storage-service): finalize layering refactor verification"
```

### Task 6 (Optional): Rename confusing Default*Domain types

**Files:**
- Modify: `storage-service/src/domain/*.rs`
- Modify: `storage-service/src/transport/*.rs`

**Step 1: Write the failing check**

Run: `rg "Default[A-Za-z]+Domain" storage-service/src`
Expected: Existing matches.

**Step 2: Apply minimal implementation**

Rename only one concern at a time (e.g., disks) from `DefaultDisksDomain` to `DisksPolicy` (or `DisksDomainService`) and update call sites.

**Step 3: Re-run checks**

Run: `cargo check -p storage-service`
Expected: PASS.

**Step 4: Repeat per concern and commit frequently**

```bash
git add storage-service/src/domain storage-service/src/transport
git commit -m "refactor(storage-service): rename domain defaults to policy-oriented names"
```

## Execution Notes

- Keep commits phase-scoped and reversible.
- Avoid behavior edits during Tasks 1–4.
- Preserve D-Bus interface names/object paths exactly.
- If a step introduces broad compile failures, stop and correct imports/module declarations before continuing.

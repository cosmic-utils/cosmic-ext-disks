# Logical Volume Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver a first-class `Logical` topology and full-management v1 operations (LVM/MD RAID/multi-device BTRFS) across service and app, while preserving strict crate boundaries.

**Architecture:** Introduce canonical logical models in `storage-types`, implement discovery in two lanes (`storage-udisks` for UDisks-backed only, `storage-sys` for non-UDisks), merge and expose via a new logical DBus surface in `storage-service`, then consume in `storage-app` with a true Logical sidebar/detail UX. Keep physical disk/partition behavior backward-compatible during migration.

**Tech Stack:** Rust workspace (Cargo), zbus/DBus, UDisks2 interfaces, system tools (LVM/mdadm/btrfs where applicable), COSMIC iced UI, `just check`.

---

### Task 1: Baseline and guardrails

**Files:**
- Modify: `docs/plans/2026-02-26-layered-crate-structure-checklist.md`

**Step 1: Run baseline workspace verification**

Run: `just check`
Expected: Baseline diagnostics recorded.

**Step 2: Record baseline status**

Add timestamp + pass/fail notes under preparation checklist.

**Step 3: Commit checkpoint**

Run:
```bash
git add docs/plans/2026-02-26-layered-crate-structure-checklist.md
git commit -m "docs: record baseline before logical topology work"
```

### Task 2: UI layout and feature planning (required)

**Files:**
- Create: `docs/plans/2026-02-26-logical-volume-support-ui-plan.md`
- Modify: `docs/plans/2026-02-26-logical-volume-support-design.md`

**Step 1: Define logical detail layouts**

Specify section-by-section layouts for:
- LVM VG/LV/PV details
- MD RAID details
- multi-device BTRFS details

**Step 2: Define per-entity feature matrix**

For each entity, list:
- Read fields
- Action buttons
- Disabled states and reasons

**Step 3: Document reuse-vs-new view decisions**

Explicitly record:
- `disk_header` not reused
- partition segment control not reused
- detail-card shell reused and extended

**Step 4: Commit checkpoint**

Run:
```bash
git add docs/plans/2026-02-26-logical-volume-support-ui-plan.md docs/plans/2026-02-26-logical-volume-support-design.md
git commit -m "docs: lock logical UI layout and feature matrix"
```

### Task 3: Add canonical logical models in storage-types

**Files:**
- Create: `storage-types/src/logical.rs`
- Modify: `storage-types/src/lib.rs`
- Test: `storage-types/src/logical.rs` (unit tests in-module)

**Step 1: Write failing model tests**

Add tests for:
- logical entity kind roundtrips (serde)
- member linking
- capability/blocked-reason serialization

**Step 2: Run model tests and confirm failure**

Run: `cargo test -p storage-types logical -- --nocapture`
Expected: FAIL (module/types missing).

**Step 3: Implement minimal logical model types**

Implement core structs/enums for:
- logical entity identity
- entity-specific payload (LVM/RAID/BTRFS)
- members and health/progress fields
- capability flags and blocked reasons

**Step 4: Run tests and verify pass**

Run: `cargo test -p storage-types logical -- --nocapture`
Expected: PASS.

**Step 5: Commit checkpoint**

Run:
```bash
git add storage-types/src/logical.rs storage-types/src/lib.rs
git commit -m "feat(types): add canonical logical topology models"
```

### Task 4: Implement UDisks-only logical discovery in storage-udisks

**Files:**
- Create: `storage-udisks/src/logical/mod.rs`
- Create: `storage-udisks/src/logical/lvm_udisks.rs`
- Create: `storage-udisks/src/logical/mdraid_udisks.rs`
- Create: `storage-udisks/src/logical/btrfs_udisks.rs`
- Modify: `storage-udisks/src/lib.rs`
- Test: `storage-udisks/src/logical/mod.rs` (unit tests with parser/mapper fixtures)

**Step 1: Write failing mapping tests**

Cover UDisks object/property mapping for:
- MD RAID properties
- LVM links via UDisks LVM2 interfaces
- BTRFS multi-device filesystem identity

**Step 2: Run tests and confirm failure**

Run: `cargo test -p storage-udisks logical -- --nocapture`
Expected: FAIL.

**Step 3: Implement UDisks discovery modules**

Implement only UDisks-backed discovery calls and mapping into `storage_types::logical`.

**Step 4: Validate no non-UDisks probing is introduced**

Run: `rg "Command::new|/sbin/|which\(" storage-udisks/src/logical -n`
Expected: no matches in new logical discovery modules.

**Step 5: Run tests and verify pass**

Run: `cargo test -p storage-udisks logical -- --nocapture`
Expected: PASS.

**Step 6: Commit checkpoint**

Run:
```bash
git add storage-udisks/src/logical storage-udisks/src/lib.rs
git commit -m "feat(udisks): add udisks-backed logical topology discovery"
```

### Task 5: Move non-UDisks logical probing into storage-sys

**Files:**
- Create: `storage-sys/src/logical/mod.rs`
- Create: `storage-sys/src/logical/lvm_tools.rs`
- Create: `storage-sys/src/logical/mdadm_tools.rs`
- Create: `storage-sys/src/logical/btrfs_tools.rs`
- Modify: `storage-sys/src/lib.rs`
- Test: `storage-sys/src/logical/mod.rs`

**Step 1: Write failing tests for tool parsing/normalization**

Add fixture-based parser tests for command outputs.

**Step 2: Run tests and confirm failure**

Run: `cargo test -p storage-sys logical -- --nocapture`
Expected: FAIL.

**Step 3: Implement tool-backed fallback modules**

Implement non-UDisks discovery/probing only in `storage-sys`.

**Step 4: Run tests and verify pass**

Run: `cargo test -p storage-sys logical -- --nocapture`
Expected: PASS.

**Step 5: Commit checkpoint**

Run:
```bash
git add storage-sys/src/logical storage-sys/src/lib.rs
git commit -m "feat(sys): add non-udisks logical probing modules"
```

### Task 6: Add service-level logical DBus contracts and handler

**Files:**
- Create: `storage-service/src/handlers/logical.rs`
- Modify: `storage-service/src/handlers/mod.rs`
- Modify: `storage-service/src/main.rs`
- Modify: `storage-service/src/policies/mod.rs`
- Create: `storage-service/src/policies/logical.rs`
- Test: `storage-service/src/handlers/logical.rs`

**Step 1: Write failing handler tests**

Test for:
- list/get logical entities
- capability flags present
- error mapping and policy failures

**Step 2: Run tests and confirm failure**

Run: `cargo test -p storage-service logical -- --nocapture`
Expected: FAIL.

**Step 3: Implement logical handler orchestration**

Merge UDisks discovery + sys fallbacks and expose stable JSON payloads.

**Step 4: Register DBus path/interface**

Serve logical handler at a dedicated service path.

**Step 5: Run tests and verify pass**

Run: `cargo test -p storage-service logical -- --nocapture`
Expected: PASS.

**Step 6: Commit checkpoint**

Run:
```bash
git add storage-service/src/handlers/logical.rs storage-service/src/handlers/mod.rs storage-service/src/main.rs storage-service/src/policies/mod.rs storage-service/src/policies/logical.rs
git commit -m "feat(service): expose logical topology dbus API"
```

### Task 7: Add app client and state for logical topology

**Files:**
- Create: `storage-app/src/client/logical.rs`
- Modify: `storage-app/src/client/mod.rs`
- Create: `storage-app/src/state/logical.rs`
- Modify: `storage-app/src/state/mod.rs`
- Modify: `storage-app/src/message/mod.rs`
- Modify: `storage-app/src/update/mod.rs`
- Test: `storage-app/src/state/logical.rs`

**Step 1: Write failing state/client tests**

Cover parse/update/selection behavior for logical entities.

**Step 2: Run tests and confirm failure**

Run: `cargo test -p cosmic-ext-storage logical -- --nocapture`
Expected: FAIL.

**Step 3: Implement logical client + state**

Add fetch, refresh, and selection state for logical entities.

**Step 4: Run tests and verify pass**

Run: `cargo test -p cosmic-ext-storage logical -- --nocapture`
Expected: PASS.

**Step 5: Commit checkpoint**

Run:
```bash
git add storage-app/src/client/logical.rs storage-app/src/client/mod.rs storage-app/src/state/logical.rs storage-app/src/state/mod.rs storage-app/src/message/mod.rs storage-app/src/update/mod.rs
git commit -m "feat(app): add logical client and state layer"
```

### Task 8: Implement Logical sidebar and logical detail views

**Files:**
- Modify: `storage-app/src/views/sidebar.rs`
- Create: `storage-app/src/views/logical.rs`
- Modify: `storage-app/src/views/mod.rs`
- Modify: `storage-app/src/views/app.rs`
- Test: `storage-app/src/views/logical.rs` (view-state tests as practical)

**Step 1: Write failing UI selection/render tests**

Test:
- logical roots render in `Logical` section
- selecting logical root opens logical details
- existing disk sections unaffected

**Step 2: Run tests and confirm failure**

Run: `cargo test -p cosmic-ext-storage views::logical -- --nocapture`
Expected: FAIL.

**Step 3: Implement sidebar population and detail routing**

Populate `Logical` section from logical state and route selection to new logical detail content.

**Step 4: Implement entity-specific action rows**

Add action groups for LVM, MD RAID, multi-device BTRFS.

**Step 5: Run tests and verify pass**

Run: `cargo test -p cosmic-ext-storage views::logical -- --nocapture`
Expected: PASS.

**Step 6: Commit checkpoint**

Run:
```bash
git add storage-app/src/views/sidebar.rs storage-app/src/views/logical.rs storage-app/src/views/mod.rs storage-app/src/views/app.rs
git commit -m "feat(app): render logical sidebar and detail views"
```

### Task 9: Wire logical operations end-to-end

**Files:**
- Modify: `storage-app/src/message/volumes.rs`
- Modify: `storage-app/src/update/volumes/mod.rs`
- Create: `storage-app/src/update/logical/mod.rs`
- Modify: `storage-service/src/handlers/logical.rs`
- Test: app update tests + service handler tests

**Step 1: Write failing operation-flow tests**

Cover operation dispatch and refresh signaling for:
- LVM actions
- MD RAID actions
- BTRFS multi-device actions

**Step 2: Run tests and confirm failure**

Run: `cargo test -p cosmic-ext-storage logical_operation -- --nocapture`
Expected: FAIL.

**Step 3: Implement minimal passing operation wiring**

Wire UI messages to client calls, service handler calls, and post-op refresh/signal handling.

**Step 4: Run targeted tests and verify pass**

Run: `cargo test -p cosmic-ext-storage logical_operation -- --nocapture`
Expected: PASS.

**Step 5: Commit checkpoint**

Run:
```bash
git add storage-app/src/message/volumes.rs storage-app/src/update/volumes/mod.rs storage-app/src/update/logical/mod.rs storage-service/src/handlers/logical.rs
git commit -m "feat(logical): wire management operations end-to-end"
```

### Task 10: Documentation and final verification

**Files:**
- Modify: `README.md`
- Modify: `docs/plans/2026-02-26-logical-volume-support-design.md`
- Modify: `docs/plans/2026-02-26-logical-volume-support-ui-plan.md`

**Step 1: Confirm README future-scope section**

Ensure out-of-v1 logical classes remain explicitly listed under `Later`.

**Step 2: Run workspace verification**

Run: `just check`
Expected: PASS (or only pre-existing unrelated diagnostics).

**Step 3: Run focused tests for touched crates**

Run:
```bash
cargo test -p storage-types
cargo test -p storage-udisks
cargo test -p storage-sys
cargo test -p storage-service
cargo test -p cosmic-ext-storage
```
Expected: PASS for modified areas.

**Step 4: Final commit**

Run:
```bash
git add README.md docs/plans/2026-02-26-logical-volume-support-design.md docs/plans/2026-02-26-logical-volume-support-ui-plan.md
git commit -m "docs: finalize logical volume support scope and validation"
```

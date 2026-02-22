# Largest Files Per Category Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend the current fast scanner to retain path+size candidates and output top 20 largest files per category after totals.

**Architecture:** Keep the current one-pass traversal and add per-category bounded min-heaps in thread-local scan stats. Merge local heaps into global heaps at finalize, then emit deterministic descending top-file lists in `ScanResult` and CLI output.

**Tech Stack:** Rust 2021, std collections (`BinaryHeap`/ordered wrappers), rayon parallel scanning, clap CLI, serde JSON output, cargo test.

---

### Task 1: Extend usage result types for top files

**Files:**
- Modify: `storage-sys/src/usage/types.rs`
- Test: `storage-sys/src/usage/types.rs`

**Step 1: Write the failing test**

Add a serialization-oriented test that expects `ScanResult` to include `top_files_by_category` with path and bytes entries.

```rust
#[test]
fn scan_result_serializes_top_files() {
    // construct ScanResult with one top file
    // assert serialized JSON contains top_files_by_category and file path/bytes
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys scan_result_serializes_top_files -v`  
Expected: FAIL because fields/types do not exist yet.

**Step 3: Write minimal implementation**

Add:

```rust
pub struct TopFileEntry { pub path: PathBuf, pub bytes: u64 }
pub struct CategoryTopFiles { pub category: Category, pub files: Vec<TopFileEntry> }
```

Extend `ScanResult`:

```rust
pub top_files_by_category: Vec<CategoryTopFiles>
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys scan_result_serializes_top_files -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/types.rs
git commit -m "feat(storage-sys): extend scan result with top files per category"
```

---

### Task 2: Add bounded top-20 candidate structure in scanner

**Files:**
- Modify: `storage-sys/src/usage/scanner.rs`
- Test: `storage-sys/src/usage/scanner.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn keeps_only_top_twenty_per_category() {
    // create 25 .rs files with ascending sizes
    // run scan
    // assert category top list len == 20 and smallest retained is expected cutoff
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys keeps_only_top_twenty_per_category -v`  
Expected: FAIL due to missing top list tracking.

**Step 3: Write minimal implementation**

- Add per-category bounded heap logic in `LocalStats`.
- On every regular file, update totals + bounded heap.
- Keep `N=20` constant local to scanner for now.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys keeps_only_top_twenty_per_category -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/scanner.rs
git commit -m "feat(storage-sys): track top 20 files per category during scan"
```

---

### Task 3: Implement deterministic merge + ordering rules

**Files:**
- Modify: `storage-sys/src/usage/scanner.rs`
- Test: `storage-sys/src/usage/scanner.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn top_files_sorted_desc_then_path_for_ties() {
    // create same-size files with lexicographic path differences
    // run scan
    // assert deterministic order: size desc, path asc for ties
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys top_files_sorted_desc_then_path_for_ties -v`  
Expected: FAIL due to unstable or missing ordering.

**Step 3: Write minimal implementation**

- Define deterministic comparator:
  - heap internal comparator for bounded winners
  - final output sort: bytes descending, path ascending
- Merge local heaps into global heaps using same bounded insert policy.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys top_files_sorted_desc_then_path_for_ties -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/scanner.rs
git commit -m "feat(storage-sys): add deterministic top-file merge and ordering"
```

---

### Task 4: Populate `ScanResult.top_files_by_category`

**Files:**
- Modify: `storage-sys/src/usage/scanner.rs`
- Test: `storage-sys/src/usage/scanner.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn scan_result_contains_top_files_for_each_category() {
    // mixed fixture across code/images/docs
    // assert top_files_by_category includes all categories (possibly empty)
    // assert selected entries match expected largest files
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys scan_result_contains_top_files_for_each_category -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Build `top_files_by_category` in finalize path.
- Ensure `Other` included.
- Keep category ordering aligned with existing category list order.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys scan_result_contains_top_files_for_each_category -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/scanner.rs
git commit -m "feat(storage-sys): include per-category top files in scan results"
```

---

### Task 5: Update CLI output to print top files after totals

**Files:**
- Modify: `storage-sys/src/bin/scan-categories.rs`
- Test: `storage-sys/src/bin/scan-categories.rs` (unit formatting helper) or `storage-sys/tests/scan_categories_cli.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn cli_prints_totals_before_top_file_sections() {
    // invoke formatter or CLI
    // assert totals header appears before "Top 20 largest files -"
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys cli_prints_totals_before_top_file_sections -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Keep totals section unchanged at top.
- Add per-category blocks after totals:
  - `Top 20 largest files - {Category}`
  - numbered rows with bytes and path
  - empty-state message when no files

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys cli_prints_totals_before_top_file_sections -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/bin/scan-categories.rs storage-sys/tests/scan_categories_cli.rs
git commit -m "feat(storage-sys): print top 20 largest files per category in cli"
```

---

### Task 6: Full verification and docs alignment

**Files:**
- Modify: `docs/plans/2026-02-22-fast-file-categorization.md` (append extension notes) or keep as-is and rely on new plan file
- Optional Modify: `README.md` brief CLI output note

**Step 1: Run full crate tests**

Run: `cargo test -p cosmic-ext-storage-storage-sys -v`  
Expected: PASS.

**Step 2: Run CLI smoke check**

Run: `cargo run -p cosmic-ext-storage-storage-sys --bin scan-categories -- --threads 8`  
Expected: totals section followed by top-file sections.

**Step 3: Optional docs update implementation**

Add brief note that CLI now prints top 20 largest files per category after totals.

**Step 4: Re-run targeted verification**

Run: `cargo test -p cosmic-ext-storage-storage-sys -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add README.md docs/plans/2026-02-22-fast-file-categorization.md
git commit -m "docs: note top-file category output behavior"
```

---

## Notes for the Implementer

- Preserve one-pass scan design.
- Do not increase heap capacity beyond 20 unless a separate requirement is introduced.
- Avoid cloning paths for non-winning candidates.
- Keep JSON output backward compatible by additive fields only.
- Keep deterministic ordering for reliable tests and predictable CLI output.

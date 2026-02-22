# Scan Progress Console Reporting Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add live scan progress in CLI with percent, processed bytes, and ETA using included-mount used-bytes as denominator.

**Architecture:** Add an optional progress callback/channel to scanner internals, compute denominator from included mounts via `statvfs`, emit batched byte deltas from workers, and render a throttled single-line progress view in `scan-categories` console mode.

**Tech Stack:** Rust 2021, std sync/channel/time, libc `statvfs`, rayon scanner, clap CLI.

---

### Task 1: Add progress data model and scanner config hooks

**Files:**
- Modify: `storage-sys/src/usage/types.rs`
- Modify: `storage-sys/src/usage/mod.rs`
- Create: `storage-sys/src/usage/progress.rs`

**Step 1: Write the failing test**

Add unit test in new `progress.rs` for core math helper:

```rust
#[test]
fn percent_and_eta_handle_zero_denominator() {
    // denominator=0 => percent 0 until complete, eta none
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys percent_and_eta_handle_zero_denominator -v`  
Expected: FAIL due to missing module/types.

**Step 3: Write minimal implementation**

- Add progress types:
  - `ScanProgressSnapshot { percent, bytes_processed, eta_secs }`
  - helper functions for percent/eta math.
- Add optional progress sink/callback to scanner config surface.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys percent_and_eta_handle_zero_denominator -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/types.rs storage-sys/src/usage/mod.rs storage-sys/src/usage/progress.rs
git commit -m "feat(storage-sys): scaffold progress model and math helpers"
```

---

### Task 2: Implement mount used-bytes denominator estimation

**Files:**
- Modify: `storage-sys/src/usage/mounts.rs`
- Modify: `storage-sys/src/usage/mod.rs`
- Test: `storage-sys/src/usage/mounts.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn sums_used_bytes_for_included_mounts() {
    // use parser-level fixtures and stat provider stub
    // assert sum of used bytes across included mounts
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys sums_used_bytes_for_included_mounts -v`  
Expected: FAIL due to missing API.

**Step 3: Write minimal implementation**

- Add function to compute denominator for selected roots.
- Use `statvfs` (`f_blocks`, `f_bfree`, `f_frsize`) per included mount.
- Skip mounts on stat failure and collect warning count.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys sums_used_bytes_for_included_mounts -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/mounts.rs storage-sys/src/usage/mod.rs
git commit -m "feat(storage-sys): add used-bytes denominator estimation for progress"
```

---

### Task 3: Emit batched progress deltas from scanner workers

**Files:**
- Modify: `storage-sys/src/usage/scanner.rs`
- Test: `storage-sys/src/usage/scanner.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn scanner_emits_progress_deltas_for_processed_bytes() {
    // scan fixture files with known total bytes
    // capture emitted deltas
    // assert sum(deltas) == total scanned bytes
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys scanner_emits_progress_deltas_for_processed_bytes -v`  
Expected: FAIL because no progress emission exists.

**Step 3: Write minimal implementation**

- Add optional progress emitter argument in scan path.
- Batch delta bytes per worker and flush periodically (bytes threshold and/or loop interval).
- Ensure final flush on worker completion.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys scanner_emits_progress_deltas_for_processed_bytes -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/scanner.rs
git commit -m "feat(storage-sys): emit batched progress deltas during scan"
```

---

### Task 4: Build CLI progress renderer (console mode)

**Files:**
- Modify: `storage-sys/src/bin/scan-categories.rs`
- Modify: `storage-sys/src/usage/progress.rs`
- Test: `storage-sys/src/bin/scan-categories.rs` (formatter/helper tests)

**Step 1: Write the failing test**

```rust
#[test]
fn progress_line_formats_percent_bytes_and_eta() {
    // percent, bytes, eta => formatted line contains all fields
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys progress_line_formats_percent_bytes_and_eta -v`  
Expected: FAIL due to missing renderer/formatter.

**Step 3: Write minimal implementation**

- Start progress consumer for non-JSON mode.
- Render throttled single-line updates via `\r`.
- On completion, print final progress line + newline before totals/top-files output.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys progress_line_formats_percent_bytes_and_eta -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/bin/scan-categories.rs storage-sys/src/usage/progress.rs
git commit -m "feat(storage-sys): add live progress rendering in scan-categories"
```

---

### Task 5: Enforce JSON-mode cleanliness and output ordering

**Files:**
- Modify: `storage-sys/src/bin/scan-categories.rs`
- Test: `storage-sys/tests/scan_categories_cli.rs` (create if missing)

**Step 1: Write the failing test**

```rust
#[test]
fn json_mode_has_no_progress_noise() {
    // run --json and assert output is valid JSON with no progress prefix
}

#[test]
fn console_mode_prints_progress_before_totals_section() {
    // assert progress line appears before CATEGORY header in final output
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys json_mode_has_no_progress_noise -v`  
Expected: FAIL if behavior not enforced yet.

**Step 3: Write minimal implementation**

- Disable progress renderer entirely when `--json` is set.
- Preserve final output order: progress completion line, then totals, then top-files.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys json_mode_has_no_progress_noise -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/bin/scan-categories.rs storage-sys/tests/scan_categories_cli.rs
git commit -m "test(storage-sys): verify progress output ordering and json cleanliness"
```

---

### Task 6: Full verification and smoke run

**Files:**
- Modify: `README.md` (optional short note for progress output)

**Step 1: Run full tests**

Run: `cargo test -p cosmic-ext-storage-storage-sys -v`  
Expected: PASS.

**Step 2: Console smoke run**

Run: `cargo run -p cosmic-ext-storage-storage-sys --bin scan-categories -- --threads 8 --top-files-per-category 3`  
Expected: live progress line updates with `%`, `bytes processed`, `ETA`; then totals and top-files.

**Step 3: JSON smoke run**

Run: `cargo run -p cosmic-ext-storage-storage-sys --bin scan-categories -- --json --threads 8`  
Expected: valid JSON only, no progress lines.

**Step 4: Optional docs note**

Add one short line describing progress behavior in console mode.

**Step 5: Commit**

```bash
git add README.md
git commit -m "docs: note scan progress console behavior"
```

---

## Notes for the Implementer

- Keep scanner hot path lightweight; progress emission must be batched.
- Clamp displayed percent to `[0, 100]` and avoid negative ETA.
- Treat denominator as estimate, not exact total.
- Do not change existing final totals/top-files semantics.

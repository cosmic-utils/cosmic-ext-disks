# Fast File Categorization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a high-throughput Linux scanner in `storage-sys` that scans from `/`, includes all local mounts, excludes virtual/network mounts, and returns byte totals by file category plus `Other`.

**Architecture:** Build a new `usage` subsystem in `storage-sys` with three layers: mount discovery (`/proc/self/mountinfo`), parallel traversal, and extension-first classification with deterministic aggregation. Expose the scanner via an internal API and a debug CLI binary. Keep scan behavior best-effort (skip unreadable subtrees), while failing fast for startup-critical errors.

**Tech Stack:** Rust 2021, std fs/io/path, `rayon` for bounded parallel traversal, `clap` for CLI argument parsing, `cargo test` and crate-local integration tests.

---

### Task 1: Scaffold `usage` module and public API

**Files:**
- Modify: `storage-sys/src/lib.rs`
- Modify: `storage-sys/Cargo.toml`
- Create: `storage-sys/src/usage/mod.rs`
- Create: `storage-sys/src/usage/types.rs`
- Create: `storage-sys/src/usage/error.rs`

**Step 1: Write the failing test**

Add in `storage-sys/src/usage/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_symbols_exist() {
        let _ = ScanRequest::default();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys usage::tests::api_symbols_exist -v`  
Expected: FAIL with unresolved items (`ScanRequest`, module exports, etc.).

**Step 3: Write minimal implementation**

```rust
// usage/types.rs
#[derive(Debug, Clone, Default)]
pub struct ScanRequest;

// usage/mod.rs
pub mod error;
pub mod types;
pub use types::ScanRequest;
```

Also wire in `storage-sys/src/lib.rs`:

```rust
pub mod usage;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys usage::tests::api_symbols_exist -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/lib.rs storage-sys/Cargo.toml storage-sys/src/usage/mod.rs storage-sys/src/usage/types.rs storage-sys/src/usage/error.rs
git commit -m "feat(storage-sys): scaffold usage scanner module"
```

---

### Task 2: Implement mount discovery and local mount filtering

**Files:**
- Create: `storage-sys/src/usage/mounts.rs`
- Modify: `storage-sys/src/usage/mod.rs`
- Test: `storage-sys/src/usage/mounts.rs`

**Step 1: Write the failing test**

Add tests in `mounts.rs`:

```rust
#[test]
fn parses_mountinfo_and_filters_non_local_types() {
    let sample = "36 25 0:32 / / rw - ext4 /dev/nvme0n1p2 rw\n37 25 0:5 / /proc rw - proc proc rw\n38 25 0:40 / /mnt/nfs rw - nfs server:/x rw\n";
    let mounts = parse_local_mounts(sample).unwrap();
    assert_eq!(mounts, vec!["/".to_string()]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys parse_local_mounts -v`  
Expected: FAIL because parser/filter does not exist.

**Step 3: Write minimal implementation**

```rust
pub fn parse_local_mounts(input: &str) -> Result<Vec<String>, UsageScanError> {
    // parse mount point + fs type from mountinfo lines
    // keep local fs types, drop proc/sysfs/tmpfs/devtmpfs/overlay/squashfs/nfs/cifs/fuse.sshfs
    // return sorted unique mount points
}
```

```rust
pub fn discover_local_mounts() -> Result<Vec<std::path::PathBuf>, UsageScanError> {
    let raw = std::fs::read_to_string("/proc/self/mountinfo")?;
    Ok(parse_local_mounts(&raw)?.into_iter().map(std::path::PathBuf::from).collect())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys parse_local_mounts -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/mounts.rs storage-sys/src/usage/mod.rs
git commit -m "feat(storage-sys): add local mount discovery and filtering"
```

---

### Task 3: Implement extension-first classifier with `Other` fallback

**Files:**
- Create: `storage-sys/src/usage/classifier.rs`
- Modify: `storage-sys/src/usage/types.rs`
- Modify: `storage-sys/src/usage/mod.rs`
- Test: `storage-sys/src/usage/classifier.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn classifies_by_extension_and_falls_back_to_other() {
    assert_eq!(classify_path("/tmp/file.rs"), Category::Code);
    assert_eq!(classify_path("/tmp/pic.JPG"), Category::Images);
    assert_eq!(classify_path("/tmp/noext"), Category::Other);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys classifies_by_extension_and_falls_back_to_other -v`  
Expected: FAIL due to missing classifier/category enum.

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Category {
    Documents,
    Images,
    Audio,
    Video,
    Archives,
    Code,
    Binaries,
    Other,
}

pub fn classify_path(path: &str) -> Category {
    // lowercase ext lookup
    // known ext => category
    // else Category::Other
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys classifies_by_extension_and_falls_back_to_other -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/classifier.rs storage-sys/src/usage/types.rs storage-sys/src/usage/mod.rs
git commit -m "feat(storage-sys): add extension-based category classifier"
```

---

### Task 4: Implement parallel scanner core

**Files:**
- Create: `storage-sys/src/usage/scanner.rs`
- Modify: `storage-sys/Cargo.toml`
- Modify: `storage-sys/src/usage/mod.rs`
- Modify: `storage-sys/src/usage/types.rs`
- Test: `storage-sys/src/usage/scanner.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn aggregates_category_bytes_over_tree() {
    // build temp dir with:
    // src/main.rs (10 bytes), img.png (20 bytes), note.txt (30 bytes)
    // run scan_paths([temp_root])
    // assert totals: Code=10, Images=20, Documents=30
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys aggregates_category_bytes_over_tree -v`  
Expected: FAIL due to missing `scan_paths` implementation.

**Step 3: Write minimal implementation**

```rust
pub fn scan_paths(roots: &[std::path::PathBuf], cfg: &ScanConfig) -> Result<ScanResult, UsageScanError> {
    // rayon parallel over roots
    // iterative walk with Vec<PathBuf> stack
    // symlink_metadata; regular files only
    // bytes = metadata.len(); classify and accumulate thread-local
    // merge results
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys aggregates_category_bytes_over_tree -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/Cargo.toml storage-sys/src/usage/scanner.rs storage-sys/src/usage/mod.rs storage-sys/src/usage/types.rs
git commit -m "feat(storage-sys): add parallel category scanner core"
```

---

### Task 5: Add end-to-end scan entrypoint from `/` local mounts

**Files:**
- Modify: `storage-sys/src/usage/mod.rs`
- Modify: `storage-sys/src/usage/types.rs`
- Test: `storage-sys/src/usage/mod.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn scan_local_mounts_returns_other_for_unknown_extensions() {
    // unit-level test on reducer/merge path with unknown extension case
    // assert `Other` always present when unknown files encountered
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys scan_local_mounts_returns_other_for_unknown_extensions -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

```rust
pub fn scan_local_mounts(cfg: &ScanConfig) -> Result<ScanResult, UsageScanError> {
    let mounts = mounts::discover_local_mounts()?;
    scanner::scan_paths(&mounts, cfg)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys scan_local_mounts_returns_other_for_unknown_extensions -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/mod.rs storage-sys/src/usage/types.rs
git commit -m "feat(storage-sys): expose local-mount scan entrypoint"
```

---

### Task 6: Add debug/prototype CLI binary

**Files:**
- Create: `storage-sys/src/bin/scan-categories.rs`
- Modify: `storage-sys/Cargo.toml`
- Test: `storage-sys/tests/scan_categories_cli.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn cli_prints_category_table_headers() {
    let cmd = assert_cmd::Command::cargo_bin("scan-categories").unwrap();
    cmd.arg("--help").assert().success().stdout(predicates::str::contains("CATEGORY"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys cli_prints_category_table_headers -v`  
Expected: FAIL because binary/test deps not yet wired.

**Step 3: Write minimal implementation**

```rust
fn main() -> anyhow::Result<()> {
    // clap args: --root (default /), --json optional
    // call usage::scan_local_mounts()
    // print sorted categories + summary metrics
    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys cli_prints_category_table_headers -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/bin/scan-categories.rs storage-sys/Cargo.toml storage-sys/tests/scan_categories_cli.rs
git commit -m "feat(storage-sys): add scan-categories debug cli"
```

---

### Task 7: Full verification and docs touch-up

**Files:**
- Modify: `storage-sys/README.md` (if present; otherwise skip)
- Modify: `README.md` (short debug command note)

**Step 1: Write the failing test/check**

```text
No new unit test in this task; verification is command-based quality gate.
```

**Step 2: Run verification before docs updates**

Run: `cargo test -p cosmic-ext-storage-storage-sys -v`  
Expected: PASS all tests in `storage-sys`.

**Step 3: Write minimal documentation implementation**

Add brief usage example:

```bash
cargo run -p cosmic-ext-storage-storage-sys --bin scan-categories -- --root /
```

**Step 4: Run verification again**

Run: `cargo test -p cosmic-ext-storage-storage-sys -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add README.md storage-sys/README.md
git commit -m "docs: add scan-categories debug usage"
```

---

## Notes for the Implementer

- Keep behavior deterministic: sorted mounts, sorted category output.
- Keep hot path allocation-light.
- Do not follow symlinks.
- Continue on subtree errors; count and report skipped errors.
- Keep mount filtering isolated to support later policy flags.
- Prefer incremental PR-size commits (one per task).

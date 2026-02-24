# Fast File Categorization Design

Date: 2026-02-22  
Status: Approved

## Problem Statement

We need the fastest accurate way to scan from `/` on Linux and produce category totals (bytes) such as Documents, Images, Audio, Code, and Other. The scan must include **all local mounts** and exclude non-local or virtual filesystems (for example proc/sys/run/network mounts). Mount filtering beyond this baseline will be added later.

This design is for `storage-sys` and must expose a CLI tool for debugging and prototyping.

## Goals

- Maximize scan throughput on Linux while preserving accurate per-file byte accounting.
- Include all local mounts reachable from `/`.
- Exclude pseudo/runtime/network filesystem mounts.
- Classify files into fixed categories and aggregate bytes per category.
- Put unknown/unmapped files into `Other`.
- Provide a debug/prototype CLI for repeatable validation.

## Non-Goals

- User-configurable mount include/exclude policy (deferred).
- Content-based file type detection (no file reads for detection).
- Exact allocated-on-disk block usage accounting.
- UI integration.

## Recommended Approach

Implement a native Rust parallel scanner in `storage-sys` with three layers:

1. Mount discovery and filtering from `/proc/self/mountinfo`
2. Parallel traversal of selected roots
3. Lock-light category byte aggregation

This is preferred over subprocess-based strategies and lower-level syscall-only implementations because it delivers high speed, robust accuracy, and maintainability.

## Architecture

### 1) Mount Discovery

- Read and parse `/proc/self/mountinfo` once at startup.
- Build scan roots from mounts that are local filesystems.
- Exclude known pseudo/runtime/network fs types (for example: `proc`, `sysfs`, `tmpfs`, `devtmpfs`, `overlay`, `squashfs`, `nfs`, `cifs`, `fuse.sshfs`, and similar non-local classes).
- Keep result deterministic and sorted to stabilize output and tests.

### 2) Traversal Engine

- Start one top-level job per selected mount.
- Use iterative directory walking (`read_dir` + `DirEntry`) and `symlink_metadata`.
- Count only regular files.
- Do not follow symlinks.
- Continue scan on recoverable errors (permission denied, transient IO on subtree).
- Use bounded parallelism based on available CPUs.

### 3) Aggregation

- Per-worker local counters for:
  - total bytes
  - total regular files
  - total directories visited
  - category byte map
- Merge worker totals at the end to avoid per-file locking.
- Emit compact scan stats including skipped error count.

## Data Model

### Categories

Minimum category set:

- Documents
- Images
- Audio
- Video
- Archives
- Code
- Binaries
- Other

`Other` is the required fallback for unknown/ambiguous extensions.

### Result Shape (conceptual)

- `categories: Vec<{ category, bytes }>` (sorted by descending bytes)
- `total_bytes`
- `files_scanned`
- `dirs_scanned`
- `skipped_errors`
- `mounts_scanned`
- `elapsed_ms`

## Classification Strategy

### Fast Path (default)

- Extension-based classification only.
- Prebuilt lowercase extension lookup map for O(1) mapping.
- Case-insensitive extension handling.

### Fallback

- Optional and disabled by default: lightweight MIME hint for extensionless/unknown names.
- If unresolved, classify as `Other`.

### Accuracy Rules

- Byte accounting uses logical file size (`metadata.len()`).
- No content reads.
- Hidden files are treated like any other file.
- No symlink-target traversal.

## Error Handling

- Hard fail only on startup-critical failures (for example, mountinfo unreadable).
- Best-effort scanning thereafter:
  - Record and count subtree/file errors.
  - Continue scanning remaining work.
- CLI returns non-zero only for startup-critical failure.

## CLI Prototype

Add a debug/prototype binary under `storage-sys`:

- Command: `scan-categories`
- Default root behavior: start from `/`, discover all local mounts, scan each included root.
- Output:
  - category totals by bytes (descending)
  - optional percent of total
  - summary footer (`elapsed_ms`, files, dirs, skipped_errors, mounts_scanned)

Future mount filtering flags can be added without changing scanner internals.

## Performance Considerations

- Avoid subprocess invocation.
- Avoid `canonicalize` in hot paths.
- Avoid content reads and repeated allocations.
- Use thread-local counters; merge once.
- Keep queueing bounded and predictable.

## Testing Strategy

1. Unit tests: extension-to-category mapping and `Other` fallback.
2. Unit tests: mount filter classification (local vs excluded fs types).
3. Integration tests: synthetic temp tree with known expected bytes/category totals.
4. Regression/consistency tests: deterministic output ordering.
5. Optional benchmark smoke test (non-gating) for throughput trends.

## Rollout Notes

- Implement in `storage-sys` as isolated scanner module + CLI binary.
- Keep APIs internal-first, then expose for broader use once CLI outputs are validated.
- Add mount-policy options later as a thin layer over existing mount discovery/filter stage.

## Trade-offs Accepted

- Using logical file size instead of block usage to keep cross-filesystem consistency and speed.
- Extension-first categorization for throughput; richer MIME/content inference deferred.
- Best-effort scan continuity over strict fail-fast behavior on subtree errors.

# Largest Files Per Category Design

Date: 2026-02-22  
Status: Approved

## Problem Statement

Extend the existing fast category scanner so it also reports the largest files per category. The scanner must preserve current total-by-category output and then print top files per category afterwards.

The initial categorization pass must capture path + size information needed to produce this result efficiently.

## Scope

- Keep existing categories and totals logic.
- Add top-N largest files per category (`N = 20` default for now).
- Include `Other` in top-file output.
- Print totals first, then top files per category in console output.

## Goals

- One-pass scanning only (no second filesystem pass).
- Bounded memory for top-file tracking.
- Deterministic output ordering.
- Minimal scan throughput impact.

## Non-Goals

- User-configurable `N` (future enhancement).
- Extra content/MIME probing.
- UI integration.

## Recommended Approach

Use per-category bounded min-heaps during the existing scan, with per-thread local heaps that are merged at finalize.

### Why this approach

- Preserves one-pass scan speed.
- Keeps memory bounded to roughly `categories * N` entries per worker.
- Avoids global lock contention in hot path.
- Naturally composes with current thread-local stats merge.

## Architecture Changes

### 1) Scanner internals

- Extend local scan stats to track top candidates per category:
  - Existing: `bytes_by_category`, totals, counters
  - New: `top_files_heap_by_category`
- On each regular file:
  1. Classify category
  2. Add to category bytes
  3. Consider `(path, size)` for that category heap

### 2) Bounded top-N policy

Per category heap behavior (`N=20`):

- If heap size `< N`: push candidate.
- If heap size `== N`:
  - compare against heap minimum (smallest current winner)
  - replace only if candidate is larger
  - tie-break by path string for deterministic behavior.

### 3) Thread merge

- Keep current per-root/per-thread local stats.
- During merge, feed each local heap entry into global category heap using the same bounded insertion rule.
- Finalize heaps into descending sorted vectors.

## Data Model Changes

Add result structures:

- `TopFileEntry { path: PathBuf, bytes: u64 }`
- `CategoryTopFiles { category: Category, files: Vec<TopFileEntry> }`
- `ScanResult.top_files_by_category: Vec<CategoryTopFiles>`

Sorting for final vectors:

- primary: `bytes` descending
- secondary: `path` ascending

## Console Output Contract

Console output ordering:

1. Existing totals section (unchanged, first)
2. New per-category sections:
   - `Top 20 largest files - <Category>`
   - ranked rows with size + path

Include all categories, including `Other`. If a category has no files, print an explicit empty-state line.

## Error Handling

- Keep existing best-effort scanning behavior.
- Top-file tracking never introduces hard failures.
- Permission/read failures continue to increment `skipped_errors` and scanning proceeds.

## Performance Considerations

- No second pass.
- Heap operations are $O(log N)$ with small fixed `N=20`.
- Avoid path string allocation unless candidate enters/replaces heap.
- Keep merge cost proportional to `categories * N * workers`.

## Testing Plan

1. Unit test: per-category top list capped at 20.
2. Unit test: ordering correctness (size desc, deterministic tie-break).
3. Integration test: mixed fixture validates totals and top-file consistency.
4. CLI test: verifies totals print before any top-file sections.

## Rollout Notes

- Extend scanner/types first.
- Update CLI formatting next.
- Preserve JSON compatibility by adding new field without altering existing keys.
- Keep implementation isolated to `storage-sys` usage module and CLI binary.

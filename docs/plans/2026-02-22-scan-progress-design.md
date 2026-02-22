# Scan Progress Console Reporting Design

Date: 2026-02-22  
Status: Approved

## Problem Statement

Add scan progress reporting to console output for `scan-categories`.

Progress output must include:
- percent complete
- bytes processed (sum of scanned file sizes)
- ETA

The denominator for percent should be based on the total used bytes of included mounts.

## Goals

- Keep scanner fast (no second full filesystem pass).
- Provide useful real-time progress for long scans.
- Keep output readable and stable in console mode.
- Preserve clean JSON output mode.

## Non-Goals

- Exact completion percentage guarantees under all FS dynamics.
- Historical progress persistence.
- Progress UI in non-CLI consumers.

## Recommended Approach

Use **included-mount used-bytes** as the total-work estimate and compute runtime progress from scanned file bytes.

### Rationale

- Fast pre-scan denominator from mount metadata.
- No second tree walk.
- Good practical ETA signal for large scans.

## Architecture

### 1) Denominator discovery

- Reuse included mount list from existing mount-discovery/filter stage.
- For each included mount, read filesystem usage via `statvfs`.
- Compute estimated used bytes per mount:

`used_bytes = (f_blocks - f_bfree) * f_frsize`

- Sum all included mounts to get `estimated_total_used_bytes`.

### 2) Progress event stream

- Scanner workers emit byte deltas while scanning regular files.
- Worker-side batching reduces overhead (flush deltas periodically, not per file).
- Progress channel is consumed by a renderer/aggregator in CLI runtime.

### 3) Runtime progress model

Track:
- `bytes_processed`
- `started_at`
- smoothed throughput (`bytes/sec`, EWMA)

Compute:
- `% = clamp(bytes_processed / estimated_total_used_bytes, 0..1) * 100`
- `ETA = (estimated_total_used_bytes - bytes_processed) / smoothed_throughput`

Render line (non-JSON mode):

`Progress: 42.7% | 123.4 GiB processed | ETA 00:18:32`

## Console Behavior

- Progress line updates in place with `\r` and throttled refresh interval (for example 250ms).
- On completion:
  - force final progress line (`100.0%`)
  - print newline
  - print existing totals and per-category top-files output.
- In `--json` mode, suppress live progress line entirely.

## Error Handling

- If one mount usage stat fails, skip it from denominator and continue scan.
- If denominator resolves to zero:
  - show `0.0%` and `ETA --:--:--` during scan
  - show final completion line at finish.
- Progress channel/render failures do not fail scan; final result still prints.

## Performance Considerations

- No second pass over filesystem tree.
- Batching progress deltas to avoid channel overhead in hot path.
- Render throttling prevents console churn.
- Keep all progress math on consumer side, not in critical scan loops.

## Testing Plan

1. Unit test: denominator calculation from mount stats inputs.
2. Unit test: progress math (% clamp, zero denominator, ETA behavior).
3. Unit test/integration: batched progress deltas aggregate correctly.
4. CLI test: non-JSON mode prints progress line and final report order is preserved.
5. CLI test: JSON mode has no progress line noise.

## Rollout Notes

- Add progress support in scanner API as optional callback/channel so existing callers can opt out.
- Wire `scan-categories` to opt in for console mode only.
- Keep final data result structures unchanged unless final summary progress fields are explicitly needed later.

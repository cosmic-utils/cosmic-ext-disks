# Audit Implementation Plan — Remaining GAPs

**Audit:** 2026-02-06T19-14-59Z  
**Status:** 8 fixes complete (GAP-001, 002, 005, 006, 009, 010, 011, 012)  
**Remaining:** 2 large structural refactorings (GAP-003, 004) + 2 low-priority (GAP-007, 008)

---

## ✅ Completed (8 fixes)

1. **GAP-001** — Removed 17 unused i18n keys
2. **GAP-002** — Replaced glob re-exports with explicit exports (3 modules, 39 exports documented)
3. **GAP-005** — Replaced blanket dead_code suppression with targeted annotations (11 functions documented)
4. **GAP-006** — Deleted typo method
5. **GAP-009** — Fixed unwrap() calls in logging
6. **GAP-010** — Removed println! debug code
7. **GAP-011** — Documented module structure convention
8. **GAP-012** — Fixed 7 untranslated UI strings (added 8 new i18n keys)

---

## ⏳ Remaining (Large Refactorings)

### GAP-003: Split volume_model.rs (944 lines → 7 files)
**Severity:** Medium  
**Effort:** 4-6 hours  
**Risk:** Medium (internal refactor, well-tested)

**Target structure:**
```
disks/volume_model/
├── mod.rs           # VolumeModel struct + common (~150 lines)
├── types.rs         # VolumeType enum (~20 lines)
├── mount.rs         # mount(), unmount(), mount options (~150 lines)
├── encryption.rs    # unlock(), lock(), change_passphrase() (~150 lines)
├── filesystem.rs    # format(), check(), repair(), edit_label() (~120 lines)
├── partition.rs     # edit_partition(), resize(), delete() (~100 lines)
└── config.rs        # configuration parsing helpers (~180 lines)
```

**Dependencies:**
- Must preserve all public API
- Tests need to be reorganized
- Some helper functions shared across modules need careful placement

**Recommended approach:**
1. Create folder structure
2. Move tests first (easiest to verify)
3. Move methods one group at a time
4. Keep `mod.rs` re-exporting everything
5. Verify `cargo test` after each move

**Blocked by:** Time allocation (not technically complex, just tedious)

---

### GAP-004: Data-Driven Partition Types (1503 lines → data file)
**Severity:** Medium  
**Effort:** 6-8 hours  
**Risk:** Low (mostly data transformation, well-isolated)

**Current:** 
- `disks-dbus/src/partition_types/gpt.rs` — 1503 lines of hardcoded structs
- `disks-dbus/src/partition_types/dos.rs` — 357 lines

**Target:**
```
disks-dbus/src/partition_types/
├── mod.rs
├── types.rs (PartitionTypeInfo struct)
├── gpt_types.toml (data file ~500 lines)
├── dos_types.toml (data file ~100 lines)
└── load.rs (parsing logic ~100 lines)
```

**Dependencies:**
- Need to add `serde` + `toml` to dependencies
- Need to decide: build-time (include_str!) or runtime parsing?
- Extract all ~150 GPT + ~30 DOS type definitions to TOML

**Recommended approach:**
1. Add dependencies: `serde = { version = "1", features = ["derive"] }`, `toml = "0.8"`
2. Create TOML schema
3. Extract first 10 types as proof-of-concept
4. Write parser with `lazy_static` or `OnceLock`
5. Migrate remaining types
6. Delete old hardcoded constants
7. Benchmark to ensure no runtime regression

**Blocked by:** Decision on build-time vs runtime; need to verify no performance regression

---

## 📋 Lower Priority (Not Blocking)

### GAP-007: TODO Comments
**Severity:** Low  
**Effort:** 30 minutes  
**Risk:** None

**What to do:**
- Create GitHub issues for 3 TODOs
- Link them in code comments
- Or delete if not actionable

### GAP-008: Excessive Clone
**Severity:** Low  
**Effort:** 1 hour (investigation)  
**Risk:** Low

**What to do:**
- Profile clone overhead in hot paths
- Only 1 instance found; document if intentional

---

## Recommendation

**Immediate next steps:**
1. ✅ Complete GAP-005 (decide which dead code to keep)
2. Start GAP-002 Phase 1 (add explicit exports alongside glob exports)
3. Defer GAP-003 and GAP-004 until after PR merge (these are internal refactors with no user-visible impact)

**Rationale:**
- GAP-005 is 90% done, finish it
- GAP-002 improves API clarity but is risky; do it in phases
- GAP-003/004 are large internal refactors that don't fix bugs or add features; save for dedicated refactor PR

**Total remaining effort:** ~10-15 hours for full completion of all GAPs

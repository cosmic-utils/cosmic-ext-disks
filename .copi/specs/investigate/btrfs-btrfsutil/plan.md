# BTRFS btrfsutil Migration — Implementation Spec

**Branch:** `investigate/btrfs-btrfsutil`  
**Type:** Major Refactor (V2.0 Breaking Change)  
**Estimated Effort:** 4-6 weeks (destructive migration, no legacy code)  
**Status:** Ready for Implementation

---

## Goal

Replace UDisks2 D-Bus BTRFS integration with direct `btrfsutil` crate usage to unlock:
- **17+ metadata fields** (UUIDs, timestamps, generations, flags)
- **6+ advanced operations** (read-only control, default subvolume, deleted tracking)
- **2-5x performance improvement** (direct syscalls vs D-Bus IPC)
- **Simplified dependencies** (remove udisks2-btrfs package)

## Scope

### In Scope
- ✅ Complete removal of `disks-dbus/src/disks/btrfs.rs` (289 lines)
- ✅ New `disks-dbus/src/disks/btrfs_native.rs` implementation using btrfsutil
- ✅ Privilege helper binary (`cosmic-ext-disks-btrfs-helper`)
- ✅ Enhanced UI with 6 new V2.0 features
- ✅ Polkit integration for privilege escalation
- ✅ Integration test suite with real BTRFS filesystems
- ✅ Feature parity with V1 (list, create, delete, snapshot operations)

### Out of Scope
- ❌ Snapshot scheduling (V3.0 feature)
- ❌ Send/receive support (V3.0 feature)
- ❌ Compatibility layer (clean break, V2.0)
- ❌ Feature flags (btrfsutil is the only implementation)

## Architecture Change

### Current (UDisks2)
```
UI → Business Logic → D-Bus Wrapper → UDisks2 Daemon (polkit) → Kernel
     3 metadata fields, 4 operations, D-Bus IPC overhead
```

### New (btrfsutil)
```
UI → Business Logic → BTRFS Native → btrfsutil crate → libbtrfsutil.so → Helper (polkit) → Kernel
     17+ metadata fields, 10+ operations, direct syscalls
```

### Key Components
1. **btrfs_native.rs** - Async wrapper around btrfsutil with enhanced types
2. **BtrfsHelper** - Privilege escalation via separate binary
3. **cosmic-ext-disks-btrfs-helper** - Minimal privileged binary with polkit
4. **Enhanced BtrfsSubvolume** - Full metadata (UUIDs, timestamps, flags)

## Acceptance Criteria

### Must Have (V2.0 Blocking)
- [ ] **AC-1:** All V1 operations work (list, create, delete, snapshot)
- [ ] **AC-2:** No UDisks2 BTRFS code remains in codebase
- [ ] **AC-3:** Privilege helper binary installed and working
- [ ] **AC-4:** Polkit policy correctly integrated
- [ ] **AC-5:** All subvolume metadata displayed (UUIDs, timestamps)
- [ ] **AC-6:** Read-only toggle UI functional
- [ ] **AC-7:** Default subvolume badge and set operation working
- [ ] **AC-8:** Deleted subvolumes section functional
- [ ] **AC-9:** Integration tests passing (requires root + BTRFS)
- [ ] **AC-10:** No memory leaks in blocking operations
- [ ] **AC-11:** `cargo test --workspace` passes
- [ ] **AC-12:** `cargo clippy --workspace` passes with no warnings
- [ ] **AC-13:** `cargo fmt --check` passes

### Should Have (V2.0 Goals)
- [ ] **AC-14:** Context menu with common operations
- [ ] **AC-15:** Properties dialog showing all metadata
- [ ] **AC-16:** Automatic snapshot naming with templates
- [ ] **AC-17:** Relative time display for timestamps
- [ ] **AC-18:** Localization for all new strings

### Nice to Have (V2.1+)
- [ ] **AC-19:** Snapshot relationship visualization
- [ ] **AC-20:** Batch operations support
- [ ] **AC-21:** Usage breakdown per subvolume

## Technical Constraints

1. **Privilege Model:** CAP_SYS_ADMIN operations must go through helper binary
2. **Blocking Calls:** All btrfsutil calls must use `tokio::spawn_blocking()`
3. **Rust Edition:** 2024 (per repo-rules.md)
4. **Error Handling:** Rich context with `anyhow`
5. **Commit Messages:** Conventional Commits format
6. **Testing:** CI must pass all quality gates

## Dependencies

### Add
- `btrfsutil = "0.2.0"` (disks-dbus)
- `uuid = { version = "1.10", features = ["serde"] }` (disks-dbus)
- `chrono = { version = "0.4", features = ["serde"] }` (disks-dbus)
- `clap = { version = "4.5", features = ["derive"] }` (helper)
- `serde_json = "1.0"` (helper)

### Remove
- System dependency: `udisks2-btrfs` package (packaging scripts)

### New Crate
- `disks-btrfs-helper` - Workspace member for privileged binary

## Risk Mitigation

### Risk 1: btrfsutil Crate Maintenance
**Status:** Low activity since 2021  
**Mitigation:** 
- Small codebase (~1000 lines), wraps stable C library
- Can fork to cosmic-utils if needed
- Wraps libbtrfsutil v1.2.0 (stable since 2019)

### Risk 2: Privilege Escalation Complexity
**Mitigation:**
- Minimal helper binary (<500 lines)
- Path validation (must be under /mnt or /media)
- Polkit authentication required
- Audit logging of operations

### Risk 3: Regression from UDisks2
**Mitigation:**
- Extensive integration testing
- Detailed error messages
- Test with real BTRFS filesystems

## Success Metrics

- ✅ Feature parity achieved
- ✅ All acceptance criteria met
- ✅ Performance 2-5x faster (measured via benchmarks)
- ✅ No new bugs filed within 2 weeks of release
- ✅ CI passing on all platforms

## References

- **Investigation:** `.copi/specs/investigate/btrfs-btrfsutil/investigation.md`
- **Detailed Migration Plan:** `.copi/specs/investigate/btrfs-btrfsutil/migration-plan.md`
- **Feature Recommendations:** `.copi/specs/investigate/btrfs-btrfsutil/feature-recommendations.md`
- **btrfsutil Crate:** https://crates.io/crates/btrfsutil
- **libbtrfsutil Docs:** https://docs.rs/btrfsutil

## Implementation Notes

This is a **destructive migration** - no compatibility layer, no feature flags. Clean V2.0 architecture.

See `tasks.md` for detailed implementation steps organized by phase.

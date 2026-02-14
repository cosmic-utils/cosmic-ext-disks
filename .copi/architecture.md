# Architecture — cosmic-ext-disks

Last updated: 2026-01-24

## Overview

COSMIC Disks is a Rust workspace that provides:

- A COSMIC/libcosmic UI application (`cosmic-ext-disks`) for viewing and performing basic disk/partition operations.
- A DBus/udisks abstraction crate (`cosmic-ext-storage-dbus`, used as `storage-dbus`) that talks to UDisks2 and provides higher-level models (`DriveModel`, `PartitionModel`) and helpers.

Evidence:
- Workspace layout: [Cargo.toml](../Cargo.toml)
- UI crate: [storage-ui/Cargo.toml](../storage-ui/Cargo.toml)
- DBus crate: [storage-dbus/Cargo.toml](../storage-dbus/Cargo.toml)

## Repo Structure

- `.github/workflows/`
  - CI checks (test/clippy/fmt): [.github/workflows/ci.yml](../.github/workflows/ci.yml)
  - Publish workflow (push to main): [.github/workflows/main.yml](../.github/workflows/main.yml)
- `storage-ui/` — COSMIC GUI app
  - Entrypoint: [storage-ui/src/main.rs](../storage-ui/src/main.rs)
  - Application model + subscriptions: [storage-ui/src/app.rs](../storage-ui/src/app.rs)
  - Views: [storage-ui/src/views](../storage-ui/src/views)
  - Resources (.desktop/.metainfo/icons/i18n): [storage-ui/resources](../storage-ui/resources), [storage-ui/i18n](../storage-ui/i18n)
  - Packaging/dev recipes: [storage-ui/justfile](../storage-ui/justfile)
- `storage-dbus/` — DBus/UDisks2 abstraction
  - Public API surface: [storage-dbus/src/lib.rs](../storage-dbus/src/lib.rs)
  - Models and operations: [storage-dbus/src/disks](../storage-dbus/src/disks)
  - Partition types catalog: [storage-dbus/src/partition_type.rs](../storage-dbus/src/partition_type.rs)

## Architecture Diagram (text)

```
+------------------------------+            +--------------------------+
| storage-ui (COSMIC/libcosmic)  |  DBus/IPC  | UDisks2 system service   |
| - AppModel + views           +----------->+ org.freedesktop.UDisks2  |
| - nav of drives/partitions   |            | Manager/Drive/Partition  |
+--------------+---------------+            +------------+-------------+
               |                                             |
               | local command                                | kernel/storage
               v                                             v
        +--------------+                               +-------------+
        | `df` command |                               | block devs  |
        | usage data   |                               | partitions  |
        +--------------+                               +-------------+
```

## Data Flow

### Startup / steady state

1) UI starts and initializes localization.
   - Evidence: [storage-ui/src/main.rs](../storage-ui/src/main.rs), [storage-ui/src/i18n.rs](../storage-ui/src/i18n.rs)

2) UI bootstraps the application model and fetches initial drives.
   - Evidence: `DriveModel::get_drives()` called in `AppModel::init`: [storage-ui/src/app.rs](../storage-ui/src/app.rs)

3) Drive discovery in `storage-dbus`:
   - Connects to the system bus.
   - Enumerates block devices via UDisks2 Manager.
   - Filters out partition objects and keeps drive objects.
   - Reads partition table + partitions for each drive.
   - Enriches partitions with “usage” by calling `df --block-size=1`.
   - Evidence: drive enumeration: [storage-dbus/src/disks/drive.rs](../storage-dbus/src/disks/drive.rs), usage enrichment: [storage-dbus/src/usage.rs](../storage-dbus/src/usage.rs)

4) UI renders:
   - Drives in the COSMIC nav bar; the active drive includes a `VolumesControl` model.
   - Partitions/free-space rendered as segments.
   - Evidence: nav building + active `VolumesControl`: [storage-ui/src/app.rs](../storage-ui/src/app.rs), segment logic: [storage-ui/src/views/volumes.rs](../storage-ui/src/views/volumes.rs)

### Device change detection

- `DiskManager` subscribes to UDisks2 add/remove events via `org.freedesktop.DBus.ObjectManager` signals.
- Events are filtered to objects affecting the `org.freedesktop.UDisks2.Block` interface and emitted as `Added/Removed`.
- UI subscribes to these events and refreshes the drive list by re-running `DriveModel::get_drives()`.

Evidence:
- Signal stream: [storage-dbus/src/disks/manager.rs](../storage-dbus/src/disks/manager.rs)
- UI subscription: [storage-ui/src/app.rs](../storage-ui/src/app.rs)

### User actions

#### Mount / unmount partition

- UI action dispatches to `PartitionModel::mount()` / `PartitionModel::unmount()`.
- These call the UDisks2 filesystem interface on the partition path.

Evidence:
- UI wiring: [storage-ui/src/views/volumes.rs](../storage-ui/src/views/volumes.rs)
- DBus calls: [storage-dbus/src/disks/partition.rs](../storage-dbus/src/disks/partition.rs)

#### Delete partition

- UI action dispatches `PartitionModel::delete()`.
- Attempts unmount first, then calls UDisks2 Partition `delete()`.

Evidence:
- UI wiring: [storage-ui/src/views/volumes.rs](../storage-ui/src/views/volumes.rs)
- DBus calls: [storage-dbus/src/disks/partition.rs](../storage-dbus/src/disks/partition.rs)

#### Create partition (+ format)

- UI shows a create dialog for “free space” segments.
- On confirmation, calls `DriveModel::create_partition(CreatePartitionInfo)`.
- `DriveModel::create_partition`:
  - Gets the partition table type (`gpt` / `dos`).
  - Picks a partition-type entry from a catalog (common GPT/DOS types).
  - Calls UDisks2 PartitionTable `create_partition(offset, size, type, name, options)`.
  - Then formats the newly created partition via UDisks2 Block `format(filesystem_type, options)`.

Evidence:
- Dialog + message flow: [storage-ui/src/views/dialogs.rs](../storage-ui/src/views/dialogs.rs), [storage-ui/src/views/volumes.rs](../storage-ui/src/views/volumes.rs)
- Partition creation + format: [storage-dbus/src/disks/drive.rs](../storage-dbus/src/disks/drive.rs)
- Partition type catalog: [storage-dbus/src/partition_type.rs](../storage-dbus/src/partition_type.rs)

## Core Components / Modules

### `storage-ui` (GUI)

- `AppModel` (COSMIC Application): app state, nav items, dialogs, subscriptions.
  - Evidence: [storage-ui/src/app.rs](../storage-ui/src/app.rs)
- `views/volumes.rs`: partition/free-space segmentation and main user interaction surface.
  - Evidence: [storage-ui/src/views/volumes.rs](../storage-ui/src/views/volumes.rs)
- Localization: `i18n-embed` + `rust-embed` for Fluent translations.
  - Evidence: [storage-ui/src/i18n.rs](../storage-ui/src/i18n.rs), [storage-ui/i18n](../storage-ui/i18n)
- Build metadata: version info from git for the About view via `vergen`.
  - Evidence: [storage-ui/build.rs](../storage-ui/build.rs), [storage-ui/src/views/about.rs](../storage-ui/src/views/about.rs)

### `storage-dbus` (UDisks2 abstraction)

- `DriveModel`: enumerates drives and exposes `eject`, `power_off`, and `create_partition`.
  - Evidence: [storage-dbus/src/disks/drive.rs](../storage-dbus/src/disks/drive.rs)
- `PartitionModel`: mount/unmount/delete (plus placeholders for other operations).
  - Evidence: [storage-dbus/src/disks/partition.rs](../storage-dbus/src/disks/partition.rs)
- `DiskManager`: polling-based device event stream.
  - Evidence: [storage-dbus/src/disks/manager.rs](../storage-dbus/src/disks/manager.rs)
- `usage.rs`: reads filesystem usage from the `df` command.
  - Evidence: [storage-dbus/src/usage.rs](../storage-dbus/src/usage.rs)

## External Dependencies

- COSMIC/libcosmic (UI framework) pulled from git.
  - Evidence: workspace dependency: [Cargo.toml](../Cargo.toml)
- UDisks2 and DBus:
  - `udisks2` crate (high-level DBus bindings)
  - `zbus` + `zbus_macros` for proxies and system bus connection
  - Evidence: dependencies: [Cargo.toml](../Cargo.toml), DBus proxy: [storage-dbus/src/disks/manager.rs](../storage-dbus/src/disks/manager.rs)
- Tokio/futures for async tasks.
  - Evidence: workspace deps: [Cargo.toml](../Cargo.toml)
- Local OS tool: `df` (usage data).
  - Evidence: [storage-dbus/src/usage.rs](../storage-dbus/src/usage.rs)
- System packages called out in docs/CI:
  - `libxkbcommon-dev` (CI build dep): [.github/workflows/ci.yml](../.github/workflows/ci.yml)
  - `ntfs-3g`, `dosfstools` (README prerequisites): [README.md](../README.md)

## Configuration & Secrets

- App config uses `cosmic_config` with the app ID `com.cosmos.Disks`.
  - Evidence: config type: [storage-ui/src/config.rs](../storage-ui/src/config.rs), loading/watching: [storage-ui/src/app.rs](../storage-ui/src/app.rs)
- Secrets are not stored in-repo.
  - CI uses `CARGO_REGISTRY_TOKEN` to publish crates.
  - Evidence: [.github/workflows/main.yml](../.github/workflows/main.yml)

## Runtime & Deployment

- Local run/dev commands via `just`.
  - Evidence: [storage-ui/README.md](../storage-ui/README.md), [storage-ui/justfile](../storage-ui/justfile)
- CI gates on PR; publish pipeline on pushes to `main`.
  - Evidence: [.github/workflows/ci.yml](../.github/workflows/ci.yml), [.github/workflows/main.yml](../.github/workflows/main.yml)
- Releases:
  - Workflow calculates a SemVer version and tags `vX.Y.Z`.
  - Publishes `cosmic-ext-storage-dbus` and `cosmic-ext-disks` to crates.io.
  - Evidence: [.github/workflows/main.yml](../.github/workflows/main.yml)

## Observability

- `storage-dbus` uses `tracing` (errors logged in device polling).
  - Evidence: [storage-dbus/src/disks/manager.rs](../storage-dbus/src/disks/manager.rs)
- UI currently uses `println!/eprintln!` and has tracing init commented out.
  - Evidence: [storage-ui/src/main.rs](../storage-ui/src/main.rs), [storage-ui/src/app.rs](../storage-ui/src/app.rs)

## Security & Compliance Notes

- Disk/partition operations are performed via UDisks2 over the system bus; authorization is typically mediated by the OS/Polkit (exact policy TBD for this app).
  - Evidence: UDisks2 proxies used throughout: [storage-dbus/src/disks/drive.rs](../storage-dbus/src/disks/drive.rs), [storage-dbus/src/disks/partition.rs](../storage-dbus/src/disks/partition.rs)
- The app is explicitly marked early/prototyping and not safe for important disks.
  - Evidence: [README.md](../README.md)
- Compliance constraints: none (per repo rules).
  - Evidence: [.copi/repo-rules.md](repo-rules.md)

## Operational Concerns

- Device change detection is polling-based (1s interval), which is simple but potentially noisy and may miss very fast transitions.
  - Evidence: [storage-dbus/src/disks/manager.rs](../storage-dbus/src/disks/manager.rs)
- A number of UI flows are still `todo!()` or use `unwrap()`/`panic!()` placeholders.
  - Evidence: action handlers: [storage-ui/src/app.rs](../storage-ui/src/app.rs), dialog state handling: [storage-ui/src/views/volumes.rs](../storage-ui/src/views/volumes.rs)
- Usage/mount state currently depends on parsing `df` output, which may diverge from UDisks2’s mount knowledge.
  - Evidence: [storage-dbus/src/usage.rs](../storage-dbus/src/usage.rs), UI check: [storage-ui/src/views/volumes.rs](../storage-ui/src/views/volumes.rs)

## Known Unknowns / TODO Questions

- What is the intended privilege/auth flow (Polkit prompts, error surfaces, safe-guards)?
- Should device change detection move from polling to DBus signal subscriptions (UDisks2 signals)?
- How should errors be reported in the UI (toast/banner) instead of `println!`?
- Several menu actions are defined but unimplemented (`todo!()`): power-off/format/benchmark/etc.
  - Evidence: [storage-ui/src/app.rs](../storage-ui/src/app.rs), [storage-ui/src/views/menu.rs](../storage-ui/src/views/menu.rs)

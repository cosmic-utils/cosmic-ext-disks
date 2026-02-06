# 🚧 Disks 🚧

> [!WARNING]
> This is now stable enough to use for disk operations, however it is still in early development so use with caution!
---
A Disk utility application for the Cosmic Desktop.


### Prerequisites
You will need the following packages/services:
 - `udisks2` (system service; required for device enumeration and events)

### Logging

Logs are written to stdout/stderr and to daily log files.

- Default log directory:
    - `$XDG_STATE_HOME/cosmic-ext-disks/logs/` (if `XDG_STATE_HOME` is set)
    - otherwise `~/.local/state/cosmic-ext-disks/logs/`
- Rotation: daily
- Retention: best-effort cleanup of logs older than 7 days

Environment variables:

- `RUST_LOG`: controls log verbosity (example: `RUST_LOG=debug`)
- `COSMIC_EXT_DISKS_LOG_DIR`: override the log directory
- `COSMIC_EXT_DISKS_LOG_FILE`: override log directory + filename prefix

For partition type support:
 - `ntfs-3g`
 - `exfatprogs`
 - `dosfstools`


### What Works
Most things, although the app is pretty ugly at the moment!

### Upcoming Changes

#### V1 - 99% Feature Parity with Gnome Disks (plus a few extras!)
1. UI - The current UI is a bare minimum to test out functionality... and very ugly.
2. Solve LUKS Creation issues (all other functionality working)
3. Automatic "Resource Busy" resolution on unmount
   - Essentially this will list what processes are holding the mount open, and give you the option to kill them.
5. Implement log files & improve generated events
6. Make sure all UI strings are in i8n for language support
7. Add detection for required packages:
    - Add help text in about pane listing missing deps and what functionality relies on them. 
8. Test Pass (integeration/functional tests)
9. Documentation Pass (Readme and code summaries)
10. Packaging for package managers/flathub 



#### Later
1. Benchmark Disks
2. ATA Drive Settings
4. LVM Support
5. 1st class BTRFS support - Subvolumes CRUD, and snapshotting maybe?


![Screenshot of cosmos-disks](https://github.com/stoorps/cosmos-apps/blob/main/screenshots/cosmos-disks.png)


### Notes on use of AI
AI has been used as a ***tool*** for development of this project, and has not been treated as a self-sufficient engineer.

I have been a professional (employed) software engineer since 2012, and I am very much against AI slop and the existential threat it imposes on our industry.
That being said, when used correctly, I believe it's an invaulable tool for a sole developer on a project as large as this; Especially when money, or the threat of taking somebody's job, isn't on the line.

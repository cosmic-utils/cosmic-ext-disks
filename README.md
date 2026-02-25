# Storage

> [!IMPORTANT]
> By using this software, you fully accept all responsibility for any data loss or corruption caused whilst using this software.

> [!WARNING]
> This software is currently in early beta, and has not been tested against many drive type, partition type, and partition scheme combinations yet. 
---
An All-in-one Storage utility for the Cosmic Desktop.


### Prerequisites
You will need the following packages/services:
 - `udisks2` (system service) - required for device enumeration and events
 - `just` (task runner) - install via `cargo install just` or your package manager
 
For partition type support:
Recommended:
 - `ntfs-3g` / `ntfsprogs` - NTFS Support
 - `exfatprogs` - exFAT Support
 - `dosfstools` - FAT32 Support
 - `rclone` - SMB, FTP, S3, etc. mount support

 Optional: 
 - `xfsprogs` - XFS Support - Untested but "should" work
 - `btrfs-progs` - BTRFS support
 - `f2fs-tools` - F2FS support - Untested but "should" work
 - `udftools` - UDF Support - Untested but "should" work

 No bcachefs support as of yet, but will be coming soon.
 

### Development

**Quick Start:**
```bash
just
```
This single command builds the workspace, installs development policies (D-Bus + Polkit), starts the storage service in the background, and launches the UI.

**Other useful commands:**
```bash
just build              # Build workspace only
just dev                # Build, start service, run UI (stops service on exit)
just service            # Start service attached
just service-bg         # Start service in background only
just app                # Start UI only (assumes service is running)
just stop-service       # Stop the storage service
just test               # Run tests
just clippy             # Run linter
```


### Features

#### v0.1 - ⌛ WIP
- ✅ Feature Parity with Gnome Disks
   - **Deferred until v0.2**: Benchmark Disk/Partition
   - **Deferred until v0.2**: ATA Drive settings
- 🎯 Performance improvements
- 🎯 LVM/Logical container support
- ✅ Detailed Usage tool
- ⌛ BTRFS support - Partial implementation complete.
   - Subvolumes Management
   - Snapshot Management & Scheduling
   - Optional Usage breakdown (requires enablement of quotas)
- ✅ Rclone configuration
   - Setup wizard for common mount types
   - Mount on boot option
   - Supports all providers/types
   - Supports System & User mounts
- ✅ Automatic "Resource Busy" resolution on unmount
   - List processes that are holding the mount open, and give you the option to kill them.
- ⌛ Detection for required packages:
    - rclone detection missing currenty.
- 🎯 Full test of all drive, volume, and mount types. 
- 🎯 Documentation - Docs/Readme/Code comments & summaries
- 🎯 Packaging for package managers/flathub 


#### Later
- Potential move from udisks2.
- Any feature requests welcome!


![Screenshot of Storage App](https://github.com/cosmic-utils/cosmic-ext-storage/blob/main/screenshots/cosmic-ext-storage.png)


### Notes on use of AI
AI has been used as a ***tool*** for development of this project, and has not been treated as a self-sufficient engineer.

I have been a professional software engineer since 2012, and I am very much against AI slop and the existential threat it imposes on our industry.

That being said, I believe when it's used correctly, it is an invaulable tool for a sole developer on a project as large as this; Especially when money, or the threat of taking somebody's job, isn't on the line.

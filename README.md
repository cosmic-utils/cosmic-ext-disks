# Disks

> [!IMPORTANT]
> By using this software, you fully accept all responsibility for any data loss or corruption caused whilst using this software.

> [!WARNING]
> This software is currently in early beta, and has not been tested against many drive type, partition type, and partition scheme combinations yet. 
---
A Disk utility application for the Cosmic Desktop.


### Prerequisites
You will need the following packages/services on a systemd-based installation:
 - `udisks2` (system service; required for device enumeration and events)

For partition type support:
 - `ntfs-3g`
 - `exfatprogs`
 - `dosfstools`


### Upcoming Changes

#### V1 - 99% Feature Parity with Gnome Disks (plus a few extras!)
1. UI - Dialogs are still ugly, and main UI still needs some tweaking. 
2. Testing of as many disk types and partition/disk schemes as possible.
3. 1st class BTRFS support - Subvolumes CRUD, and snapshotting maybe
4. Automatic "Resource Busy" resolution on unmount
   - Essentially this will list what processes are holding the mount open, and give you the option to kill them.
5. Add detection for required packages:
    - Add help text in about pane listing missing deps and what functionality relies on them. 
6. Documentation Pass - Readme and code summaries for contributors/testers
7. Packaging for package managers/flathub 


#### Later
1. Benchmark Disks
2. ATA Drive Settings
4. LVM Support


![Screenshot of cosmos-disks](https://github.com/cosmic-utils/cosmic-ext-disks/blob/main/screenshots/cosmic-ext-disks.png)


### Notes on use of AI
AI has been used as a ***tool*** for development of this project, and has not been treated as a self-sufficient engineer.

I have been a professional (employed) software engineer since 2012, and I am very much against AI slop and the existential threat it imposes on our industry.
That being said, when used correctly, I believe it's an invaulable tool for a sole developer on a project as large as this; Especially when money, or the threat of taking somebody's job, isn't on the line.

// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub(crate) fn is_owned_tree(path: &Path, uid: u32) -> std::io::Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.uid() != uid {
        return Ok(false);
    }

    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if !is_owned_tree(&entry.path(), uid)? {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

pub(crate) fn caller_can_unlink(
    path: &Path,
    uid: u32,
    caller_gids: &[u32],
) -> std::io::Result<bool> {
    let Some(parent) = path.parent() else {
        return Ok(false);
    };

    let metadata = fs::symlink_metadata(parent)?;
    let mode = metadata.mode();

    if metadata.uid() == uid {
        return Ok(mode & 0o300 == 0o300);
    }

    if caller_gids.contains(&metadata.gid()) {
        return Ok(mode & 0o030 == 0o030);
    }

    Ok(mode & 0o003 == 0o003)
}

pub(crate) fn path_requires_admin_delete(
    path: &Path,
    caller_uid: u32,
    caller_gids: &[u32],
) -> bool {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return true,
    };

    if !metadata.is_file() {
        return false;
    }

    if metadata.uid() != caller_uid {
        return true;
    }

    !matches!(caller_can_unlink(path, caller_uid, caller_gids), Ok(true))
}

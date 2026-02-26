// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::{CStr, CString};

pub(crate) fn current_process_groups() -> Vec<u32> {
    let mut gids = vec![unsafe { libc::getegid() } as u32];

    let group_count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if group_count > 0 {
        let mut groups = vec![0 as libc::gid_t; group_count as usize];
        let read_count = unsafe { libc::getgroups(group_count, groups.as_mut_ptr()) };
        if read_count > 0 {
            groups.truncate(read_count as usize);
            gids.extend(groups);
        }
    }

    gids.sort_unstable();
    gids.dedup();
    gids
}

pub(crate) fn resolve_caller_groups(uid: u32, username: Option<&str>) -> Vec<u32> {
    let process_uid = unsafe { libc::geteuid() } as u32;
    if uid == process_uid {
        return current_process_groups();
    }

    let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
    let mut pwd_ptr: *mut libc::passwd = std::ptr::null_mut();
    let mut buffer = vec![0_u8; 4096];

    let lookup_result = unsafe {
        libc::getpwuid_r(
            uid,
            pwd.as_mut_ptr(),
            buffer.as_mut_ptr() as *mut libc::c_char,
            buffer.len(),
            &mut pwd_ptr,
        )
    };

    if lookup_result != 0 || pwd_ptr.is_null() {
        tracing::warn!("Failed to resolve passwd entry for UID {}", uid);
        return Vec::new();
    }

    let passwd = unsafe { pwd.assume_init() };
    let primary_gid = passwd.pw_gid;
    let username_cstr = if let Some(name) = username {
        CString::new(name).ok()
    } else {
        unsafe { Some(CStr::from_ptr(passwd.pw_name).to_owned()) }
    };

    let Some(username_cstr) = username_cstr else {
        tracing::warn!("Failed to construct username for UID {}", uid);
        return vec![primary_gid];
    };

    let mut ngroups = 32_i32;
    let mut groups = vec![0 as libc::gid_t; ngroups as usize];

    let result = unsafe {
        libc::getgrouplist(
            username_cstr.as_ptr(),
            primary_gid,
            groups.as_mut_ptr(),
            &mut ngroups,
        )
    };

    if result == -1 {
        if ngroups <= 0 {
            return vec![primary_gid];
        }

        groups.resize(ngroups as usize, 0);
        let retry = unsafe {
            libc::getgrouplist(
                username_cstr.as_ptr(),
                primary_gid,
                groups.as_mut_ptr(),
                &mut ngroups,
            )
        };

        if retry == -1 {
            return vec![primary_gid];
        }
    }

    groups.truncate(ngroups.max(0) as usize);
    let mut gids: Vec<u32> = groups.into_iter().collect();
    gids.push(primary_gid);
    gids.sort_unstable();
    gids.dedup();
    gids
}

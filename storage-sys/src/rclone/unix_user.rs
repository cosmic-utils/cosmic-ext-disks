use crate::error::{Result, SysError};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub(crate) fn username_for_uid(uid: u32) -> Option<String> {
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            return None;
        }
        let name = std::ffi::CStr::from_ptr((*pw).pw_name);
        name.to_str().ok().map(|name| name.to_string())
    }
}

pub(crate) fn uid_gid_for_uid(uid: u32) -> Option<(u32, u32)> {
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            return None;
        }
        Some(((*pw).pw_uid as u32, (*pw).pw_gid as u32))
    }
}

pub(crate) fn chown_path(path: &Path, uid: u32, gid: u32) -> Result<()> {
    let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|e| {
        SysError::OperationFailed(format!("Invalid path for chown {}: {}", path.display(), e))
    })?;
    let result = unsafe { libc::chown(c_path.as_ptr(), uid as libc::uid_t, gid as libc::gid_t) };
    if result != 0 {
        return Err(SysError::OperationFailed(format!(
            "Failed to chown {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}

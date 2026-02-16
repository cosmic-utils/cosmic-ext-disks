// SPDX-License-Identifier: GPL-3.0-only

//! D-Bus caller identity information
//!
//! This module provides types for tracking the identity of D-Bus method callers,
//! which is essential for proper Polkit authorization and user context passthrough.

use serde::{Deserialize, Serialize};

/// Information about a D-Bus method caller
///
/// This struct is populated by the `#[authorized_interface]` macro from the
/// D-Bus message header and provided to service methods for:
///
/// 1. **Authorization**: The caller's identity is used for Polkit checks
/// 2. **User context passthrough**: The UID/username can be passed to UDisks2
///    for operations like mounting that should run as the calling user
///
/// # Example
///
/// ```rust,ignore
/// #[authorized_interface(action = "org.cosmic.ext.storage-service.mount")]
/// async fn mount(
///     &self,
///     caller: CallerInfo,  // Auto-injected by macro
///     device: String,
///     mount_point: String,
/// ) -> zbus::fdo::Result<String> {
///     // Use caller.uid for UDisks2 passthrough
///     mount_filesystem(&device, &mount_point, Some(caller.uid)).await
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerInfo {
    /// Unix user ID of the calling process
    pub uid: u32,

    /// Username resolved from UID via getpwuid
    ///
    /// This is `None` if the username lookup failed, but the UID is always available.
    pub username: Option<String>,

    /// D-Bus unique bus name of the caller (e.g., ":1.42")
    ///
    /// This is the actual caller's bus name from the message header,
    /// NOT the service's own name (which would be returned by `connection.unique_name()`).
    pub sender: String,
}

impl CallerInfo {
    /// Create a new CallerInfo with the given parameters
    pub fn new(uid: u32, username: Option<String>, sender: String) -> Self {
        Self {
            uid,
            username,
            sender,
        }
    }

    /// Check if this caller is root (UID 0)
    pub fn is_root(&self) -> bool {
        self.uid == 0
    }
}

// SPDX-License-Identifier: GPL-3.0-only

//! Authorization module
//!
//! Primary authorization for D-Bus interface methods is handled by the `#[authorized_interface]`
//! procedural macro from the `storage-macros` crate.
//!
//! See `storage-macros/src/lib.rs` for the macro implementation.
//!
//! The macro:
//! 1. Gets the actual caller's sender from `header.sender()` (NOT `connection.unique_name()`)
//! 2. Looks up the caller's UID and PID via D-Bus
//! 3. Checks Polkit authorization with the correct subject
//! 4. Injects a `caller: CallerInfo` variable into the method body
//!
//! For secondary authorization checks within a method (e.g., checking additional
//! permissions for destructive operations), use `check_authorization()`.

use zbus::Connection;
use zbus_polkit::policykit1::AuthorityProxy;

/// Check D-Bus caller authorization using Polkit
///
/// This is used for secondary authorization checks within methods that need
/// to verify additional permissions (e.g., killing processes during unmount).
///
/// For primary method authorization, use `#[authorized_interface]` macro instead.
pub async fn check_authorization(
    connection: &Connection,
    sender: &str,
    action_id: &str,
) -> Result<bool, zbus::Error> {
    tracing::debug!(
        "Checking authorization for sender={} action={}",
        sender,
        action_id
    );

    // Create authority proxy directly
    let authority = AuthorityProxy::new(connection).await?;

    // Get the sender's process ID from D-Bus
    let dbus_proxy = zbus::fdo::DBusProxy::new(connection).await?;
    let bus_name: zbus::names::BusName = sender
        .try_into()
        .map_err(|e| zbus::Error::Failure(format!("Invalid bus name: {}", e)))?;
    let pid = dbus_proxy.get_connection_unix_process_id(bus_name).await?;

    tracing::debug!("Sender {} has PID {}", sender, pid);

    // Create subject from the caller's process ID
    let subject = zbus_polkit::policykit1::Subject::new_for_owner(
        pid, None, // start_time - None means current process
        None, // pid_fd - None for now
    )
    .map_err(|e| zbus::Error::Failure(format!("Failed to create subject: {}", e)))?;

    // Check authorization with user interaction allowed
    let result = authority
        .check_authorization(
            &subject,
            action_id,
            &std::collections::HashMap::new(),
            zbus_polkit::policykit1::CheckAuthorizationFlags::AllowUserInteraction.into(),
            "",
        )
        .await?;

    tracing::debug!(
        "Authorization result for {}: authorized={}, challenged={}",
        action_id,
        result.is_authorized,
        result.is_challenge
    );

    Ok(result.is_authorized)
}

// SPDX-License-Identifier: GPL-3.0-only

use zbus::Connection;
use zbus_polkit::policykit1::AuthorityProxy;

use crate::error::ServiceError;

/// Check D-Bus caller authorization using Polkit
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

/// Helper to check authorization and return error if denied
pub async fn require_authorization(
    connection: &Connection,
    sender: &str,
    action_id: &str,
) -> Result<(), zbus::fdo::Error> {
    let authorized = check_authorization(connection, sender, action_id)
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {}", e)))?;

    if !authorized {
        return Err(zbus::fdo::Error::AccessDenied(format!(
            "Not authorized for action: {}",
            action_id
        )));
    }

    Ok(())
}

/// Check Polkit authorization for D-Bus interface methods
///
/// This is a simplified version that works with the zbus::interface pattern.
/// The actual authorization behavior (whether to prompt for password) is determined
/// by the Polkit policy file, not by this function.
pub async fn check_polkit_auth(
    connection: &Connection,
    action_id: &str,
) -> Result<(), ServiceError> {
    // Get the caller's sender name from the connection
    let sender = connection
        .unique_name()
        .ok_or_else(|| ServiceError::AuthorizationFailed("No caller name".to_string()))?
        .to_string();

    tracing::debug!(
        "Checking authorization for sender={} action={}",
        sender,
        action_id
    );

    // Create authority proxy
    let authority = AuthorityProxy::new(connection)
        .await
        .map_err(|e| ServiceError::DBus(format!("Failed to connect to Polkit: {e}")))?;

    // Get the sender's process ID
    let dbus_proxy = zbus::fdo::DBusProxy::new(connection)
        .await
        .map_err(|e| ServiceError::DBus(format!("Failed to connect to D-Bus: {e}")))?;

    let bus_name: zbus::names::BusName = sender
        .as_str()
        .try_into()
        .map_err(|e| ServiceError::DBus(format!("Invalid bus name: {e}")))?;

    let pid = dbus_proxy
        .get_connection_unix_process_id(bus_name)
        .await
        .map_err(|e| ServiceError::DBus(format!("Failed to get caller PID: {e}")))?;

    tracing::debug!("Sender {} has PID {}", sender, pid);

    // Create subject from the caller's process ID
    let subject = zbus_polkit::policykit1::Subject::new_for_owner(pid, None, None)
        .map_err(|e| ServiceError::AuthorizationFailed(format!("Failed to create subject: {e}")))?;

    // Check authorization with user interaction allowed
    // The actual prompt behavior is determined by the Polkit policy
    let result = authority
        .check_authorization(
            &subject,
            action_id,
            &std::collections::HashMap::new(),
            zbus_polkit::policykit1::CheckAuthorizationFlags::AllowUserInteraction.into(),
            "",
        )
        .await
        .map_err(|e| ServiceError::DBus(format!("Authorization check failed: {e}")))?;

    tracing::debug!(
        "Authorization result for {}: authorized={}, challenged={}",
        action_id,
        result.is_authorized,
        result.is_challenge
    );

    if !result.is_authorized {
        return Err(ServiceError::AuthorizationFailed(format!(
            "Not authorized for action: {}",
            action_id
        )));
    }

    Ok(())
}

// SPDX-License-Identifier: GPL-3.0-only

use zbus::Connection;
use zbus_polkit::policykit1::AuthorityProxy;

/// Check D-Bus caller authorization using Polkit
pub async fn check_authorization(
    connection: &Connection,
    sender: &str,
    action_id: &str,
) -> Result<bool, zbus::Error> {
    tracing::debug!("Checking authorization for sender={} action={}", sender, action_id);
    
    // Create authority proxy directly
    let authority = AuthorityProxy::new(connection).await?;
    
    // Get the sender's process ID from D-Bus
    let dbus_proxy = zbus::fdo::DBusProxy::new(connection).await?;
    let bus_name: zbus::names::BusName = sender.try_into()
        .map_err(|e| zbus::Error::Failure(format!("Invalid bus name: {}", e)))?;
    let pid = dbus_proxy.get_connection_unix_process_id(bus_name).await?;
    
    tracing::debug!("Sender {} has PID {}", sender, pid);
    
    // Create subject from the caller's process ID
    let subject = zbus_polkit::policykit1::Subject::new_for_owner(
        pid,
        None, // start_time - None means current process
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

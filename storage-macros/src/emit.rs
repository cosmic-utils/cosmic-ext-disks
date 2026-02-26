use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemFn;

pub(crate) fn emit_authorized_method(
    action_id: &str,
    method: &ItemFn,
    connection_name: &str,
    header_name: &str,
) -> TokenStream2 {
    let vis = &method.vis;
    let sig = &method.sig;
    let method_name = &sig.ident;
    let original_block = &method.block;

    let connection_ident: syn::Ident =
        syn::Ident::new(connection_name, proc_macro2::Span::call_site());
    let header_ident: syn::Ident = syn::Ident::new(header_name, proc_macro2::Span::call_site());

    let inputs: Vec<_> = sig.inputs.iter().collect();
    let generics = &sig.generics;
    let output = &sig.output;

    quote! {
        #[allow(clippy::too_many_arguments)]
        #vis async fn #method_name #generics ( #(#inputs),* ) #output {
            let __sender = #header_ident
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            tracing::debug!("Method called by sender: {}", __sender);

            let __dbus_proxy = zbus::fdo::DBusProxy::new(#connection_ident).await
                .map_err(|e| zbus::fdo::Error::Failed(format!("D-Bus connection error: {}", e)))?;

            let __bus_name: zbus::names::BusName = __sender.clone()
                .try_into()
                .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid bus name: {}", e)))?;

            let __caller_uid = __dbus_proxy
                .get_connection_unix_user(__bus_name.clone()).await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get caller UID: {}", e)))?;

            tracing::debug!("Caller {} has UID {}", __sender, __caller_uid);

            let __caller_pid = __dbus_proxy
                .get_connection_unix_process_id(__bus_name).await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get caller PID: {}", e)))?;

            tracing::debug!("Caller {} has PID {}", __sender, __caller_pid);

            let __authority = zbus_polkit::policykit1::AuthorityProxy::new(#connection_ident).await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Polkit connection error: {}", e)))?;

            let __subject = zbus_polkit::policykit1::Subject::new_for_owner(__caller_pid, None, None)
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to create Polkit subject: {}", e)))?;

            let __auth_result = __authority
                .check_authorization(
                    &__subject,
                    #action_id,
                    &std::collections::HashMap::new(),
                    zbus_polkit::policykit1::CheckAuthorizationFlags::AllowUserInteraction.into(),
                    "",
                )
                .await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {}", e)))?;

            tracing::debug!(
                "Authorization result for {}: is_authorized={}, is_challenge={}",
                #action_id,
                __auth_result.is_authorized,
                __auth_result.is_challenge
            );

            if !__auth_result.is_authorized {
                tracing::warn!("Authorization denied for action: {}", #action_id);
                return Err(zbus::fdo::Error::AccessDenied(format!(
                    "Not authorized for action: {}",
                    #action_id
                )));
            }

            tracing::info!("Authorization granted for action: {}", #action_id);

            let __caller_username = unsafe {
                let __pw = libc::getpwuid(__caller_uid);
                if __pw.is_null() {
                    tracing::warn!("Failed to resolve username for UID {}", __caller_uid);
                    None
                } else {
                    std::ffi::CStr::from_ptr((*__pw).pw_name)
                        .to_str()
                        .ok()
                        .map(|s| s.to_string())
                }
            };

            let caller = storage_types::CallerInfo::new(
                __caller_uid,
                __caller_username,
                __sender,
            );

            #original_block
        }
    }
}

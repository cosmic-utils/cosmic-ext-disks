// SPDX-License-Identifier: GPL-3.0-only

//! Procedural macros for storage-service D-Bus authorization
//!
//! This crate provides the `#[authorized_interface]` macro that adds Polkit authorization
//! checking to D-Bus interface methods.
//!
//! # Usage
//!
//! Apply `#[authorized_interface(action = "...")]` to methods inside a `#[zbus::interface]` impl block.
//! The method MUST have `#[zbus(connection)]` and `#[zbus(header)]` parameters:
//!
//! ```rust,ignore
//! #[interface(name = "org.cosmic.ext.StorageService.Filesystems")]
//! impl FilesystemsHandler {
//!     #[authorized_interface(action = "org.cosmic.ext.storage-service.mount")]
//!     async fn mount(
//!         &self,
//!         #[zbus(connection)] connection: &Connection,
//!         #[zbus(header)] header: MessageHeader<'_>,
//!         #[zbus(signal_context)] signal_ctx: SignalEmitter<'_>,
//!         device: String,
//!     ) -> zbus::fdo::Result<String> {
//!         // Authorization already checked, use `caller.uid` or `caller.username`
//!         tracing::info!("Mounting as UID {}", caller.uid);
//!     }
//! }
//! ```
//!
//! The macro will:
//! 1. Check Polkit authorization against the actual caller
//! 2. Create a `caller: CallerInfo` variable with the caller's uid, username, and sender
//! 3. Execute the original method body

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, ItemFn, FnArg, Pat, parse::Parse, parse::ParseStream, Token, LitStr, Result as SynResult,
};

/// Arguments for the authorized_interface attribute
struct AuthorizedInterfaceArgs {
    action: String,
}

impl Parse for AuthorizedInterfaceArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut action = "org.cosmic.ext.storage-service.default".to_string();

        // Parse: action = "..."
        if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            if ident == "action" {
                let _: Token![=] = input.parse()?;
                let lit: LitStr = input.parse()?;
                action = lit.value();
            }
        }

        Ok(AuthorizedInterfaceArgs { action })
    }
}

/// Attribute macro for D-Bus interface methods that require Polkit authorization.
///
/// This macro injects authorization checking code at the beginning of the method body.
/// The method MUST have `#[zbus(connection)]` and `#[zbus(header)]` parameters.
///
/// The macro creates a `caller: CallerInfo` variable that can be used in the method body.
#[proc_macro_attribute]
pub fn authorized_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AuthorizedInterfaceArgs);
    let method = parse_macro_input!(item as ItemFn);

    let expanded = transform_method(&args.action, &method);

    expanded.into()
}

/// Transform a method to add authorization checking at the beginning of the body
fn transform_method(action_id: &str, method: &ItemFn) -> TokenStream2 {
    let vis = &method.vis;
    let sig = &method.sig;
    let method_name = &sig.ident;
    let is_async = sig.asyncness.is_some();
    let _return_type = &sig.output;

    // Check if the method has connection and header parameters
    // We look for parameters with #[zbus(connection)] and #[zbus(header)] attributes
    // OR parameters named "connection" or "header" (with optional underscore prefix)
    let mut has_connection = false;
    let mut has_header = false;
    let mut connection_name = String::new();
    let mut header_name = String::new();

    for arg in &sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                let param_name = pat_ident.ident.to_string();

                // Check if this parameter has a zbus attribute
                let _has_zbus_attr = pat_type.attrs.iter().any(|attr| {
                    attr.path().segments.iter().any(|seg| seg.ident == "zbus")
                });

                // Check the attribute content for connection/header
                let mut found_connection_attr = false;
                let mut found_header_attr = false;

                for attr in &pat_type.attrs {
                    if let syn::Meta::List(list) = &attr.meta {
                        let tokens_str = list.tokens.to_string();
                        if tokens_str.contains("connection") {
                            found_connection_attr = true;
                        }
                        if tokens_str.contains("header") {
                            found_header_attr = true;
                        }
                    }
                }

                // Check by attribute content
                if found_connection_attr {
                    has_connection = true;
                    connection_name = param_name.clone();
                }
                if found_header_attr {
                    has_header = true;
                    header_name = param_name.clone();
                }

                // Also check by parameter name convention (with underscore prefix)
                if param_name == "connection" || param_name == "_connection" {
                    has_connection = true;
                    connection_name = param_name.clone();
                }
                if param_name == "header" || param_name == "_header" {
                    has_header = true;
                    header_name = param_name.clone();
                }
            }
        }
    }

    // Get the original method body
    let original_block = &method.block;

    // Check if async
    if !is_async {
        // For non-async methods, just return as-is with a compile error
        return quote! {
            compile_error!("#[authorized_interface] only supports async methods");
        };
    }

    if !has_connection || !has_header {
        return quote! {
            compile_error!("#[authorized_interface] requires method to have #[zbus(connection)] and #[zbus(header)] parameters");
        };
    }

    let connection_ident: syn::Ident = syn::Ident::new(&connection_name, proc_macro2::Span::call_site());
    let header_ident: syn::Ident = syn::Ident::new(&header_name, proc_macro2::Span::call_site());

    // Collect the inputs into a vec for quote
    let inputs: Vec<_> = sig.inputs.iter().collect();

    // Get generics from signature (if any)
    let generics = &sig.generics;
    let output = &sig.output;

    // Generate the transformed method
    // We keep the same parameters but inject authorization code at the beginning of the body
    quote! {
        #vis async fn #method_name #generics ( #(#inputs),* ) #output {
            // === Step 1: Get the actual sender from the message header ===
            let __sender = #header_ident
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            tracing::debug!("Method called by sender: {}", __sender);

            // === Step 2: Look up the caller's UID from D-Bus ===
            let __dbus_proxy = zbus::fdo::DBusProxy::new(#connection_ident).await
                .map_err(|e| zbus::fdo::Error::Failed(format!("D-Bus connection error: {}", e)))?;

            let __bus_name: zbus::names::BusName = __sender.clone()
                .try_into()
                .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid bus name: {}", e)))?;

            let __caller_uid = __dbus_proxy
                .get_connection_unix_user(__bus_name.clone()).await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get caller UID: {}", e)))?;

            tracing::debug!("Caller {} has UID {}", __sender, __caller_uid);

            // === Step 3: Get caller PID for Polkit subject ===
            let __caller_pid = __dbus_proxy
                .get_connection_unix_process_id(__bus_name).await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get caller PID: {}", e)))?;

            tracing::debug!("Caller {} has PID {}", __sender, __caller_pid);

            // === Step 4: Check Polkit authorization with correct subject ===
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

            // === Step 5: Resolve username from UID ===
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

            // === Step 6: Create CallerInfo for method body ===
            let caller = storage_common::CallerInfo::new(
                __caller_uid,
                __caller_username,
                __sender,
            );

            // === Step 7: Execute the original method body ===
            #original_block
        }
    }
}

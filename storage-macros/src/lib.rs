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
//! #[interface(name = "org.cosmic.ext.Storage.Service.Filesystems")]
//! impl FilesystemsHandler {
//!     #[authorized_interface(action = "org.cosmic.ext.storage.service.mount")]
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

mod emit;
mod parse;
mod transform;

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use parse::AuthorizedInterfaceArgs;
use transform::transform_method;

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

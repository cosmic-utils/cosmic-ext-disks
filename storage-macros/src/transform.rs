use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{FnArg, ItemFn, Pat};

use crate::emit::emit_authorized_method;

pub(crate) fn transform_method(action_id: &str, method: &ItemFn) -> TokenStream2 {
    let sig = &method.sig;
    let is_async = sig.asyncness.is_some();

    let mut has_connection = false;
    let mut has_header = false;
    let mut connection_name = String::new();
    let mut header_name = String::new();

    for arg in &sig.inputs {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = pat_type.pat.as_ref()
        {
            let param_name = pat_ident.ident.to_string();

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

            if found_connection_attr {
                has_connection = true;
                connection_name = param_name.clone();
            }
            if found_header_attr {
                has_header = true;
                header_name = param_name.clone();
            }

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

    if !is_async {
        return quote! {
            compile_error!("#[authorized_interface] only supports async methods");
        };
    }

    if !has_connection || !has_header {
        return quote! {
            compile_error!("#[authorized_interface] requires method to have #[zbus(connection)] and #[zbus(header)] parameters");
        };
    }

    emit_authorized_method(action_id, method, &connection_name, &header_name)
}

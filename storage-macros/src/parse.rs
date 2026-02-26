use syn::{LitStr, Result as SynResult, Token, parse::Parse, parse::ParseStream};

pub(crate) struct AuthorizedInterfaceArgs {
    pub(crate) action: String,
}

impl Parse for AuthorizedInterfaceArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut action = "org.cosmic.ext.storage.service.default".to_string();

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

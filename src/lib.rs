use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Ident};
use heck::*;

extern crate proc_macro;

#[proc_macro]
pub fn able(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(item as Able);
    let t = quote!(#parsed);
    //print!("{}", t);
    proc_macro::TokenStream::from(t)
}

/// Able.
pub(crate) struct Able {
    ident: Ident,
}

impl Parse for Able {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            ident: input.parse()?,
        })
    }
}

impl ToTokens for Able {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident.to_string().to_camel_case();
        let ident_fn = Ident::new(&format!("{}", ident).to_snake_case(), Span::call_site());
        let ident_able = Ident::new(&format!("{}able", ident).to_camel_case(), Span::call_site());
        let on_ident = Ident::new(&format!("on_{}", ident).to_snake_case(), Span::call_site());
        let as_ident_able = Ident::new(
            &format!("as_{}", ident_able).to_snake_case(),
            Span::call_site(),
        );
        let as_ident_able_mut = Ident::new(
            &format!("as_{}_mut", ident_able).to_snake_case(),
            Span::call_site(),
        );
        let into_ident_able = Ident::new(
            &format!("into_{}", ident_able).to_snake_case(),
            Span::call_site(),
        );
        let expr = quote! {
            pub trait #ident_able {
                fn #ident_fn(&mut self, skip_callbacks: bool);
                fn #on_ident(&mut self, callback: Option<Box<FnMut(&mut #ident_able)>>);

                fn #as_ident_able(&self) -> &dyn #ident_able;
                fn #as_ident_able_mut(&mut self) -> &mut dyn #ident_able;
                fn #into_ident_able(self: Box<Self>) -> Box<dyn #ident_able>;
            }
        };
        expr.to_tokens(tokens);
    }
}

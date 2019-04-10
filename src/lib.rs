use heck::*;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse_macro_input, token, Ident, Token, TypeParam, Type};

extern crate proc_macro;

#[proc_macro]
pub fn able(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(item as Able);
    let t = quote!(#parsed);
    dbg!(format!("{:#}", t));
    proc_macro::TokenStream::from(t)
}

/// Able.
pub(crate) struct Able {
    name: Ident,
    _paren: Option<token::Paren>,
    params: Option<Punctuated<Type, Token![,]>>,
    _colon: Option<Token![:]>,
    extends: Option<Punctuated<TypeParam, Token![+]>>,
}

impl Parse for Able {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut extends_present = false;
        let mut params = None;
        Ok(Self {
            name: input.parse()?,
            _paren: {
                let lookahead = input.lookahead1();
                if lookahead.peek(token::Paren) {
                    let content;
                    let paren = parenthesized!(content in input);
                    params = Some(content);
                    Some(paren)
                } else {
                    None
                }
            },
            params: params.map(|content| content.parse_terminated(Type::parse).unwrap()),
            _colon: {
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![:]) {
                    extends_present = true;
                    Some(input.parse()?)
                } else {
                    None
                }
            },
            extends: if extends_present {
                Some(input.parse_terminated(TypeParam::parse).unwrap())
            } else {
                None
            },
        })
    }
}

impl ToTokens for Able {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.name.to_string().to_camel_case();
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
        let params = self
            .params
            .as_ref()
            .map(|punct| punct.iter().map(|i| i.clone()).collect::<Vec<_>>())
            .unwrap_or(vec![]);
        let params2 = params.clone();
        let param_names = (0..params.len())
            .map(|i| Ident::new(&format!("arg{}", i), Span::call_site()))
            .collect::<Vec<_>>();
            
        let extends = self.extends.as_ref()
            .map(|punct| punct.iter().map(|i| i.clone()).collect::<Vec<_>>())
            .unwrap_or(vec![]);

        let expr = if extends.len() > 0 {
	        quote! {
	            pub trait #ident_able: #(#extends)+* {
	                fn #ident_fn(&mut self, #(#param_names: #params,)* skip_callbacks: bool);
	                fn #on_ident(&mut self, callback: Option<Box<FnMut(&mut #ident_able #(,#params2)* )>>);
	
	                fn #as_ident_able(&self) -> &dyn #ident_able;
	                fn #as_ident_able_mut(&mut self) -> &mut dyn #ident_able;
	                fn #into_ident_able(self: Box<Self>) -> Box<dyn #ident_able>;
	            }
	        }
        } else {
	        quote! {
	            pub trait #ident_able {
	                fn #ident_fn(&mut self, #(#param_names: #params,)* skip_callbacks: bool);
	                fn #on_ident(&mut self, callback: Option<Box<FnMut(&mut #ident_able #(,#params2)* )>>);
	
	                fn #as_ident_able(&self) -> &dyn #ident_able;
	                fn #as_ident_able_mut(&mut self) -> &mut dyn #ident_able;
	                fn #into_ident_able(self: Box<Self>) -> Box<dyn #ident_able>;
	            }
	        }
        };
        expr.to_tokens(tokens);
    }
}

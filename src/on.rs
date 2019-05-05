use heck::*;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse_macro_input, token, Ident, Token, Type};

pub fn make(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(item as On);
    let t = quote!(#parsed);
    dbg!(format!("{:#}", t));
    proc_macro::TokenStream::from(t)
}

pub enum OnReturnParams {
    None,
    Single(token::RArrow, Type),
    Multi(token::RArrow, token::Paren, Punctuated<Type, Token![,]>),
}

pub(crate) struct On {
    pub name: Ident,
    pub paren: token::Paren,
    pub params: Punctuated<Type, Token![,]>,
    pub ret: OnReturnParams,
}

impl Parse for On {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            name: input.parse()?,
            paren: parenthesized!(content in input),
            params: content.parse_terminated(Type::parse)?,
            ret: {
                let lookahead = input.lookahead1();
                if lookahead.peek(token::RArrow) {
                    let arrow = input.parse()?;
                    let lookahead = input.lookahead1();
                    if lookahead.peek(token::Paren) {
                        let content;
                        OnReturnParams::Multi(arrow, parenthesized!(content in input), content.parse_terminated(Type::parse)?)
                    } else {
                        OnReturnParams::Single(arrow, input.parse()?)
                    }
                } else {
                    OnReturnParams::None
                }
            }
        })
    }
}

impl ToTokens for On {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.name.to_string().to_camel_case();
        let params = &self.params;

        let on_ident = Ident::new(&format!("On{}", ident).to_camel_case(), Span::call_site());

        let expr = quote! {
            pub struct #on_ident(CallbackId, Box<dyn FnMut(#(#params,)* )>);

            impl Callback for #on_ident {
                fn name(&self) -> &'static str {
                    stringify!(#on_ident)
                }
                fn id(&self) -> CallbackId {
                    self.0
                }
            }

            impl <T> From<T> for #on_ident where T: FnMut(#(#params,)*) + Sized + 'static {
                fn from(t: T) -> #on_ident {
                    #on_ident(CallbackId::next(), Box::new(t))
                }
            }
            impl AsRef<dyn FnMut(#(#params,)*)> for #on_ident {
                fn as_ref(&self) -> &(dyn FnMut(#(#params,)*)  + 'static) {
                    self.1.as_ref()
                }
            }
            impl AsMut<dyn FnMut(#(#params,)*)> for #on_ident {
                fn as_mut(&mut self) -> &mut (dyn FnMut(#(#params,)*) + 'static) {
                    self.1.as_mut()
                }
            }
            impl From<#on_ident> for (CallbackId, Box<dyn FnMut(#(#params,)*)>) {
                fn from(a: #on_ident) -> Self {
                    (a.0, a.1)
                }
            }
            impl From<(CallbackId, Box<dyn FnMut(#(#params,)*)>)> for #on_ident {
                fn from(a: (CallbackId, Box<dyn FnMut(#(#params,)*)>)) -> Self {
                    #on_ident(a.0, a.1)
                }
            }

            impl ::std::fmt::Debug for #on_ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    write!(f, "{}({})", self.name(), self.id())
                }
            }
            impl ::std::cmp::PartialEq for #on_ident {
                fn eq(&self, other: &#on_ident) -> bool {
                    self.id().eq(&other.id())
                }
            }
        };
        expr.to_tokens(tokens)
    }
}

use heck::*;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{Ident, Type};

pub struct On<'a> {
    pub ident_camel: &'a Ident,
    pub ident_owner_camel: &'a Ident,
    pub params: &'a Vec<Type>,
}

impl<'a> ToTokens for On<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident_camel;
        let ident_owner = &self.ident_owner_camel;
        let params = self.params;

        let on_ident = Ident::new(&format!("On{}", ident).to_camel_case(), Span::call_site());

        let expr = quote! {
            pub struct #on_ident(CallbackId, Box<dyn FnMut(&mut dyn #ident_owner #(,#params)* )>);

            impl Callback for #on_ident {
                fn name(&self) -> &'static str {
                    stringify!(#on_ident)
                }
                fn id(&self) -> CallbackId {
                    self.0
                }
            }

            impl <T> From<T> for #on_ident where T: FnMut(&mut dyn #ident_owner #(,#params)*) + Sized + 'static {
                fn from(t: T) -> #on_ident {
                    #on_ident(CallbackId::next(), Box::new(t))
                }
            }
            impl AsRef<dyn FnMut(&mut dyn #ident_owner #(,#params)*)> for #on_ident {
                fn as_ref(&self) -> &(dyn FnMut(&mut dyn #ident_owner #(,#params)*)  + 'static) {
                    self.1.as_ref()
                }
            }
            impl AsMut<dyn FnMut(&mut dyn #ident_owner #(,#params)*)> for #on_ident {
                fn as_mut(&mut self) -> &mut (dyn FnMut(&mut dyn #ident_owner #(,#params)*) + 'static) {
                    self.1.as_mut()
                }
            }
            impl From<#on_ident> for (CallbackId, Box<dyn FnMut(&mut dyn #ident_owner #(,#params)*)>) {
                fn from(a: #on_ident) -> Self {
                    (a.0, a.1)
                }
            }
            impl From<(CallbackId, Box<dyn FnMut(&mut dyn #ident_owner #(,#params)*)>)> for #on_ident {
                fn from(a: (CallbackId, Box<dyn FnMut(&mut dyn #ident_owner #(,#params)*)>)) -> Self {
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

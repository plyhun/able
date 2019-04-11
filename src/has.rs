use heck::*;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse_macro_input, token, Ident, Token, Type, Lifetime};

pub fn make(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(item as Has);
    let t = quote!(#parsed);
    dbg!(format!("{:#}", t));
    proc_macro::TokenStream::from(t)
}

pub(crate) struct Has {
    name: Ident,
    _paren: Option<token::Paren>,
    params: Option<Punctuated<Type, Token![,]>>,
    _colon: Option<Token![:]>,
    extends: Option<Punctuated<Ident, Token![+]>>,
}

impl Parse for Has {
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
                Some(input.parse_terminated(Ident::parse).unwrap())
            } else {
                None
            },
        })
    }
}

impl ToTokens for Has {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.name.to_string().to_camel_case();
        
        let ident_has = Ident::new(&format!("Has{}", ident).to_camel_case(), Span::call_site());
        let ident_has_inner = Ident::new(&format!("Has{}Inner", ident).to_camel_case(), Span::call_site());
        let on_ident = Ident::new(&format!("On{}", ident).to_camel_case(), Span::call_site());
        
        let ident_fn = Ident::new(&format!("{}", ident).to_snake_case(), Span::call_site());
        let set_ident_fn = Ident::new(&format!("set_{}", ident).to_snake_case(), Span::call_site());
        let on_ident_fn = Ident::new(&format!("on_{}", ident).to_snake_case(), Span::call_site());
        
        let oopify = &crate::oopify::Oopify {
	        ident: ident_has.clone()
        };
        
        let params = &self
            .params
            .as_ref()
            .map(|punct| punct.iter().map(|i| i.clone()).collect::<Vec<_>>())
            .unwrap_or(vec![]);
        let param_names = &(0..params.len())
            .map(|i| Ident::new(&format!("arg{}", i), Span::call_site()))
            .collect::<Vec<_>>();
            
        let extends = self.extends.as_ref()
            .map(|punct| punct.iter().map(|i| i.clone()).collect::<Vec<_>>())
            .unwrap_or(vec![]);
        let extends_inner = self.extends.as_ref()
            .map(|punct| punct.iter().map(|i| Ident::new(&format!("{}Inner", i.to_string().to_camel_case()), Span::call_site())).collect::<Vec<_>>())
            .unwrap_or(vec![]);   
            
        let static_ = Lifetime::new("'static", Span::call_site());  
        let static_inner = Lifetime::new("'static", Span::call_site());   
        
        let expr = quote! {
            pub trait #ident_has: AsAny + #static_ #(+#extends)* {
                fn #ident_fn(&mut self) -> #(#params),* ;
                fn #set_ident_fn(&mut self #(,#param_names: #params)*);
                fn #on_ident_fn(&mut self, callback: Option<#on_ident>);

                #oopify
            }
            pub trait #ident_has_inner: #static_inner #(+#extends_inner)* {
                fn #ident_fn(&mut self) -> #(#params),* ;
                fn #set_ident_fn(&mut self #(,#param_names: #params,)*);
                fn #on_ident_fn(&mut self, callback: Option<Box<FnMut(&mut dyn #ident_has #(,#params)* )>>);
            }
            
            pub struct #on_ident(CallbackId, Box<dyn FnMut(&mut dyn #ident_has #(,#params)* )>);

			impl Callback for #on_ident {
				fn name(&self) -> &'static str {
					stringify!(#on_ident)
				}
				fn id(&self) -> CallbackId {
					self.0
				}
			}
	
			impl <T> From<T> for #on_ident where T: FnMut(&mut dyn #ident_has #(,#params)*) + Sized + 'static {
				fn from(t: T) -> #on_ident {
					#on_ident(CallbackId::next(), Box::new(t))
				}
			}
			impl AsRef<dyn FnMut(&mut dyn #ident_has #(,#params)*)> for #on_ident {
				fn as_ref(&self) -> &(dyn FnMut(&mut dyn #ident_has #(,#params)*)  + 'static) {
					self.1.as_ref()
				}
			}
			impl AsMut<dyn FnMut(&mut dyn #ident_has #(,#params)*)> for #on_ident {
				fn as_mut(&mut self) -> &mut (dyn FnMut(&mut dyn #ident_has #(,#params)*) + 'static) {
					self.1.as_mut()
				}
			}
			impl From<#on_ident> for (CallbackId, Box<dyn FnMut(&mut dyn #ident_has #(,#params)*)>) {
			    fn from(a: #on_ident) -> Self {
			        (a.0, a.1)
			    }
			}
			impl From<(CallbackId, Box<dyn FnMut(&mut dyn #ident_has #(,#params)*)>)> for #on_ident {
			    fn from(a: (CallbackId, Box<dyn FnMut(&mut dyn #ident_has #(,#params)*)>)) -> Self {
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
        expr.to_tokens(tokens);
    }
}

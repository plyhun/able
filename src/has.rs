use heck::*;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, parse_macro_input, token, Ident, Lifetime, Token, Type};

pub fn make(item: proc_macro::TokenStream, use_reactor: bool) -> proc_macro::TokenStream {
    let mut parsed = parse_macro_input!(item as Has);
    if use_reactor {
        parsed.inner = InnerType::Reactor;
    }
    let t = quote!(#parsed);
    dbg!(format!("{:#}", t));
    proc_macro::TokenStream::from(t)
}

struct HasReturnParams<'a> {
    params: &'a Punctuated<Type, Token![,]>,
    paren: token::Paren,
}
impl <'a> ToTokens for HasReturnParams<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.params.len() > 1 {
            self.paren.surround(tokens, |tokens| self.params.to_tokens(tokens));
        } else {
            self.params.to_tokens(tokens)
        }
    }
}

enum InnerType {
    GetterSetter,
    Reactor,
}

pub(crate) struct Has {
    name: Ident,
    _paren: token::Paren,
    params: Punctuated<Type, Token![,]>,
    _colon: Option<Token![:]>,
    extends: Option<Punctuated<Ident, Token![+]>>,
    _brace: Option<token::Brace>,
    custom: Option<proc_macro2::TokenStream>,
    inner: InnerType,
}

impl Parse for Has {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut extends_present = false;
        let content;
        let mut custom = None;
        Ok(Self {
            name: input.parse()?,
            _paren: parenthesized!(content in input),
            params: content.parse_terminated(Type::parse)?,
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
                let mut extends: Punctuated<Ident, Token![+]> = Punctuated::new();
                loop {
                    extends.push_value(input.parse()?);
                    if input.peek(token::Brace) || input.is_empty() {
                        break;
                    }
                    extends.push_punct(input.parse()?);
                    if input.peek(token::Brace) || input.is_empty() {
                        break;
                    }
                }
                Some(extends)
            } else {
                None
            },
            _brace: {
                let lookahead = input.lookahead1();
                if lookahead.peek(token::Brace) {
                    let content;
                    let brace = braced!(content in input);
                    custom = Some(content);
                    Some(brace)
                } else {
                    None
                }
            },
            custom: custom.map(|content| content.parse().unwrap()),
            inner: InnerType::GetterSetter,
        })
    }
}

impl ToTokens for Has {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.name.to_string().to_camel_case();

        let has_ident = Ident::new(&format!("Has{}", ident).to_camel_case(), Span::call_site());
        let has_ident_inner = Ident::new(
            &format!("Has{}Inner", ident).to_camel_case(),
            Span::call_site(),
        );

        let ident_fn = Ident::new(&format!("{}", ident).to_snake_case(), Span::call_site());
        let set_ident_fn = Ident::new(&format!("set_{}", ident).to_snake_case(), Span::call_site());
        let on_ident_fn = Ident::new(&format!("on_{}", ident).to_snake_case(), Span::call_site());
        let on_ident_set_fn = Ident::new(&format!("on_{}_set", ident).to_snake_case(), Span::call_site());

        let as_into = &crate::as_into::AsInto {
            ident_camel: &has_ident,
        };

        let params = &self.params;
        let param_names = &(0..params.len())
            .map(|i| Ident::new(&format!("arg{}", i), Span::call_site()))
            .collect::<Vec<_>>();
        let return_params = HasReturnParams {
            params: &self.params,
            paren: token::Paren { span: Span::call_site() },
        };    

        let extends = self
            .extends
            .as_ref()
            .map(|punct| punct.iter().map(|i| i.clone()).collect::<Vec<_>>())
            .unwrap_or(vec![]);
        let extends_inner = self
            .extends
            .as_ref()
            .map(|punct| {
                punct
                    .iter()
                    .map(|i| {
                        Ident::new(
                            &format!("{}Inner", i.to_string().to_camel_case()),
                            Span::call_site(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or(vec![]);

        let static_ = Lifetime::new("'static", Span::call_site());
        let static_inner = Lifetime::new("'static", Span::call_site());
        
        let on_ident = Ident::new(&ident.to_camel_case(), Span::call_site());
        let on = &crate::on::On {
            ident_camel: &on_ident,
            ident_owner_camel: &has_ident,
            params: Some(params),
        };
		let on_ident = Ident::new(&format!("On{}", ident).to_camel_case(), Span::call_site());

        let custom = &self.custom;
        
        let inner = match self.inner {
            InnerType::GetterSetter => quote!{
                fn #ident_fn(&self) -> #return_params ;
                fn #set_ident_fn(&mut self #(,#param_names: #params)*);
            },
            InnerType::Reactor => quote!{
                fn #on_ident_set_fn(&mut self, base: &mut MemberBase, value: #return_params) -> bool;
            },
        };

        let expr = quote! {
            pub trait #has_ident: AsAny + #static_ #(+#extends)* {
                fn #ident_fn(&self) -> #return_params ;
                fn #set_ident_fn(&mut self #(,#param_names: #params)*);
                fn #on_ident_fn(&mut self, callback: Option<#on_ident>);

                #custom
                #as_into
            }
            pub trait #has_ident_inner: #static_inner #(+#extends_inner)* {
                #inner
                #custom
            }

            #on
        };
        expr.to_tokens(tokens);
    }
}

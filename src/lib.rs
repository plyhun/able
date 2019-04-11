#![recursion_limit = "1024"]

extern crate proc_macro;

mod able;
mod as_into;
mod has;
mod maybe;
mod on;

#[proc_macro]
pub fn able(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    able::make(item)
}

#[proc_macro]
pub fn has(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    has::make(item)
}

#[proc_macro]
pub fn maybe(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    maybe::make(item)
}

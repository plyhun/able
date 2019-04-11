#![recursion_limit="1024"]

extern crate proc_macro;

mod able;
mod has;
mod oopify;

#[proc_macro]
pub fn able(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    able::make(item)
}

#[proc_macro]
pub fn has(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    has::make(item)
}
extern crate proc_macro;

mod able;
mod able_inner;

#[proc_macro]
pub fn able(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    able::make(item)
}
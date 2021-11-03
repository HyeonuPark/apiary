extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro_error::{abort_if_dirty, proc_macro_error};

mod attr_apiary;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn api(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let item = syn::parse_macro_input!(item as syn::ItemTrait);
    let res = attr_apiary::process(args, item);
    abort_if_dirty();
    res.unwrap().into()
}

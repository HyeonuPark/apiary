extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro_error::{abort_if_dirty, proc_macro_error};

mod attr_apiary;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn api(param: TokenStream, item: TokenStream) -> TokenStream {
    let res = attr_apiary::process(param.into(), item.into());
    abort_if_dirty();
    res.unwrap().into()
}

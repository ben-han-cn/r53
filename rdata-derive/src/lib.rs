extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
mod parser;

#[proc_macro_derive(Rdata, attributes(field))]
pub fn derive_rdata(input: TokenStream) -> TokenStream {
    let node = parse_macro_input!(input as DeriveInput);
    expand::derive(&node)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

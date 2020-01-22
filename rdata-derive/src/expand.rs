use crate::parser::RdataStruct;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Result};

pub fn derive<'a>(node: &'a DeriveInput) -> Result<TokenStream> {
    let rdata = RdataStruct::parse_token(node)?;
    let name = rdata.name;
    let from_wire = derive_from_wire(&rdata)?;
    let to_wire = derive_to_wire(&rdata)?;
    let from_str = derive_from_str(&rdata)?;
    Ok(quote! {
        impl #name {
            #from_wire

            #to_wire

            #from_str
        }
    })
}

pub fn derive_from_wire<'a>(rdata: &RdataStruct<'a>) -> Result<TokenStream> {
    let field_from_wire = rdata.fields.iter().map(|field| {
        let name = field.name;
        let from_wire_func = Ident::new(&format!("{}_from_wire", field.codec), field.name.span());
        quote! {
            let (#name, len) = #from_wire_func(buf, len)?;
        }
    });

    let field_assignment = rdata.fields.iter().map(|field| {
        let name = field.name;
        quote! {
            #name: #name,
        }
    });
    let name = rdata.name;
    Ok(quote! {
            pub fn from_wire(buf: &mut InputBuffer, len: u16) -> Result<Self> {
                #(#field_from_wire)*
                ensure!(len == 0, "has extra rdata");
                Ok(#name{
                #(#field_assignment)*
                })
            }
    })
}

pub fn derive_to_wire<'a>(rdata: &RdataStruct<'a>) -> Result<TokenStream> {
    let field_to_wire = rdata.fields.iter().map(|field| {
        let name = field.name;
        let to_wire_func = Ident::new(&format!("{}_to_wire", field.codec), field.name.span());
        match field.codec.as_ref() {
            "name" | "name_uncompressed" | "text" | "byte_binary" | "binary" => {
                quote! {
                    #to_wire_func(render, &self.#name);
                }
            }
            _ => {
                quote! {
                    #to_wire_func(render, self.#name);
                }
            }
        }
    });

    Ok(quote! {
            pub fn to_wire(&self, render: &mut MessageRender) {
                #(#field_to_wire)*
            }
    })
}

pub fn derive_from_str<'a>(rdata: &RdataStruct<'a>) -> Result<TokenStream> {
    let field_assignment = rdata.fields.iter().map(|field| {
        let name = field.name;
        let from_str_func = Ident::new(&format!("{}_from_str", field.display), field.name.span());
        quote! {
            #name: #from_str_func(buf)?,
        }
    });
    let name = rdata.name;
    Ok(quote! {
            pub fn from_str(buf: &mut StringBuffer) -> Result<Self> {
                Ok(#name{
                #(#field_assignment)*
                })
            }
    })
}

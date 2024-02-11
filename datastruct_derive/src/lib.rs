mod cmp;
mod config;
mod generate;
mod syntax;
mod utils;
mod ops;

use crate::generate::RichStructContent;
use proc_macro::TokenStream;
use std::io::Write;
use syn::parse_macro_input;


#[proc_macro_derive(DataStruct, attributes(dstruct, dfield))]
pub fn datastruct(input: TokenStream) -> TokenStream {
    // let uinput = input.clone();
    let mut file =
        std::fs::File::create(r"D:\WBH\rust\datastruct\macro_output\raw.output").unwrap();
    file.write_fmt(format_args!("{}", input)).unwrap();
    let parsed = parse_macro_input!(input with syntax::RichStruct::parse_struct);
    let expanded = match RichStructContent::from_syntax(parsed) {
        Err(e) => e.to_compile_error(),
        Ok(content) => content
            .to_impl()
            .unwrap_or_else(syn::Error::into_compile_error),
    };

    let mut file =
        std::fs::File::create(r"D:\WBH\rust\datastruct\macro_output\derive.output").unwrap();
    file.write_fmt(format_args!("{}", expanded)).unwrap();

    expanded.into()
}

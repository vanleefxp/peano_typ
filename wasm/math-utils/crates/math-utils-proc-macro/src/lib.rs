extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Pat, parse_macro_input};

#[proc_macro]
pub fn define_func(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DefineFuncInput);

    let func_name = input.func_name;
    let closure = input.closure;

    let arg_types = get_arg_types(&closure);
    let n_args = arg_types.len();

    let arg_declarations = (0..n_args).map(|i| {
        let arg_name = Ident::new(&format!("arg{}", i), Span::call_site());
        quote! { #arg_name: &[u8] }
    });
    let var_declarations = arg_types.iter().enumerate().map(|(i, &arg_type)| {
        let arg_name = Ident::new(&format!("arg{}", i), Span::call_site());
        let var_name = Ident::new(&format!("num{}", i), Span::call_site());
        quote! {
            let #var_name = <#arg_type>::from_wasm_input(#arg_name)?;
        }
    });
    let closure_args = (0..n_args).map(|i| {
        let var_name = Ident::new(&format!("num{}", i), Span::call_site());
        quote! { #var_name }
    });

    let expanded = quote! {
        #[wasm_func]
        fn #func_name(#(#arg_declarations),*) -> Result<Vec<u8>, anyhow::Error> {
            #(#var_declarations)*
            let result = (#closure)(#(#closure_args),*);
            Ok(result.into_wasm_output())
        }
    };

    TokenStream::from(expanded)
}

fn get_arg_types(closure: &syn::ExprClosure) -> Vec<&syn::Type> {
    let mut arg_types = Vec::new();

    for (_, input) in closure.inputs.iter().enumerate() {
        match input {
            Pat::Type(pat_type) => {
                arg_types.push(pat_type.ty.as_ref());
            }
            // Pat::Ident(pat_ident) if pat_ident.ty.is_none() => {
            //     bail!("Missing type annotation for closure parameter at index {}", index);
            // }
            // _ => {
            //     bail!("Missing type annotation for closure parameter at index {}", index);
            // }
            _ => {}
        }
    }

    arg_types
}

struct DefineFuncInput {
    func_name: Ident,
    closure: syn::ExprClosure,
}

impl syn::parse::Parse for DefineFuncInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let func_name: Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let closure: syn::ExprClosure = input.parse()?;

        Ok(DefineFuncInput { func_name, closure })
    }
}

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Query)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let m = match input.data {
        syn::Data::Struct(s) => s,
        _ => unreachable!(),
    };
    let (impl_gen, type_gen, where_clause) = input.generics.split_for_impl();
    // todo: validate against remote struct.
    let fields = m.fields.iter()
        .map(|f| {
            let (name, ty) = (f.ident.as_ref().unwrap(), &f.ty);
            let name = format!("{}", name);
            quote! {
                query.push_str(&format!("{} {} ", #name, <#ty as prisma_client::Queryable>::query()));
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl #impl_gen prisma_client::Queryable for #name #type_gen #where_clause {
            fn query() -> String {
                let mut query = String::new();
                #(#fields)*
                format!("{{ {}}}", query)
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

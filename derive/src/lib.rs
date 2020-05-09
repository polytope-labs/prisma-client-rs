extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta, MetaList, NestedMeta, Lit, Attribute};

#[proc_macro_derive(Query, attributes(query))]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_gen, type_gen, where_clause) = input.generics.split_for_impl();

    let m = match input.data {
        syn::Data::Struct(s) => s,
        syn::Data::Enum(_) => {
            let expanded = quote! {
                impl #impl_gen prisma_client_rs::Queryable for #name #type_gen #where_clause {
                    fn query() -> String {
                        String::new()
                    }
                }
            };

            // Hand the output tokens back to the compiler
            return TokenStream::from(expanded)
        }
        _ => unreachable!(),
    };
    // todo: validate against remote struct.
    let fields = m.fields.iter()
        .map(|f| {
            let rename = get_rename(&f.attrs);
            let (name, ty) = (
                rename.or_else(|| Some(format!("{}", f.ident.as_ref().unwrap())))
                    .unwrap(),
                &f.ty
            );
            quote! {
                query.push_str(&format!("{} {} ", #name, <#ty as prisma_client_rs::Queryable>::query()));
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl #impl_gen prisma_client_rs::Queryable for #name #type_gen #where_clause {
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

fn get_rename(attrs: &Vec<Attribute>) -> Option<String> {
    attrs.iter()
        // this looks like a christmas tree
        .filter_map(|a| {
            if a.path.is_ident("query") {
                if let Meta::List(MetaList { nested, .. }) =  a.parse_meta().ok()? {
                    if let Some(NestedMeta::Meta(Meta::NameValue(name))) = nested.first() {
                       if name.path.is_ident("rename") {
                           if let Lit::Str(lstr) = &name.lit {
                               return Some(lstr.value())
                           }
                       }
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .pop()
}

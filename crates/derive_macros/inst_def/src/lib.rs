use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data::Struct, DeriveInput, GenericParam::Const, GenericParam::Type, Meta,
};

#[proc_macro_derive(InstDef, attributes(instdef))]
pub fn derive_air_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_inst_def(&input)
}

fn impl_inst_def(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    //handle genric structs
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    let fields = match &ast.data {
        Struct(data) => &data.fields,
        _ => panic!("AirFn can only be derived for structs"),
    };

    // Generate code to insert field names and values into the IndexMap
    let field_entries = fields
        .iter()
        .filter(|field| {
            // Skip the field if it has the `#[instdef(skip)]` attribute
            !field.attrs.iter().any(|attr| {
                attr.path().is_ident("instdef")
                    && if let Meta::List(list) = attr.meta.clone() {
                        list.tokens.to_string().contains("skip")
                    } else {
                        false
                    }
            })
        })
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let field_name = ident.to_string();

            quote! {
                let field_value = format!("{:?}", self.#ident);
                map.insert(#field_name.to_string(), field_value);
            }
        });

    // Handle generics
    let generics = ast.generics.params.iter().filter_map(|param| {
        if let Type(type_param) = param {
            let type_ident = &type_param.ident;
            Some(quote! {
                let mut type_name = std::any::type_name::<#type_ident>().to_string();
                type_name = type_name
                    .rfind("::")
                    .map(|i| type_name[i + 2..].to_string())
                    .unwrap_or(type_name);
                type_name = type_name.replace('>', "");
                map.insert(stringify!(#type_ident).to_string(), type_name);
            })
        } else if let Const(const_param) = param {
            let const_ident = &const_param.ident;
            Some(quote! {
                map.insert(stringify!(#const_ident).to_string(), format!("{:?}", #const_ident));
            })
        } else {
            None
        }
    });

    // Generate the final implementation for AirFn
    let expanded = quote! {
        impl #impl_generics InstDefTrait for #name #type_generics #where_clause {
            fn inst_def(&self) -> indexmap::IndexMap<String, String> {
                let mut map = indexmap::IndexMap::new();
                #(#generics)*
                #(#field_entries)*
                map
            }
        }
    };

    TokenStream::from(expanded)
}

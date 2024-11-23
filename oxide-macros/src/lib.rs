use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let columns_name = format_ident!("{}Columns", name);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_idents = fields.iter().map(|f| &f.ident);
    let field_types = fields.iter().map(|f| &f.ty);
    let field_names = field_idents.clone();

    let gen = quote! {
        #[derive(Debug, Clone)]
        pub struct #columns_name {
            #(
                pub #field_idents: Column<#name, #field_types>,
            )*
        }

        impl ModelColumns for #columns_name {
            type Model = #name;
        }

        impl Model<#columns_name> for #name {
            const TABLE: &'static str = stringify!(#name);

            fn columns() -> #columns_name {
                #columns_name {
                    #(
                        #field_names: Column::new(stringify!(#field_names)),
                    )*
                }
            }
        }

        impl #name {
            pub fn query() -> OxideQueryBuilder<Self, #columns_name> {
                OxideQueryBuilder::new()
            }
        }
    };

    gen.into()
}

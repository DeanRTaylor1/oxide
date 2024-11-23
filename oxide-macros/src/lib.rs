use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// A derive macro that generates ORM functionality for structs
/// This creates:
/// 1. A companion *Columns struct containing typed column definitions
/// 2. ModelColumns implementation for type safety
/// 3. Model implementation for table name and column access
/// 4. Query builder initialization method
#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree that we can analyze
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let table_name = format!("{}s", name.to_string().to_lowercase());

    // Create identifier for the companion *Columns struct
    // e.g., User -> UserColumns
    let columns_name = format_ident!("{}Columns", name);

    // Extract the fields from the struct, ensuring it's a struct with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Create iterators for field properties we'll need in the generated code
    let field_idents = fields.iter().map(|f| &f.ident); // Field names
    let field_types = fields.iter().map(|f| &f.ty); // Field types
    let field_names = field_idents.clone(); // Second iterator for names

    // Generate the companion types and implementations
    let gen = quote! {
        // Create the columns struct that holds metadata about each field
        #[derive(Debug, Clone)]
        pub struct #columns_name {
            #(
                // Each field becomes a Column<Model, Type>
                pub #field_idents: Column<#name, #field_types>,
            )*
        }

        // Implement ModelColumns trait to enable type-safe query building
        impl ModelColumns for #columns_name {
            type Model = #name;
        }

        // Implement Model trait to provide table name and column access
        impl Model<#columns_name> for #name {
            // Use the struct name as the database table name
            const TABLE: &'static str = stringify!(#table_name);

            // Create a columns instance with all field definitions
            fn columns() -> #columns_name {
                #columns_name {
                    #(
                        // Initialize each column with its name
                        #field_names: Column::new(stringify!(#field_names)),
                    )*
                }
            }
        }

        impl #name {
            pub fn table() -> &'static str {
                Self::TABLE
            }

            pub fn query() -> oxide_orm::OxideQueryBuilder<Self, #columns_name> {
                oxide_orm::OxideQueryBuilder::new()
            }

            pub fn insert() -> oxide_orm::OxideInsertQueryBuilder<Self, #columns_name> {
                oxide_orm::OxideInsertQueryBuilder::new()
            }

            pub fn update(id: i32) -> oxide_orm::OxideUpdateQueryBuilder<Self, #columns_name> {
                oxide_orm::OxideUpdateQueryBuilder::new(id)
            }

            pub fn get_field<T: Clone>(&self, field: &Option<T>) -> Option<T> {
                field.clone()
            }

            pub fn columns() -> #columns_name {
                <Self as oxide_orm::Model<#columns_name>>::columns()
            }
        }
    };

    // println!("Generated code: {}", gen.to_string());
    gen.into()
}

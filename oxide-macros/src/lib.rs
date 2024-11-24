use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemFn};

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree that we can analyze
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let table_name = format!("{}s", name.to_string().to_lowercase());

    // Create identifier for the companion *Columns struct
    let columns_name = format_ident!("{}Columns", name);
    let oxide_name = format_ident!("Oxide{}", name); // e.g., `OxideUser`

    // Extract the fields from the struct, ensuring it's a struct with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Collect field properties into vectors for reuse
    let field_idents: Vec<_> = fields.iter().map(|f| &f.ident).collect(); // Field names
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect(); // Field types

    // Generate the companion types and implementations
    let gen = quote! {
        #[derive(
            Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow
        )]
        pub struct #oxide_name {
            #(
                pub #field_idents: #field_types,
            )*
        }

        // Create the columns struct that holds metadata about each field
        #[derive(Debug, Clone)]
        pub struct #columns_name {
            #(
                // Each field becomes a Column<Model, Type>
                pub #field_idents: Column<#oxide_name, #field_types>,
            )*
        }

        // Implement ModelColumns trait to enable type-safe query building
        impl ModelColumns for #columns_name {
            type Model = #oxide_name;
        }

        // Implement Model trait to provide table name and column access
        impl Model<#columns_name> for #oxide_name {
            // Use the struct name as the database table name
            const TABLE: &'static str = stringify!(#table_name);

            // Create a columns instance with all field definitions
            fn columns() -> #columns_name {
                #columns_name {
                    #(
                        // Initialize each column with its name
                        #field_idents: Column::new(stringify!(#field_idents)),
                    )*
                }
            }
        }

        impl #oxide_name {
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

    gen.into()
}

/// Converts an async function into a compatible HTTP request handler for the Oxide framework.
///
/// # Usage
/// ```rust
/// #[handler]
/// async fn my_handler(ctx: &Context) -> OxideResponse {
///     // Your handler logic here
///     OxideResponse::new(ResponseType::SUCCESS, "Hello World!")
/// }
/// ```
///
/// # What it does
/// This macro takes your async function and:
/// 1. Keeps it as-is so you can still call it directly
/// 2. Creates a static handler function named `{your_function_name}_handler`
/// 3. Makes the handler compatible with Oxide's route registration system
///
/// # Requirements
/// Your function must:
/// - Be async
/// - Take a single parameter of type `&Context`
/// - Return an `OxideResponse`
///
/// # Example with Route Registration
/// ```rust
/// #[handler]
/// async fn get_user(ctx: &Context) -> OxideResponse {
///     // Handler logic
///     OxideResponse::new(ResponseType::SUCCESS, "User data")
/// }
///
/// // The macro creates a static `get_user_handler` that you can register
/// app.route("/user", Method::GET, get_user_handler);
/// ```
#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let handler_name = format!("{}_handler", fn_name);
    let handler_ident = syn::Ident::new(&handler_name, fn_name.span());
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;

    let output = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name(ctx: &Context) -> OxideResponse
            #fn_block

        #[allow(non_upper_case_globals)]
        pub static #handler_ident: fn(&Context) -> AsyncResponse<'_> = |ctx| Box::pin(#fn_name(ctx));
    };

    output.into()
}

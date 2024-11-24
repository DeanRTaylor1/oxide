use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemFn, ItemStruct};

/// Enhances a struct with ORM functionality and common derives for use with the Oxide framework.
///
/// # Usage
/// ```rust
/// #[model]
/// pub struct User {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub age: i32,
///     pub active: bool,
/// }
/// ```
///
/// # What it does
/// The `#[model]` macro modifies your struct to:
/// 1. Automatically derive commonly needed traits:
///    - `Debug`, `Clone`, `serde::Serialize`, `serde::Deserialize`, `sqlx::FromRow`
/// 2. Generate a companion `Columns` struct (e.g., `UserColumns`) for type-safe query building.
/// 3. Implement the `Model` trait, including:
///    - A `table()` method for accessing the table name (`users` for the example above).
///    - A `columns()` method for accessing field metadata.
/// 4. Add query-building methods for use with Oxide ORM:
///    - `query()`, `insert()`, `update(id)`, etc.
///
/// # Requirements
/// - The struct must have named fields.
/// - The struct's fields should map directly to database columns.
///
/// # Example
/// ```rust
/// #[model]
/// pub struct Product {
///     pub id: i32,
///     pub name: String,
///     pub price: f64,
///     pub in_stock: bool,
/// }
///
/// // Usage
/// fn main() {
///     let query = Product::query().filter(Product::columns().price.gt(10.0));
///     println!("Generated query: {:?}", query);
/// }
/// ```
///
/// # Notes
/// - The table name is automatically derived by pluralizing the struct name (e.g., `Product` -> `products`).
/// - Column metadata is accessible through the generated `Columns` struct (e.g., `Product::columns().name`).
/// - This macro eliminates the need to manually implement boilerplate for database operations.

#[proc_macro_attribute]
pub fn model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens as a struct definition
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident; // Struct name (e.g., `User`)
    let table_name = format!("{}s", name.to_string().to_lowercase());
    let columns_name = format_ident!("{}Columns", name);

    // Extract fields
    let fields = match input.fields {
        syn::Fields::Named(ref named) => &named.named,
        _ => panic!("Only named fields are supported"),
    };

    let field_idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Generate code to modify the struct definition and add implementations
    let output = quote! {
        #[derive(
            Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow
        )]
        #input

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
            const TABLE: &'static str = stringify!(#table_name);

            fn columns() -> #columns_name {
                #columns_name {
                    #(
                        #field_idents: Column::new(stringify!(#field_idents)),
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

    output.into()
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

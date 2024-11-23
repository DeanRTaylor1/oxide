pub use oxide_macros::Model;

mod database;
mod error;
mod query;
mod schema;
mod types;

pub use database::Database;
pub use error::Error;
pub use query::OxideQueryBuilder;
pub use schema::{Column, Model, ModelColumns};
pub use types::{SqlType, ToSql};

// Create a prelude for easy imports
pub mod prelude {
    pub use super::{Column, Model, ModelColumns, OxideQueryBuilder, SqlType, ToSql};
}

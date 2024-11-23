pub use oxide_macros::Model;

mod database;
mod error;
mod query;
mod schema;
mod types;

pub use database::{Database, Error};
pub use query::{OxideInsertQueryBuilder, OxideQueryBuilder};
pub use schema::{Column, Model, ModelColumns};
pub use types::{SqlType, ToSql};

// Create a prelude for easy imports
pub mod prelude {
    pub use super::{
        Column, Model, ModelColumns, OxideInsertQueryBuilder, OxideQueryBuilder, SqlType, ToSql,
    };
}

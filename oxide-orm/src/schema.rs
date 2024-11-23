use std::marker::PhantomData;

use sqlx::FromRow;

use crate::ToSql;

pub trait Table: Sized {
    const NAME: &'static str;
    type Data;
    fn create_sql() -> String;
    fn table_name() -> &'static str {
        Self::NAME
    }
}

pub trait ModelColumns: Sized {
    type Model: Model<Self>;
}

pub trait Model<C: ModelColumns>: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Sync {
    const TABLE: &'static str;
    fn columns() -> C;
}

#[derive(Debug, Clone)]
pub struct Column<M, T> {
    pub name: &'static str,
    _marker: PhantomData<(M, T)>,
}

impl<M, T> Column<M, T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

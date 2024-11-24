use oxide_core::Error;
use sqlx::postgres::PgQueryResult;
use sqlx::{postgres::PgRow, Error as SqlxError};
use sqlx::{FromRow, Pool, Postgres};

pub type PgPool = Pool<Postgres>;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, Error> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(Error::Database)?;
        Ok(Self { pool })
    }

    pub async fn query<T>(&self, query: String) -> Result<Vec<T>, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::Database)
    }

    pub async fn query_one<T>(&self, query: String) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(Error::Database)
    }

    pub async fn query_optional<T>(&self, query: String) -> Result<Option<T>, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&query)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Database)
    }

    pub async fn execute(&self, query: String) -> Result<PgQueryResult, Error> {
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(Error::Database)
    }
}

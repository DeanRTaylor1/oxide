use crate::error::Error;
use sqlx::{Pool, Postgres};

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

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}

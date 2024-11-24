mod pool;
mod transaction;

use async_trait::async_trait;
pub use pool::Pool;
pub use transaction::Transaction;

#[async_trait]
pub trait Connection: Send + Sync {
    async fn execute(&self, query: &str) -> Result<u64, Error>;
    async fn query<T: for<'a> From<Row>>(&self, query: &str) -> Result<Vec<T>, Error>;
    async fn transaction(&self) -> Result<Transaction<'_>, Error>;
}

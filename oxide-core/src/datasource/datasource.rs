use crate::Error;
use sqlx::postgres::PgQueryResult;
use sqlx::postgres::PgRow;
use sqlx::PgPool;
use sqlx::Transaction;
use sqlx::{FromRow, Postgres};
use tokio::time::{timeout, Duration};

/// A connection pool wrapper for PostgreSQL database operations.
///
/// # Features
/// - Connection pooling with timeout
/// - Query execution with type-safe results
/// - Transaction support
/// - Optional result handling
///
/// # Example
/// ```rust
/// use your_crate::{PgDatabase, User};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     // Initialize database connection
///     let db = PgDatabase::connect("postgres://user:pass@localhost/dbname").await?;
///
///     // Execute a query returning multiple rows
///     let users: Vec<User> = db.query("SELECT * FROM users".to_string()).await?;
///
///     // Execute a query returning one row
///     let user: User = db.query_one("SELECT * FROM users WHERE id = 1".to_string()).await?;
///
///     // Execute a query that might return no rows
///     if let Some(user) = db.query_optional("SELECT * FROM users WHERE email = 'test@example.com'".to_string()).await? {
///         println!("User found: {:?}", user);
///     }
///
///     // Execute a mutation query
///     db.execute("UPDATE users SET active = true WHERE id = 1".to_string()).await?;
///
///     // Use transactions
///     let mut tx = db.begin().await?;
///     sqlx::query("INSERT INTO users (name) VALUES ($1)")
///         .bind("Alice")
///         .execute(&mut *tx)
///         .await?;
///     tx.commit().await?;
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct PgDatabase {
    pool: PgPool,
}

impl PgDatabase {
    /// Creates a new database connection pool with a 5-second timeout.
    ///
    /// # Arguments
    /// * `database_url` - PostgreSQL connection string (e.g., "postgres://user:pass@localhost/dbname")
    ///
    /// # Returns
    /// * `Result<Self, Error>` - Connection pool or error if connection fails
    pub async fn connect(database_url: &str) -> Result<Self, Error> {
        let pool = timeout(Duration::from_secs(5), PgPool::connect(database_url))
            .await
            .map_err(|_| Error::Database(sqlx::Error::Configuration("Connection timeout".into())))?
            .map_err(Error::Database)?;
        Ok(Self { pool })
    }

    /// Executes a query returning multiple rows.
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize rows into. Must implement `FromRow`.
    ///
    /// # Arguments
    /// * `query` - SQL query string
    ///
    /// # Returns
    /// * `Result<Vec<T>, Error>` - Vector of deserialized rows or error
    pub async fn query<T>(&self, query: String) -> Result<Vec<T>, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::Database)
    }

    /// Executes a query expecting exactly one row.
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize the row into. Must implement `FromRow`.
    ///
    /// # Arguments
    /// * `query` - SQL query string
    ///
    /// # Returns
    /// * `Result<T, Error>` - Deserialized row or error if no/multiple rows found
    pub async fn query_one<T>(&self, query: String) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(Error::Database)
    }

    /// Executes a query returning zero or one row.
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize the row into. Must implement `FromRow`.
    ///
    /// # Arguments
    /// * `query` - SQL query string
    ///
    /// # Returns
    /// * `Result<Option<T>, Error>` - Optional deserialized row or error
    pub async fn query_optional<T>(&self, query: String) -> Result<Option<T>, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&query)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Database)
    }

    /// Executes a query that doesn't return rows (INSERT, UPDATE, DELETE).
    ///
    /// # Arguments
    /// * `query` - SQL query string
    ///
    /// # Returns
    /// * `Result<PgQueryResult, Error>` - Query result containing affected rows or error
    pub async fn execute(&self, query: String) -> Result<PgQueryResult, Error> {
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(Error::Database)
    }

    /// Begins a new database transaction.
    ///
    /// # Returns
    /// * `Result<Transaction<'_, Postgres>, Error>` - Transaction handle or error
    ///
    /// # Note
    /// Remember to either commit or rollback the transaction when done by calling tx.commit().await or tx.rollback().await
    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>, Error> {
        self.pool.begin().await.map_err(Error::Database)
    }
}

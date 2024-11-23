use core::fmt;

#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    Configuration(String),
    Value(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Database(e) => write!(f, "Database error: {}", e),
            Error::Configuration(e) => write!(f, "Configuration error: {}", e),
            Error::Value(e) => write!(f, "Value error: {}", e),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::Database(error)
    }
}

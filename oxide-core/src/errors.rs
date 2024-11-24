use sqlx::Error as SqlxError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    // HTTP related errors
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    InternalServer(String),

    // Database errors
    Database(SqlxError),

    // Validation errors
    Validation(String),

    // Configuration errors
    Config(String),

    // Serialization/Deserialization errors
    Serialization(String),
    Deserialization(String),

    // IO errors
    Io(std::io::Error),

    // Custom error for specific use cases
    Custom(String),
}

impl Error {
    pub fn status_code(&self) -> u16 {
        match self {
            Error::BadRequest(_) => 400,
            Error::Unauthorized(_) => 401,
            Error::Forbidden(_) => 403,
            Error::NotFound(_) => 404,
            Error::InternalServer(_) => 500,
            Error::Database(_) => 500,
            Error::Validation(_) => 400,
            Error::Config(_) => 500,
            Error::Serialization(_) => 500,
            Error::Deserialization(_) => 400,
            Error::Io(_) => 500,
            Error::Custom(_) => 500,
        }
    }

    pub fn error_type(&self) -> &str {
        match self {
            Error::BadRequest(_) => "BAD_REQUEST",
            Error::Unauthorized(_) => "UNAUTHORIZED",
            Error::Forbidden(_) => "FORBIDDEN",
            Error::NotFound(_) => "NOT_FOUND",
            Error::InternalServer(_) => "INTERNAL_SERVER_ERROR",
            Error::Database(_) => "DATABASE_ERROR",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::Config(_) => "CONFIG_ERROR",
            Error::Serialization(_) => "SERIALIZATION_ERROR",
            Error::Deserialization(_) => "DESERIALIZATION_ERROR",
            Error::Io(_) => "IO_ERROR",
            Error::Custom(_) => "CUSTOM_ERROR",
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            Error::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Error::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Error::NotFound(msg) => write!(f, "Not Found: {}", msg),
            Error::InternalServer(msg) => write!(f, "Internal Server Error: {}", msg),
            Error::Database(e) => write!(f, "Database Error: {}", e),
            Error::Validation(msg) => write!(f, "Validation Error: {}", msg),
            Error::Config(msg) => write!(f, "Configuration Error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization Error: {}", msg),
            Error::Deserialization(msg) => write!(f, "Deserialization Error: {}", msg),
            Error::Io(e) => write!(f, "IO Error: {}", e),
            Error::Custom(msg) => write!(f, "Custom Error: {}", msg),
        }
    }
}

// Implement From traits for common error types
impl From<SqlxError> for Error {
    fn from(err: SqlxError) -> Self {
        Error::Database(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        if err.is_syntax() || err.is_data() {
            Error::Deserialization(err.to_string())
        } else {
            Error::Serialization(err.to_string())
        }
    }
}

// Helper trait for Result with our Error type
pub type OxideResult<T> = std::result::Result<T, Error>;

// Extension trait for easy error conversion in handlers
pub trait IntoResponse {
    fn into_response(self) -> Vec<u8>;
}

impl IntoResponse for Error {
    fn into_response(self) -> Vec<u8> {
        let error_response = serde_json::json!({
            "error": {
                "type": self.error_type(),
                "message": self.to_string(),
                "status": self.status_code()
            }
        });

        serde_json::to_vec(&error_response).unwrap_or_else(|_| {
            serde_json::to_vec(&serde_json::json!({
                "error": {
                    "type": "INTERNAL_SERVER_ERROR",
                    "message": "Failed to serialize error response",
                    "status": 500
                }
            }))
            .unwrap_or_default()
        })
    }
}

pub mod config;
pub mod connection;
pub mod errors;
pub mod http;
pub mod logger;
pub mod server;
pub mod macros {
    pub use oxide_macros::handler;
}

pub use config::Config;
pub use connection::Connection;
pub use errors::Error;
pub use http::{HttpHandler, HttpMethod, RequestResponse};
pub use logger::Logger;
pub use server::Server;

pub mod prelude {
    pub use crate::errors::Error;
    pub use crate::http::{BufferBuilder, HttpHandler, HttpMethod, OxideResponse};
    pub use crate::macros::handler;
    pub use crate::Config;
    pub use crate::Logger;
    pub use crate::Server;
}

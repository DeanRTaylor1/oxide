use crate::logger::{LogLevel, Logger};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_request_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_request_size: 1024 * 1024,
        }
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_request_size: Option<usize>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn max_request_size(mut self, size: usize) -> Self {
        self.max_request_size = Some(size);
        self
    }

    pub fn build(self) -> Config {
        let default = Config::default();
        Config {
            host: self.host.unwrap_or(default.host),
            port: self.port.unwrap_or(default.port),
            max_request_size: self.max_request_size.unwrap_or(default.max_request_size),
        }
    }
}

impl From<ConfigBuilder> for Config {
    fn from(builder: ConfigBuilder) -> Self {
        builder.build()
    }
}

impl Config {
    pub fn from_env() -> Self {
        let validator = EnvValidator::new(Logger::new());
        Self {
            host: validator.get_var("HOST", "a string (e.g., '127.0.0.1')"),
            port: validator.get_var_parse("PORT", "a number between 0-65535"),
            max_request_size: validator.get_var_parse(
                "MAX_REQUEST_SIZE",
                "a number in bytes (e.g., 1048576 for 1MB)",
            ),
        }
    }
}

pub struct EnvValidator {
    logger: Logger,
}

impl EnvValidator {
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    pub fn error(&self, key: &str, type_info: &str) -> ! {
        self.logger
            .log(LogLevel::Warning, &format!("Set {} as {}", key, type_info));
        self.logger
            .panic(&format!("Missing environment variable: {}", key));
    }

    pub fn get_var(&self, key: &str, type_info: &str) -> String {
        env::var(key).unwrap_or_else(|_| self.error(key, type_info))
    }

    pub fn get_var_parse<T: std::str::FromStr>(&self, key: &str, type_info: &str) -> T {
        self.get_var(key, type_info)
            .parse()
            .unwrap_or_else(|_| self.error(key, type_info))
    }
}

use crate::Logger;
use flate2::{write::GzEncoder, Compression};
use std::io::Write;

#[derive(Default)]
pub struct ResponseBuilder {
    status_line: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    status: u16,
}

impl ResponseBuilder {
    // Status code constants
    pub const OK: (u16, &'static str) = (200, "OK");
    pub const CREATED: (u16, &'static str) = (201, "Created");
    pub const UPDATED: (u16, &'static str) = (200, "Success");
    pub const NO_CONTENT: (u16, &'static str) = (204, "No Content");
    pub const DELETED: (u16, &'static str) = (200, "Success");
    pub const NOT_FOUND: (u16, &'static str) = (404, "Not Found");
    pub const BAD_REQUEST: (u16, &'static str) = (400, "Bad Request");
    pub const INTERNAL_SERVER_ERROR: (u16, &'static str) = (500, "Internal Server Error");

    // Content type constants
    pub const PLAIN: &'static str = "text/plain";
    pub const HTML: &'static str = "text/html";
    pub const JSON: &'static str = "application/json";

    const COMPRESSIBLE_TYPES: [&'static str; 4] =
        ["text/plain", "text/html", "text/css", "application/json"];
    const MIN_COMPRESS_SIZE: usize = 1400;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: (u16, &str)) -> Self {
        self.status_line = format!("HTTP/1.1 {} {}", status.0, status.1);
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn content_type(self, content_type: &str) -> Self {
        self.header("Content-Type", content_type)
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        let length = self.body.len().to_string();
        self.header("Content-Length", &length)
    }

    pub fn text(self, body: impl AsRef<str>) -> Self {
        self.content_type(Self::PLAIN)
            .body(body.as_ref().as_bytes().to_vec())
    }

    pub fn json(self, body: impl AsRef<str>) -> Self {
        self.content_type(Self::JSON)
            .body(body.as_ref().as_bytes().to_vec())
    }

    pub fn html(self, body: impl AsRef<str>) -> Self {
        self.content_type(Self::HTML)
            .body(body.as_ref().as_bytes().to_vec())
    }

    fn get_accepted_encoding(&self) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == "Accept-Encoding")
            .map(|(_, v)| v.as_str())
    }

    fn should_compress(&self) -> bool {
        self.headers
            .iter()
            .any(|(k, v)| k == "Content-Type" && Self::COMPRESSIBLE_TYPES.contains(&v.as_str()))
            && self.body.len() > Self::MIN_COMPRESS_SIZE
            && self
                .get_accepted_encoding()
                .map_or(false, |enc| enc.to_lowercase().contains("gzip"))
    }

    fn compress_body(&mut self) -> bool {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        match encoder.write_all(&self.body).and_then(|_| encoder.finish()) {
            Ok(compressed) if compressed.len() < self.body.len() => {
                self.body = compressed;
                self.headers
                    .push(("Content-Encoding".to_string(), "gzip".to_string()));
                self.headers.retain(|(k, _)| k != "Content-Length");
                self.headers
                    .push(("Content-Length".to_string(), self.body.len().to_string()));
                true
            }
            _ => false,
        }
    }

    pub fn build(mut self) -> Vec<u8> {
        if self.should_compress() {
            self.compress_body();
        }

        let mut response = Vec::new();

        // Add status line
        response.extend_from_slice(self.status_line.as_bytes());
        response.extend_from_slice(b"\r\n");

        // Add headers
        for (key, value) in self.headers {
            response.extend_from_slice(key.as_bytes());
            response.extend_from_slice(b": ");
            response.extend_from_slice(value.as_bytes());
            response.extend_from_slice(b"\r\n");
        }

        // Add blank line and body
        response.extend_from_slice(b"\r\n");
        response.extend_from_slice(&self.body);

        response
    }

    pub fn ok_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::OK).text(message).build()
    }

    pub fn created_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::CREATED).text(message).build()
    }

    pub fn not_found_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::NOT_FOUND).text(message).build()
    }

    pub fn bad_request_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::BAD_REQUEST).text(message).build()
    }

    pub fn server_error_response(message: impl AsRef<str>) -> Vec<u8> {
        Self::new()
            .status(Self::INTERNAL_SERVER_ERROR)
            .text(message)
            .build()
    }

    // JSON variants
    pub fn ok_json(json: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::OK).json(json).build()
    }

    pub fn not_found_json(json: impl AsRef<str>) -> Vec<u8> {
        Self::new().status(Self::NOT_FOUND).json(json).build()
    }

    // Default responses with standard messages
    pub fn default_not_found() -> Vec<u8> {
        Self::not_found_response("Resource not found")
    }

    pub fn default_bad_request() -> Vec<u8> {
        Self::bad_request_response("Bad request")
    }

    pub fn default_server_error() -> Vec<u8> {
        Self::server_error_response("Internal server error")
    }
}

// Helper methods for common responses
impl ResponseBuilder {
    pub fn ok() -> Self {
        Self::new().status(Self::OK)
    }

    pub fn deleted() -> Self {
        Self::new().status(Self::DELETED)
    }

    pub fn created() -> Self {
        Self::new().status(Self::CREATED)
    }

    pub fn not_found() -> Self {
        Self::new().status(Self::NOT_FOUND)
    }

    pub fn bad_request() -> Self {
        Self::new().status(Self::BAD_REQUEST)
    }

    pub fn server_error() -> Self {
        Self::new().status(Self::INTERNAL_SERVER_ERROR)
    }
}

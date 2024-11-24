use crate::http::{HttpHandler, HttpMethod, RequestResponse};
use crate::logger::{LogLevel, Logger};

use bytes::BytesMut;
use std::io;
use std::str::FromStr;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Protocol {
    Http1,
    WebSocket,
    Http2,
    Unknown,
}

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,

    buffer: BytesMut,

    logger: Logger,

    http_handler: Arc<HttpHandler>,
}

impl Connection {
    pub fn new(stream: TcpStream, http_handler: Arc<HttpHandler>) -> Result<Self, io::Error> {
        let stream = BufWriter::new(stream);
        let buffer = BytesMut::with_capacity(1024 * 1024);
        let logger = Logger::new();

        Ok(Self {
            stream,
            buffer,
            logger,
            http_handler,
        })
    }

    pub async fn process(mut self) -> io::Result<()> {
        if 0 == self.stream.read_buf(&mut self.buffer).await? {
            self.logger.log(LogLevel::Application, "Connection closed");
            return Ok(());
        }

        let first_bytes = self.peek(7);

        match self.detect_protocol(first_bytes) {
            Protocol::Http1 => {
                self.handle_http().await?;
            }
            Protocol::Unknown => self.logger.log(
                LogLevel::Application,
                format!(
                    "Unknown protocol: '{}'",
                    String::from_utf8_lossy(first_bytes)
                )
                .as_str(),
            ),
            _ => self.logger.log(
                LogLevel::Application,
                format!("Unsupported protocol: {:?}", first_bytes).as_str(),
            ),
        }
        Ok(())
    }

    pub async fn handle_http(&mut self) -> io::Result<()> {
        let start_time = std::time::Instant::now();

        let ip = self.stream.get_ref().peer_addr()?.to_string();

        let request_line = std::str::from_utf8(&self.buffer)
            .ok()
            .and_then(|s| s.lines().next())
            .unwrap_or("");

        let mut parts = request_line.split_whitespace();
        let method = parts
            .next()
            .map(|s| HttpMethod::from_str(s).unwrap_or(HttpMethod::Unknown))
            .unwrap_or(HttpMethod::Unknown);

        let path = parts.next().unwrap_or("/").to_string();

        let response = self.http_handler.handle(&self.buffer).await;
        let duration = start_time.elapsed();

        Logger::log_http(&RequestResponse {
            method,
            path,
            ip,
            status: response.status,
            duration,
        });

        self.stream.write_all(&response.buffer).await?;
        self.stream.flush().await
    }

    fn peek(&self, n: usize) -> &[u8] {
        &self.buffer[..std::cmp::min(n, self.buffer.len())]
    }

    fn detect_protocol(&self, bytes: &[u8]) -> Protocol {
        if bytes.len() < 4 {
            return Protocol::Unknown;
        }

        if let Some(space_pos) = bytes.iter().position(|&b| b == b' ') {
            let method = &bytes[..space_pos];
            match method {
                b"GET" | b"POST" | b"PUT" | b"HEAD" | b"DELETE" | b"PATCH" => Protocol::Http1,
                b"PRI" => Protocol::Http2,
                _ => Protocol::Unknown,
            }
        } else {
            Protocol::Unknown
        }
    }
}

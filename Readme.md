# Rust HTTP Framework

A low-level HTTP framework focused on learning and understanding web server implementations. The goal is to build a feature-rich framework from the ground up, implementing as much as possible from scratch while using established libraries only for areas where reimplementation would be impractical.

## Core Features

### HTTP Protocol Support

- [x] HTTP/1.1 basic implementation
- [ ] HTTP/2 support
  - [ ] Frame encoding/decoding
  - [ ] Stream multiplexing
  - [ ] Flow control
  - [ ] Server push
  - [ ] Priority handling
  - [ ] Settings negotiation
- [ ] WebSocket support
  - [ ] Upgrade handling
  - [ ] Frame parsing
  - [ ] Message handling
  - [ ] Connection lifecycle management
  - [ ] Ping/Pong handling

### Routing & Request Handling

- [x] Basic routing
- [x] Method-based handlers (GET, POST, etc.)
- [x] Path parameters
- [x] Query string parsing
- [x] Route groups/namespacing
- [x] Middleware support
- [ ] Static file serving
- [ ] Request body parsing
  - [x] JSON
  - [ ] Form data
  - [ ] Multipart
  - [ ] Stream handling for large uploads

### TLS & Security

- [ ] TLS support
  - [ ] Certificate management
  - [ ] Let's Encrypt integration
  - [ ] ALPN for HTTP/2
  - [ ] SNI support
- [ ] Security headers
- [ ] CORS support
- [ ] Rate limiting
- [ ] Request validation

### Domain Management

- [ ] Automatic IP detection
- [ ] DNS record management
  - [ ] Multiple provider support (Cloudflare, etc.)
  - [ ] Automatic updates
  - [ ] Health checking
- [ ] Domain verification

### Configuration & Environment

- [x] Environment-based configuration
- [ ] Config file support
- [ ] Secret management
- [ ] Multiple environment support (dev, prod, etc.)

### Logging & Monitoring

- [x] Request logging
- [x] Color-coded console output
- [ ] Structured logging
- [ ] Log rotation
- [ ] Metrics collection
  - [ ] Request duration
  - [ ] Status code distribution
  - [ ] Error rates
- [ ] Health check endpoints

### Developer Experience

- [ ] Hot reload for development
- [ ] CLI tools
  - [ ] Project scaffolding
  - [ ] Route generation
  - [ ] Configuration management
- [ ] Detailed error pages in development
- [ ] API documentation generation
- [ ] Test utilities

### Performance

- [ ] Connection pooling
- [ ] Request pipelining
- [ ] Caching support
  - [ ] In-memory cache
  - [ ] External cache support (Redis, etc.)
- [ ] Compression (gzip, brotli)
- [ ] Load balancing

## Design Principles

1. **Learn by Implementation**: Implement as much as possible from scratch to understand the underlying concepts.

2. **Minimal Dependencies**: Only use external crates when:

   - Implementation would require months of work (e.g., HTTP/2)
   - Security is critical (e.g., TLS, encryption)
   - Standard formats are involved (e.g., JSON parsing)

3. **Production Ready**: While learning focused, the framework should be reliable and secure enough for production use.

4. **Clear Abstractions**: Each component should have clear boundaries and responsibilities.

5. **Flexible Architecture**: Allow users to opt-in to higher-level abstractions while maintaining access to low-level controls.

## Usage Example

```rust
use rust_http_framework::{Server, Config, HttpHandler};

#[tokio::main]
async fn main() {
    let mut http = HttpHandler::new();

    // Basic routing
    http.get("/", |_req| {
        ResponseBuilder::ok()
            .text("Hello World!")
            .build()
    });

    // JSON handling
    http.post("/api/data", |req| {
        if let Some(data) = req.json_body::<MyData>() {
            // Handle data
        }
    });

    let server = Server::new(Config::default());
    server.run().await;
}
```

## Current Status

This is an actively developed project focusing on educational purposes while building towards production readiness. Current focus areas:

1. HTTP/2 implementation
2. WebSocket support
3. TLS integration
4. Domain management

## Contributing

Contributions are welcome! Please read our contributing guidelines and code of conduct before submitting pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

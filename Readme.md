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

# Oxide Core

## Server Setup

First, initialize the server with a database connection:

```rust
use oxide_core::{Config, Server, PgDatabase, OxideResponse, OxideRes};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize database connection
    let db = PgDatabase::connect("postgres://oxide:oxide123@localhost:5432/oxide")
        .await
        .expect("Failed to connect to database");

    // Set up server with database
    let mut server = Server::new(Config::default());
    server.static_file("/", "index.html");
    server.with_datasource(db);

    // Register routes
    server.router
        .get("/users", list_users_handler)
        .get("/users/:id", get_user_handler)
        .post("/users", create_user_handler);

    server.run().await
}
```

## Model Definition

Define your database models:

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub active: bool,
}
```

## Handler Implementation

Implement handlers that use the database through context:

```rust
use oxide_core::{Context, OxideResponse, OxideRes};

#[handler]
async fn get_user(ctx: &Context) -> OxideResponse {
    let db = match ctx.db() {
        Some(db) => db,
        None => return OxideResponse::text(OxideRes::ServerError, "No database connection"),
    };

    let user_id = match ctx.param("id").and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => return OxideResponse::text(OxideRes::BadRequest, "Invalid ID"),
    };

    let query = format!("SELECT * FROM users WHERE id = {}", user_id);
    match db.query_one::<User>(query).await {
        Ok(user) => OxideResponse::json(OxideRes::Success, user),
        Err(_) => OxideResponse::text(OxideRes::NotFound, "User not found"),
    }
}

#[handler]
async fn list_users(ctx: &Context) -> OxideResponse {
    let db = match ctx.db() {
        Some(db) => db,
        None => return OxideResponse::text(OxideRes::ServerError, "No database connection"),
    };

    match db.query::<User>("SELECT * FROM users".to_string()).await {
        Ok(users) => OxideResponse::json(OxideRes::Success, users),
        Err(e) => OxideResponse::text(OxideRes::ServerError, e.to_string()),
    }
}

#[handler]
async fn create_user(ctx: &Context) -> OxideResponse {
    let db = match ctx.db() {
        Some(db) => db,
        None => return OxideResponse::text(OxideRes::ServerError, "No database connection"),
    };

    let user: User = match ctx.request.json_body() {
        Some(user) => user,
        None => return OxideResponse::text(OxideRes::BadRequest, "Invalid user data"),
    };

    let query = format!(
        "INSERT INTO users (email, name, active) VALUES ('{}', '{}', {}) RETURNING *",
        user.email, user.name, user.active
    );

    match db.query_one::<User>(query).await {
        Ok(created_user) => OxideResponse::json(OxideRes::Created, created_user),
        Err(e) => OxideResponse::text(OxideRes::ServerError, e.to_string()),
    }
}

#[handler]
async fn update_user(ctx: &Context) -> OxideResponse {
    let db = match ctx.db() {
        Some(db) => db,
        None => return OxideResponse::text(OxideRes::ServerError, "No database connection"),
    };

    let tx = match db.begin().await {
        Ok(tx) => tx,
        Err(e) => return OxideResponse::text(OxideRes::ServerError, e.to_string()),
    };

    // Example of transaction usage
    // ... perform multiple operations ...
    tx.commit().await.map_err(|e| /* handle error */)?;

    OxideResponse::text(OxideRes::Success, "User updated")
}
```

## Database Schema

Ensure your database has the required schema:

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true
);
```

## Best Practices

1. **Context Usage**:

   - Always check for database availability using `ctx.db()`
   - Handle the Option return appropriately
   - Use early returns for error cases

2. **Error Handling**:

   - Return appropriate OxideRes types for different scenarios
   - Provide meaningful error messages
   - Use proper HTTP status codes via OxideRes enum

3. **Transactions**:

   - Use transactions for multi-step operations
   - Properly handle commit/rollback
   - Implement proper error propagation

4. **Response Types**:
   - Use `OxideResponse::json` for successful data responses
   - Use `OxideResponse::text` for error messages
   - Match response types to your API design

The `PgDatabase` can be used in two ways:

## 1. Standalone Usage

As shown above, you can use it directly with raw SQL queries and structs implementing `FromRow`:

```rust
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[handler]
async fn get_user(ctx: &Context) -> OxideResponse {
    let db = match ctx.db() {
        Some(db) => db,
        None => return OxideResponse::text(OxideRes::ServerError, "No database connection"),
    };

    let query = "SELECT * FROM users WHERE id = 1".to_string();
    match db.query_one::<User>(query).await {
        Ok(user) => OxideResponse::json(OxideRes::Success, user),
        Err(e) => OxideResponse::text(OxideRes::ServerError, e.to_string()),
    }
}
```

## 2. With oxide_orm Integration

For a more ergonomic experience with type-safe queries, you can use the `oxide_orm` package:

```rust
use oxide_orm::model;

#[model]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[handler]
async fn get_user(ctx: &Context) -> OxideResponse {
    let db = match ctx.db() {
        Some(db) => db,
        None => return OxideResponse::text(OxideRes::ServerError, "No database connection"),
    };

    // Type-safe query building
    match User::query()
        .and_where(User::columns().id, 1)
        .fetch_one(db)
        .await
    {
        Ok(user) => OxideResponse::json(OxideRes::Success, user),
        Err(e) => OxideResponse::text(OxideRes::ServerError, e.to_string()),
    }
}
```

See the `oxide_orm` documentation for more details on the ORM features and type-safe query building.

use oxide_core::{
    http::{Context, MiddlewareResult},
    logger::LogLevel,
    prelude::*,
};
use oxide_orm::prelude::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    example();
    let mut server = Server::new(Config::default());
    server.static_file("/", "index.html");
    routes(&mut server);
    register_middleware(&mut server);
    server.run().await
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonData {
    message: String,
}

fn root_handler(_ctx: &Context) -> Vec<u8> {
    ResponseBuilder::ok_response("Hello from Dean's server!")
}

fn user_handler(ctx: &Context) -> Vec<u8> {
    let user_id = ctx.param("id").unwrap_or("0");
    Logger::new().log(
        oxide_core::logger::LogLevel::Debug,
        &format!("User ID: {}", user_id),
    );
    ResponseBuilder::ok().text(format!("{}", user_id)).build()
}

fn cookies_handler(ctx: &Context) -> Vec<u8> {
    let cookies = ctx.request.cookies();
    match serde_json::to_string(&cookies) {
        Ok(json) => ResponseBuilder::ok().json(json).build(),
        Err(_) => ResponseBuilder::server_error()
            .text("Failed to serialize cookies")
            .build(),
    }
}

fn post_handler(ctx: &Context) -> Vec<u8> {
    match ctx.request.json_body::<JsonData>() {
        Some(body) => {
            println!("JSON body: {}", body.message);
            ResponseBuilder::created_response("Hello from Dean's server!")
        }
        None => ResponseBuilder::bad_request().text("Bad Request").build(),
    }
}

fn put_handler(ctx: &Context) -> Vec<u8> {
    let id = ctx.param("id").unwrap_or("0");
    ResponseBuilder::created()
        .text(format!("Updated data for ID: {}", id))
        .build()
}

fn delete_handler(ctx: &Context) -> Vec<u8> {
    let id = ctx.param("id").unwrap_or("0");
    ResponseBuilder::deleted()
        .text(format!("Deleted data for ID: {}", id))
        .build()
}

fn routes(server: &mut Server) {
    let mut api = server.router.group("/api");

    let mut data = api.group("/data");

    data.put("/:id", put_handler).delete("/:id", delete_handler);

    let mut user_group = api.group("/user");

    user_group
        .get("/:id", user_handler)
        .post("/", post_handler)
        .delete("/:id", delete_handler);

    server
        .router
        .get("/api", root_handler)
        .get("/user/:id", user_handler)
        .get("/cookies", cookies_handler)
        .post("/api", post_handler)
        .add_group(data)
        .add_group(user_group);
}

fn global_middleware(ctx: Context) -> MiddlewareResult {
    let logger = Logger::new();
    logger.log(
        oxide_core::logger::LogLevel::Info,
        "Global Middleware executed",
    );
    Ok(ctx)
}

fn specific_middleware(ctx: Context) -> MiddlewareResult {
    let logger = Logger::new();
    logger.log(
        LogLevel::Info,
        format!(
            "Specific Middleware executed in route: {}",
            ctx.request.path
        )
        .as_str(),
    );
    Ok(ctx)
}

fn register_middleware(server: &mut Server) {
    server.middleware.add_global(global_middleware);
    server
        .middleware
        .for_route("/api/data/*", specific_middleware);
}

#[derive(sqlx::FromRow, Model, Debug, Clone)]
pub struct User {
    id: i32,
    name: String,
    email: String,
    age: i32,
    active: bool,
}

fn example() {
    // Complex query example
    let query = User::query()
        .select(User::columns().name)
        .select(User::columns().email)
        .where_eq(User::columns().active, true)
        .build();

    println!("Query: {}", query);
    // SELECT name, email FROM users
    // WHERE active = TRUE
    // AND age > 18
    // AND email LIKE '%@example.com'
    // AND id IN (1, 2, 3)

    // Query with OR condition
    let query = User::query()
        .where_eq(User::columns().email, "test@example.com".to_string())
        .where_eq(User::columns().email, "other@example.com".to_string())
        .build();

    println!("Query: {}", query);
    // SELECT * FROM users
    // WHERE (email = 'test@example.com' OR email = 'other@example.com')
}

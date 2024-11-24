use std::os::macos::raw::stat;

use oxide_core::{
    http::{AsyncResponse, BufferBuilder, Context, MiddlewareResult, OxideResponse},
    logger::LogLevel,
    prelude::*,
};
use oxide_orm::{prelude::*, Database};

#[derive(Debug, serde::Deserialize)]
pub struct JsonData {
    message: String,
}

#[derive(sqlx::FromRow, Model, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub active: bool,
}

// All handlers using the new macro
#[handler]
async fn get_user(ctx: &Context) -> OxideResponse {
    let user_id = ctx.param("id").unwrap_or("0");
    let db_conn = match Database::connect("postgres://oxide:oxide123@localhost:5432/oxide").await {
        Ok(db) => db,
        Err(e) => {
            let (status, _) = BufferBuilder::INTERNAL_SERVER_ERROR;
            return OxideResponse::new(
                BufferBuilder::server_error().text(e.to_string()).build(),
                status,
            );
        }
    };

    let user_id = match user_id.parse::<i32>().ok() {
        Some(id) => id,
        None => {
            let (status, _) = BufferBuilder::BAD_REQUEST;
            return OxideResponse::new(
                BufferBuilder::bad_request().text("Invalid ID").build(),
                status,
            );
        }
    };

    match User::query()
        .and_where(User::columns().id, user_id)
        .fetch_one::<User>(&db_conn)
        .await
    {
        Ok(user) => {
            let (status, _) = BufferBuilder::OK;
            return OxideResponse::new(
                BufferBuilder::ok()
                    .json(serde_json::to_string(&user).unwrap())
                    .build(),
                status,
            );
        }
        Err(e) => {
            let (status, _) = BufferBuilder::INTERNAL_SERVER_ERROR;
            return OxideResponse::new(
                BufferBuilder::server_error().text(e.to_string()).build(),
                status,
            );
        }
    }
}

#[handler]
async fn root(_ctx: &Context) -> OxideResponse {
    let (status, _) = BufferBuilder::OK;
    return OxideResponse::new(
        BufferBuilder::ok()
            .text("Hello from Dean's server!")
            .build(),
        status,
    );
}

#[handler]
async fn user(ctx: &Context) -> OxideResponse {
    let db = Database::connect("postgres://oxide:oxide123@localhost:5432/oxide")
        .await
        .unwrap();

    let user_id = match ctx.param("id").and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            let (status, _) = BufferBuilder::BAD_REQUEST;
            return OxideResponse::new(
                BufferBuilder::bad_request().text("Invalid ID").build(),
                status,
            );
        }
    };

    let user: Option<User> = match User::query()
        .and_where(User::columns().id, user_id)
        .fetch_optional(&db)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            let (status, _) = BufferBuilder::INTERNAL_SERVER_ERROR;
            return OxideResponse::new(
                BufferBuilder::server_error().text(format!("{}", e)).build(),
                status,
            );
        }
    };

    return match user {
        Some(user) => {
            let (status, _) = BufferBuilder::OK;
            OxideResponse::new(
                BufferBuilder::ok()
                    .json(&serde_json::to_string(&user).unwrap())
                    .build(),
                status,
            )
        }
        None => {
            let (status, _) = BufferBuilder::NOT_FOUND;
            OxideResponse::new(
                BufferBuilder::not_found().text("User not found").build(),
                status,
            )
        }
        _ => {
            let (status, _) = BufferBuilder::INTERNAL_SERVER_ERROR;
            OxideResponse::new(
                BufferBuilder::server_error().text("Unknown error").build(),
                status,
            )
        }
    };
}

#[handler]
async fn cookies(ctx: &Context) -> OxideResponse {
    let cookies = ctx.request.cookies();
    match serde_json::to_string(&cookies) {
        Ok(json) => {
            let (status, _) = BufferBuilder::OK;
            return OxideResponse::new(BufferBuilder::ok().json(json).build(), status);
        }
        _ => {
            let (status, _) = BufferBuilder::INTERNAL_SERVER_ERROR;
            return OxideResponse::new(
                BufferBuilder::server_error()
                    .text("Failed to serialize cookies")
                    .build(),
                status,
            );
        }
    }
}

#[handler]
async fn post(ctx: &Context) -> OxideResponse {
    match ctx.request.json_body::<JsonData>() {
        Some(body) => {
            println!("JSON body: {}", body.message);
            let (status, _) = BufferBuilder::CREATED;
            return OxideResponse::new(
                BufferBuilder::created()
                    .text(format!("Created data for ID: {}", body.message))
                    .build(),
                status,
            );
        }
        None => {
            let (status, _) = BufferBuilder::BAD_REQUEST;
            return OxideResponse::new(
                BufferBuilder::bad_request().text("Invalid JSON").build(),
                status,
            );
        }
    }
}

#[handler]
async fn put(ctx: &Context) -> OxideResponse {
    let id = ctx.param("id").unwrap_or("0");
    let (status, _) = BufferBuilder::UPDATED;
    return OxideResponse::new(
        BufferBuilder::updated()
            .text(format!("Updated data for ID: {}", id))
            .build(),
        status,
    );
}

#[handler]
async fn delete(ctx: &Context) -> OxideResponse {
    let id = ctx.param("id").unwrap_or("0");
    let (status, _) = BufferBuilder::NO_CONTENT;
    return OxideResponse::new(
        BufferBuilder::deleted()
            .text(format!("Deleted data for ID: {}", id))
            .build(),
        status,
    );
}

// Middleware functions
fn global_middleware(ctx: Context) -> MiddlewareResult {
    let logger = Logger::new();
    logger.log(LogLevel::Info, "Global Middleware executed");
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

// Route setup functions
fn user_routes(server: &mut Server) {
    server.router.get("/users/:id", get_user_handler);
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

fn register_middleware(server: &mut Server) {
    server.middleware.add_global(global_middleware);
    server
        .middleware
        .for_route("/api/data/*", specific_middleware);
}

// Database example functions
async fn example_insert(db: &Database) -> Result<(), Error> {
    let user: Option<User> = User::query()
        .select([User::columns().name, User::columns().email])
        .and_where(User::columns().id, 1)
        .fetch_optional(db)
        .await?;

    println!("User: {:#?}", user);
    Ok(())
}

fn example_queries() {
    let basic_query = User::query()
        .and_where(User::columns().age, 25)
        .and_where(User::columns().active, true)
        .or_where(User::columns().email, "test@example.com".to_string())
        .build();
    println!("Basic Query: {}", basic_query);

    let complex_query = User::query()
        .select([User::columns().name])
        .and_where(User::columns().active, true)
        .and_group(|q| {
            q.and_where(User::columns().age, 25)
                .or_where(User::columns().age, 30)
        })
        .or_group(|q| {
            q.and_where(User::columns().email, "test@example.com".to_string())
                .and_where(User::columns().name, "John".to_string())
        })
        .build();
    println!("Complex Query: {}", complex_query);

    let nested_query = User::query()
        .and_group(|q| {
            q.and_where(User::columns().age, 25).or_group(|q| {
                q.and_where(User::columns().email, "test@example.com".to_string())
                    .and_where(User::columns().active, true)
            })
        })
        .or_group(|q| {
            q.and_where(User::columns().name, "John".to_string())
                .and_where(User::columns().age, 30)
        })
        .build();
    println!("Nested Query: {}", nested_query);

    let insert = User::insert()
        .value(User::columns().name, "John Doe".to_string())
        .value(User::columns().email, "test@example.com".to_string())
        .value(User::columns().age, 30)
        .value(User::columns().active, true)
        .build();
    println!("Insert Query: {}", insert);
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = Database::connect("postgres://oxide:oxide123@localhost:5432/oxide")
        .await
        .unwrap();

    let mut server = Server::new(Config::default());
    server.static_file("/", "index.html");

    let create_table = "
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL UNIQUE,
            age INTEGER NOT NULL,
            active BOOLEAN NOT NULL DEFAULT true
        )
    ";
    match db.execute(create_table.to_string()).await {
        Ok(_) => println!("Table created"),
        Err(e) => println!("{:?}", e),
    }

    // Insert a test user
    let insert = User::insert()
        .value(User::columns().name, "John Doe".to_string())
        .value(User::columns().email, "john@example.com".to_string())
        .value(User::columns().age, 30)
        .value(User::columns().active, true)
        .build();

    println!("Insert query: {}", insert);
    match db.execute(insert).await {
        Ok(_) => println!("User inserted"),
        Err(e) => println!("{:?}", e),
    }

    let res = db
        .query::<User>(format!("SELECT * FROM {}", User::TABLE))
        .await;

    match res {
        Ok(users) => println!("Users: {:#?}", users),
        Err(e) => println!("{:?}", e),
    }

    // Set up routes and middleware
    routes(&mut server);
    register_middleware(&mut server);
    user_routes(&mut server);

    // Query example
    let query = User::query()
        .and_where(User::columns().email, "john@example.com".to_string())
        .and_where(User::columns().id, 8)
        .build();

    println!("Select query: {}", query);
    let user = db.query_one::<User>(query).await.expect("msg");

    let update = User::update(user.id)
        .set(User::columns().email, "john@updated.com".to_string())
        .build();

    println!("Update query: {}", update);
    match db.execute(update).await {
        Ok(_) => println!("User updated"),
        Err(e) => println!("{:?}", e),
    }

    server.run().await
}

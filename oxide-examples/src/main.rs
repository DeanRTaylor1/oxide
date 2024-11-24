use oxide_core::{
    http::{AsyncResponse, BufferBuilder, Context, MiddlewareResult, OxideRes, OxideResponse},
    logger::LogLevel,
    prelude::*,
};
use oxide_orm::{model, prelude::*, Database};

#[derive(Debug, serde::Deserialize)]
pub struct JsonData {
    message: String,
}

#[model]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub active: bool,
}

#[handler]
async fn get_user(ctx: &Context) -> OxideResponse {
    let user_id = ctx.param("id").unwrap_or("0");
    let db_conn = match Database::connect("postgres://oxide:oxide123@localhost:5432/oxide").await {
        Ok(db) => db,
        Err(e) => {
            return OxideResponse::text(
                OxideRes::BadRequest,
                format!("Failed to connect to database: {}", e),
            );
        }
    };

    let user_id = match user_id.parse::<i32>().ok() {
        Some(id) => id,
        None => {
            return OxideResponse::text(OxideRes::BadRequest, "Invalid ID".to_string());
        }
    };

    match User::query()
        .and_where(User::columns().id, user_id)
        .fetch_one::<User>(&db_conn)
        .await
    {
        Ok(user) => OxideResponse::json(OxideRes::Success, user),
        Err(e) => {
            return OxideResponse::text(OxideRes::ServerError, e.to_string());
        }
    }
}

#[handler]
async fn root(_ctx: &Context) -> OxideResponse {
    return OxideResponse::text(OxideRes::Success, "Hello, World!".to_string());
}

#[handler]
async fn user(ctx: &Context) -> OxideResponse {
    let db = Database::connect("postgres://oxide:oxide123@localhost:5432/oxide")
        .await
        .unwrap();

    let user_id = match ctx.param("id").and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            return OxideResponse::text(OxideRes::BadRequest, "Invalid ID".to_string());
        }
    };

    let user: Option<User> = match User::query()
        .and_where(User::columns().id, user_id)
        .fetch_optional(&db)
        .await
    {
        Ok(user) => user,
        Err(_e) => {
            return OxideResponse::text(OxideRes::BadRequest, "Failed to fetch user".to_string());
        }
    };

    return match user {
        Some(user) => OxideResponse::json(OxideRes::Success, user),
        None => OxideResponse::text(OxideRes::BadRequest, "User not found".to_string()),
    };
}

#[handler]
async fn cookies(ctx: &Context) -> OxideResponse {
    let cookies = ctx.request.cookies();
    match serde_json::to_string(&cookies) {
        Ok(json) => OxideResponse::json(OxideRes::Success, json),
        _ => {
            return OxideResponse::text(
                OxideRes::BadRequest,
                "Failed to serialize cookies".to_string(),
            );
        }
    }
}

#[handler]
async fn post(ctx: &Context) -> OxideResponse {
    match ctx.request.json_body::<JsonData>() {
        Some(body) => {
            println!("JSON body: {}", body.message);
            return OxideResponse::text(OxideRes::Created, "Data created successfully".to_string());
        }
        None => {
            let err_data = ErrorData {
                message: "Failed to parse JSON".to_string(),
            };
            return OxideResponse::json(OxideRes::BadRequest, err_data);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ErrorData {
    message: String,
}

#[handler]
async fn put(ctx: &Context) -> OxideResponse {
    let id = ctx.param("id").unwrap_or("0");
    return OxideResponse::text(OxideRes::Updated, format!("Updated data for ID: {}", id));
}

#[handler]
async fn delete(ctx: &Context) -> OxideResponse {
    let id = ctx.param("id").unwrap_or("0");
    OxideResponse::text(OxideRes::Deleted, format!("Deleted data for ID: {}", id))
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

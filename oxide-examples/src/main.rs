use oxide_core::{
    http::{AsyncResponse, Context},
    prelude::*,
};
use oxide_orm::{prelude::*, Database, Error};

trait Boxer {
    fn boxed(self, api_handler: ApiHandler) -> fn(&Context) -> AsyncResponse<'static>;
}

struct ApiHandler {
    handler: Result<UnboxedHandler, Error>,
}

impl ApiHandler {
    fn new(handler: Result<UnboxedHandler, Error>) -> Self {
        Self { handler }
    }
}

impl Boxer for ApiHandler {
    fn boxed(self, api_handler: ApiHandler) -> fn(&Context) -> AsyncResponse<'static> {
        move |ctx| match self.handler {
            Ok(handler) => Box::pin(handler(ctx)),
            Err(err) => Box::pin(async move { Err(err) }),
        }
    }
}

async fn user_handler(ctx: &Context) -> Result<Vec<u8>, Error> {
    let user_id = ctx.param("id").unwrap_or("0");
    let db_conn = Database::connect("postgres://oxide:oxide123@localhost:5432/oxide").await?;
    // convert user_id to i32
    let user_id = match user_id.parse::<i32>().ok() {
        Some(id) => id,
        None => return Ok(ResponseBuilder::bad_request().text("Invalid ID").build()),
    };

    match User::query()
        .and_where(User::columns().id, user_id)
        .fetch_one::<User>(&db_conn)
        .await
    {
        Ok(user) => Ok(ResponseBuilder::ok()
            .json(serde_json::to_string(&user).unwrap())
            .build()),
        Err(_) => Ok(ResponseBuilder::not_found().text("User not found").build()),
    }
}

fn get_user(ctx: &Context) -> AsyncResponse<'_> {
    Box::pin(user_handler(ctx))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // example();
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

    user_routes(&mut server);

    // Query the user back
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

    // routes(&mut server);
    // register_middleware(&mut server);
    server.run().await
}

fn user_routes(server: &mut Server) {
    server.router.get("/users/:id", get_user);
}

// #[derive(Debug, serde::Deserialize)]
// pub struct JsonData {
//     message: String,
// }

// fn root_handler(_ctx: &Context) -> Vec<u8> {
//     ResponseBuilder::ok_response("Hello from Dean's server!")
// }

// async fn user_handler(ctx: &Context) -> Result<Vec<u8>, Error> {
//     let db = Database::connect("postgres://oxide:oxide123@localhost:5432/oxide").await?;
//     let user_id = match ctx.param("id").and_then(|id| id.parse::<i32>().ok()) {
//         Some(id) => id,
//         None => return Ok(ResponseBuilder::bad_request().text("Invalid ID").build()),
//     };

//     let user: Option<User> = User::query()
//         .and_where(User::columns().id, user_id)
//         .fetch_optional(&db)
//         .await?;

//     match user {
//         Some(user) => Ok(ResponseBuilder::ok()
//             .json(serde_json::to_string(&user).unwrap())
//             .build()),
//         None => Ok(ResponseBuilder::not_found().text("User not found").build()),
//     }
// }

// fn cookies_handler(ctx: &Context) -> Vec<u8> {
//     let cookies = ctx.request.cookies();
//     match serde_json::to_string(&cookies) {
//         Ok(json) => ResponseBuilder::ok().json(json).build(),
//         Err(_) => ResponseBuilder::server_error()
//             .text("Failed to serialize cookies")
//             .build(),
//     }
// }

// fn post_handler(ctx: &Context) -> Vec<u8> {
//     match ctx.request.json_body::<JsonData>() {
//         Some(body) => {
//             println!("JSON body: {}", body.message);
//             ResponseBuilder::created_response("Hello from Dean's server!")
//         }
//         None => ResponseBuilder::bad_request().text("Bad Request").build(),
//     }
// }

// fn put_handler(ctx: &Context) -> Vec<u8> {
//     let id = ctx.param("id").unwrap_or("0");
//     ResponseBuilder::created()
//         .text(format!("Updated data for ID: {}", id))
//         .build()
// }

// fn delete_handler(ctx: &Context) -> Vec<u8> {
//     let id = ctx.param("id").unwrap_or("0");
//     ResponseBuilder::deleted()
//         .text(format!("Deleted data for ID: {}", id))
//         .build()
// }

// fn routes(server: &mut Server) {
//     let mut api = server.router.group("/api");

//     let mut data = api.group("/data");

//     data.put("/:id", put_handler).delete("/:id", delete_handler);

//     let mut user_group = api.group("/user");

//     user_group
//         .get("/:id", user_handler)
//         .post("/", post_handler)
//         .delete("/:id", delete_handler);

//     server
//         .router
//         .get("/api", root_handler)
//         .get("/user/:id", user_handler)
//         .get("/cookies", cookies_handler)
//         .post("/api", post_handler)
//         .add_group(data)
//         .add_group(user_group);
// }

// fn global_middleware(ctx: Context) -> MiddlewareResult {
//     let logger = Logger::new();
//     logger.log(
//         oxide_core::logger::LogLevel::Info,
//         "Global Middleware executed",
//     );
//     Ok(ctx)
// }

// fn specific_middleware(ctx: Context) -> MiddlewareResult {
//     let logger = Logger::new();
//     logger.log(
//         LogLevel::Info,
//         format!(
//             "Specific Middleware executed in route: {}",
//             ctx.request.path
//         )
//         .as_str(),
//     );
//     Ok(ctx)
// }

// fn register_middleware(server: &mut Server) {
//     server.middleware.add_global(global_middleware);
//     server
//         .middleware
//         .for_route("/api/data/*", specific_middleware);
// }

#[derive(sqlx::FromRow, Model, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub active: bool,
}

// async fn example_insert(db: &Database) -> Result<(), Error> {
//     let user: Option<User> = User::query()
//         .select([User::columns().name, User::columns().email])
//         .and_where(User::columns().id, 1)
//         .fetch_optional(db)
//         .await?;

//     println!("User: {:#?}", user);
//     Ok(())
// }

// fn example() {
//     let basic_query = User::query()
//         .and_where(User::columns().age, 25)
//         .and_where(User::columns().active, true)
//         .or_where(User::columns().email, "test@example.com".to_string())
//         .build();
//     // SELECT * FROM users WHERE age = 25 AND active = true OR email = 'test@example.com'

//     println!("Query: {}", basic_query);

//     // Complex grouped conditions
//     let complex_query = User::query()
//         .select([User::columns().name])
//         .and_where(User::columns().active, true)
//         .and_group(|q| {
//             q.and_where(User::columns().age, 25)
//                 .or_where(User::columns().age, 30)
//         })
//         .or_group(|q| {
//             q.and_where(User::columns().email, "test@example.com".to_string())
//                 .and_where(User::columns().name, "John".to_string())
//         })
//         .build();
//     // SELECT * FROM users
//     // WHERE active = true
//     // AND (age = 25 OR age = 30)
//     // OR (email = 'test@example.com' AND name = 'John')

//     println!("Query: {}", complex_query);

//     // Deeply nested conditions
//     let query = User::query()
//         .and_group(|q| {
//             q.and_where(User::columns().age, 25).or_group(|q| {
//                 q.and_where(User::columns().email, "test@example.com".to_string())
//                     .and_where(User::columns().active, true)
//             })
//         })
//         .or_group(|q| {
//             q.and_where(User::columns().name, "John".to_string())
//                 .and_where(User::columns().age, 30)
//         })
//         .build();

//     println!("Query: {}", query);

//     // SELECT * FROM users
//     // WHERE (email = 'test@example.com' OR email = 'other@example.com')

//     let insert = User::insert()
//         .value(User::columns().name, "John Doe".to_string())
//         .value(User::columns().email, "test@example.com".to_string())
//         .value(User::columns().age, 30)
//         .value(User::columns().active, true)
//         .build();

//     println!("Insert: {}", insert);
// }

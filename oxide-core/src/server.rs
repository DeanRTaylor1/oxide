use crate::{
    config::Config,
    connection::Connection,
    http::{HttpHandler, MiddlewareHandler, RouteManager},
    logger::LogLevel,
    Logger, PgDatabase,
};
use std::{collections::HashMap, io, sync::Arc};
use tokio::net::TcpListener;

pub struct Server {
    pub router: RouteManager,
    pub middleware: MiddlewareHandler,

    config: Config,
    logger: Logger,
    http_handler: Option<Arc<HttpHandler>>,
    static_files: HashMap<String, &'static str>,
    datasource: Option<PgDatabase>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            logger: Logger::new(),
            router: RouteManager::new(),
            http_handler: None,
            middleware: MiddlewareHandler::new(),
            static_files: HashMap::new(),
            datasource: None,
        }
    }

    pub fn with_datasource(&mut self, datasource: PgDatabase) -> &mut Self {
        self.datasource = Some(datasource);
        self
    }

    pub fn static_file(&mut self, route: &str, file_path: &'static str) {
        self.static_files.insert(route.to_string(), file_path);
    }

    pub async fn run(&mut self) -> io::Result<()> {
        if self.router.routes().len() == 0 {
            self.logger.log(
                LogLevel::Application,
                "No routes have been registered, you can register routes by calling the `get`, `post`, `put` and `delete` methods on the route manager.",
            )
        }

        let shared_router = Arc::new(std::mem::take(&mut self.router));
        let shared_middleware = Arc::new(std::mem::take(&mut self.middleware));
        let static_files = Arc::new(std::mem::take(&mut self.static_files));

        let datasource = match &self.datasource {
            Some(db) => Some(Arc::new(db.clone())),
            None => None,
        };

        self.http_handler = Some(Arc::new(HttpHandler::new(
            shared_router,
            shared_middleware,
            static_files,
            datasource,
        )));

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        self.logger.log(
            LogLevel::Info,
            &format!("Server is listening on Port: {}", self.config.port),
        );

        loop {
            let (socket, _addr) = listener.accept().await?;
            let handler = Arc::clone(self.http_handler.as_ref().unwrap());
            tokio::spawn(async move {
                if let Err(e) = Connection::new(socket, handler).unwrap().process().await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
}

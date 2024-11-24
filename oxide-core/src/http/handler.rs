use std::{collections::HashMap, sync::Arc};

use serde::Serialize;

use crate::{logger::LogLevel, Logger, PgDatabase};

use super::{
    files::StaticHandler, BufferBuilder, HttpMethod, HttpRequest, MiddlewareHandler, RouteManager,
};

// pub type OxideResponse = OxideResult<Vec<u8>>;
pub struct OxideResponse {
    buffer: Vec<u8>,
    status: u16,
}

pub enum OxideRes {
    Success,     // 200
    NotFound,    // 404
    BadRequest,  // 400
    ServerError, // 500
    Created,     // 201
    Deleted,     // 200
    NoContent,   // 204
    Updated,     // 201
}

impl OxideResponse {
    pub fn json<T: Serialize>(response_type: OxideRes, data: T) -> Self {
        let status = Self::get_status(&response_type);
        let json_string = serde_json::to_string(&data).unwrap_or_default();
        let builder = Self::get_buffer_with_status(response_type);
        let buffer = builder.json(json_string).build();

        Self { buffer, status }
    }

    pub fn text(response_type: OxideRes, message: impl AsRef<str>) -> Self {
        let status = Self::get_status(&response_type);
        let builder = Self::get_buffer_with_status(response_type);

        let buffer = builder.text(message.as_ref()).build();

        Self { buffer, status }
    }

    fn get_buffer_with_status(response_type: OxideRes) -> BufferBuilder {
        return match response_type {
            OxideRes::Success => BufferBuilder::ok(),
            OxideRes::NotFound => BufferBuilder::not_found(),
            OxideRes::BadRequest => BufferBuilder::bad_request(),
            OxideRes::ServerError => BufferBuilder::server_error(),
            OxideRes::Created => BufferBuilder::created(),
            OxideRes::Deleted => BufferBuilder::deleted(),
            OxideRes::NoContent => BufferBuilder::no_content(),
            OxideRes::Updated => BufferBuilder::updated(),
        };
    }

    fn get_status(response_type: &OxideRes) -> u16 {
        let status = match response_type {
            OxideRes::Success => 200,
            OxideRes::NotFound => 404,
            OxideRes::BadRequest => 400,
            OxideRes::ServerError => 500,
            OxideRes::Created => 201,
            OxideRes::Deleted => 200,
            OxideRes::NoContent => 204,
            OxideRes::Updated => 201,
        };

        status
    }
}

pub struct RequestResponse {
    pub method: HttpMethod,
    pub path: String,
    pub ip: String,
    pub status: u16,
    pub duration: std::time::Duration,
}

pub struct Res {
    pub buffer: Vec<u8>,
    pub status: u16,
}

impl Res {
    pub fn new(buffer: Vec<u8>, status: u16) -> Self {
        Self { buffer, status }
    }
}

#[derive(Debug)]
pub struct HttpHandler {
    routes: Arc<RouteManager>,
    middleware: Arc<MiddlewareHandler>,
    static_files: Arc<HashMap<String, &'static str>>,
    datasource: Option<Arc<PgDatabase>>,
}

impl HttpHandler {
    pub fn new(
        router: Arc<RouteManager>,
        middleware: Arc<MiddlewareHandler>,
        static_files: Arc<HashMap<String, &'static str>>,
        datasource: Option<Arc<PgDatabase>>,
    ) -> Self {
        Self {
            routes: router,
            middleware,
            static_files,
            datasource,
        }
    }

    pub fn with_datasource(mut self, datasource: Arc<PgDatabase>) -> Self {
        self.datasource = Some(datasource);
        self
    }

    pub async fn handle(&self, buffer: &[u8]) -> Res {
        match HttpRequest::parse(buffer) {
            Some(request) => {
                if let Some(file_path) = self.static_files.get(&request.path) {
                    if let Some((data, mime)) = StaticHandler::serve(file_path) {
                        return Res::new(
                            BufferBuilder::ok()
                                .header("Content-Type", mime.as_str())
                                .body(data)
                                .build(),
                            200,
                        );
                    }
                }

                if let Some(route) = self.routes.find_route(&request.path, request.method) {
                    let params = self.extract_params(&route.pattern, &request.path);
                    let mut context = Context::new(request, params);
                    if let Some(db) = &self.datasource {
                        context.with_datasource(Arc::clone(db));
                    }
                    match self.middleware.run(context, route) {
                        Ok(ctx) => {
                            let logger = Logger::new();

                            let res = (route.handler)(&ctx).await;
                            logger.log(LogLevel::Info, format!("status: {}", res.status,).as_str());
                            return Res::new(res.buffer, res.status);
                        }
                        Err(res) => res,
                    }
                } else {
                    Res::new(BufferBuilder::not_found().text("Not Found").build(), 404)
                }
            }
            None => Res::new(
                BufferBuilder::bad_request().text("Bad Request").build(),
                400,
            ),
        }
    }

    fn extract_params(&self, pattern: &str, path: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let pattern_parts: Vec<_> = pattern.split('/').collect();
        let path_parts: Vec<_> = path.split('/').collect();

        for (p, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if p.starts_with(':') {
                params.insert(p[1..].to_string(), path_part.to_string());
            }
        }
        params
    }
}

pub struct Context {
    pub request: HttpRequest,
    params: HashMap<String, String>,
    pub datasource: Option<Arc<PgDatabase>>,
}

impl Context {
    pub fn new(request: HttpRequest, params: HashMap<String, String>) -> Self {
        Self {
            request,
            params,
            datasource: None,
        }
    }

    pub fn with_datasource(&mut self, datasource: Arc<PgDatabase>) -> &mut Self {
        self.datasource = Some(datasource);
        self
    }

    pub fn db(&self) -> Option<&PgDatabase> {
        self.datasource.as_ref().map(|db| db.as_ref())
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }
}

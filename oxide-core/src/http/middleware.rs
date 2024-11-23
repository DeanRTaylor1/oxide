use std::collections::HashMap;

use crate::Logger;

use super::{handler::Res, routes::Route, Context};

pub type MiddlewareResult = Result<Context, Res>;
pub type MiddlewareFn = fn(Context) -> MiddlewareResult;

#[derive(Debug, Default)]
pub struct MiddlewareHandler {
    global: Vec<MiddlewareFn>,
    route_specific: HashMap<String, Vec<MiddlewareFn>>,
}

impl MiddlewareHandler {
    pub fn new() -> Self {
        Self {
            global: Vec::new(),
            route_specific: HashMap::new(),
        }
    }

    pub fn add_global(&mut self, middleware: MiddlewareFn) {
        self.global.push(middleware);
    }

    pub fn for_route(&mut self, pattern: &str, middleware: MiddlewareFn) {
        let path = pattern
            .split('/')
            .take_while(|s| !s.starts_with('*'))
            .collect::<Vec<_>>()
            .join("/");

        let logger = Logger::new();
        logger.log(
            crate::logger::LogLevel::Info,
            &format!("Registering middleware for route: {}", path),
        );

        self.route_specific
            .entry(path.to_string())
            .or_default()
            .push(middleware);
    }

    pub fn run(&self, mut context: Context, route: &Route) -> MiddlewareResult {
        for middleware in &self.global {
            context = middleware(context)?;
        }

        if let Some(middlewares) = self.route_specific.get(&route.raw_path) {
            for middleware in middlewares {
                context = middleware(context)?;
            }
        }

        Ok(context)
    }
}

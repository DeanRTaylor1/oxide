use crate::{logger, Logger};

use super::{handler::Context, HttpMethod};

#[derive(Debug, Clone, Default)]
pub struct RouteManager {
    routes: Vec<Route>,
    logger: Logger,
}

impl RouteManager {
    pub fn new() -> Self {
        Self {
            routes: vec![],
            logger: Logger::new(),
        }
    }

    pub fn routes(&self) -> &Vec<Route> {
        &self.routes
    }

    pub fn group(&mut self, prefix: &str) -> RouteGroup {
        RouteGroup::new(prefix)
    }

    pub fn add_group(&mut self, group: RouteGroup) -> &mut Self {
        for route in &group.routes {
            self.logger.log(
                crate::logger::LogLevel::Info,
                &format!(
                    "Route registered successfully: {} | {}",
                    route.method, route.pattern
                ),
            )
        }
        self.routes.extend(group.routes);
        self
    }

    pub fn get(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) -> &mut Self {
        self.add_route(Route::new(path, HttpMethod::Get, handler));
        self
    }

    pub fn post(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) -> &mut Self {
        self.add_route(Route::new(path, HttpMethod::Post, handler));
        self
    }

    pub fn put(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) -> &mut Self {
        self.add_route(Route::new(path, HttpMethod::Put, handler));
        self
    }

    pub fn delete(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) -> &mut Self {
        self.add_route(Route::new(path, HttpMethod::Delete, handler));
        self
    }

    pub fn apply_routes(&mut self, router: RouteManager) -> &mut Self {
        for route in router.routes() {
            self.logger.log(
                crate::logger::LogLevel::Info,
                &format!("Added route: {}", route.pattern),
            );
            self.add_route(route.clone());
        }
        self
    }

    fn add_route(&mut self, route: Route) -> &mut Self {
        self.logger.log(
            crate::logger::LogLevel::Info,
            &format!(
                "Route registered successfully: {} | {}",
                route.method, route.pattern
            ),
        );
        self.routes.push(route);
        self
    }

    pub fn find_route(&self, path: &str, method: HttpMethod) -> Option<&Route> {
        self.routes
            .iter()
            .find(|r| r.method == method && r.matches(path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub pattern: String,
    pub raw_path: String,
    pub path_params: Vec<String>,
    pub method: HttpMethod,
    pub handler: fn(&Context) -> Vec<u8>,
}

impl Route {
    pub fn new(pattern: &str, method: HttpMethod, handler: fn(&Context) -> Vec<u8>) -> Self {
        let path_params = pattern
            .split('/')
            .filter(|s| s.starts_with(':'))
            .map(|s| s[1..].to_string())
            .collect();

        let raw_path = pattern
            .split('/')
            .take_while(|s| !s.starts_with(':'))
            .collect::<Vec<_>>()
            .join("/");

        Self {
            pattern: pattern.to_string(),
            raw_path,
            path_params,
            method,
            handler,
        }
    }

    fn matches(&self, path: &str) -> bool {
        let pattern_parts: Vec<&str> = self.pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        pattern_parts
            .iter()
            .zip(path_parts.iter())
            .all(|(p, u)| p.starts_with(':') || p == u)
    }
}

pub struct RouteGroup {
    prefix: String,
    routes: Vec<Route>,
}

impl RouteGroup {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            routes: vec![],
        }
    }

    pub fn get(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) -> &mut Self {
        let full_path = format!("{}{}", self.prefix, path);
        self.routes
            .push(Route::new(&full_path, HttpMethod::Get, handler));
        self
    }

    pub fn post(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) -> &mut Self {
        let full_path = format!("{}{}", self.prefix, path);
        self.routes
            .push(Route::new(&full_path, HttpMethod::Post, handler));
        self
    }

    pub fn put(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) -> &mut Self {
        let full_path = format!("{}{}", self.prefix, path);
        self.routes
            .push(Route::new(&full_path, HttpMethod::Put, handler));
        self
    }

    pub fn delete(&mut self, path: &str, handler: fn(&Context) -> Vec<u8>) -> &mut Self {
        let full_path = format!("{}{}", self.prefix, path);
        self.routes
            .push(Route::new(&full_path, HttpMethod::Delete, handler));
        self
    }

    pub fn group(&mut self, prefix: &str) -> RouteGroup {
        RouteGroup::new(&format!("{}{}", self.prefix, prefix))
    }
}

use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Unknown,
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            _ => Ok(HttpMethod::Unknown),
        }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub query_params: Params,
    pub path_params: Params,
    pub cookies: Cookies,
}

type Cookies = HashMap<String, String>;
type Params = HashMap<String, String>;

impl HttpRequest {
    pub fn new(
        method: HttpMethod,
        path: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        query_params: Params,
        path_params: Params,
        cookies: Cookies,
    ) -> Self {
        Self {
            method,
            path,
            headers,
            body,
            query_params,
            path_params,
            cookies,
        }
    }

    /** Public interface */

    pub fn cookies(&self) -> &Cookies {
        &self.cookies
    }

    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }

    pub fn content_length(&self) -> Option<usize> {
        self.headers.get("content-length")?.parse().ok()
    }

    pub fn json_body<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        if self.content_type()? != "application/json" {
            return None;
        }
        serde_json::from_slice(&self.body).ok()
    }

    /** Static interface */
    pub fn parse(buffer: &[u8]) -> Option<HttpRequest> {
        let request = std::str::from_utf8(buffer).ok()?;
        let mut parts = request.split("\r\n\r\n");
        let headers_part = parts.next()?;
        let body_part = parts.next().unwrap_or("");

        let mut lines = headers_part.lines();
        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();

        let method = HttpMethod::from_str(parts.next()?).ok()?;
        let path = parts.next()?.to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_lowercase(), value.to_string());
            }
        }

        let cookies = match headers.get("cookie") {
            Some(cookie_str) => HttpRequest::parse_cookies(cookie_str.as_str()),
            None => HashMap::new(),
        };

        let body = body_part.as_bytes().to_vec();

        let query_params = HttpRequest::parse_query_params(&path.as_str());
        let path_params = HttpRequest::parse_path_params(&path.as_str(), &path);

        Some(HttpRequest {
            method,
            path,
            headers,
            body,
            query_params,
            path_params,
            cookies,
        })
    }

    /** Private interface */
    fn parse_path_params(path: &str, pattern: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();

        for (pattern, value) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern.starts_with(':') {
                params.insert(pattern[1..].to_string(), value.to_string());
            }
        }
        params
    }

    fn parse_cookies(cookie: &str) -> Cookies {
        let mut cookies = HashMap::new();
        for pair in cookie.split(';') {
            if let Some((key, value)) = pair.split_once('=') {
                cookies.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        cookies
    }

    fn parse_query_params(path: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if let Some(query) = path.split_once('?').map(|(_, q)| q) {
            for pair in query.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    params.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        params
    }
}

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Method, StatusCode};
use regex::Regex;

use controller::Controller;
use context::Captures;


struct Route {
    pattern: Regex,
    handler: Arc<Controller>,
}


/// Builder for RouteRecognizer.
#[derive(Default)]
pub struct RoutesBuilder {
    routes: HashMap<Method, Vec<Route>>,
}

impl RoutesBuilder {
    pub fn route<S, H>(mut self, method: Method, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Controller,
    {
        let pattern = normalize_pattern(pattern.as_ref());
        let pattern = Regex::new(&pattern).unwrap();
        let handler = Arc::new(handler);
        self.routes
            .entry(method)
            .or_insert(Vec::new())
            .push(Route { pattern, handler });
        self
    }

    /// Add handler for 'GET' method
    pub fn get<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Controller,
    {
        self.route(Method::Get, pattern, handler)
    }

    /// Add handler for 'POST' method
    pub fn post<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Controller,
    {
        self.route(Method::Post, pattern, handler)
    }

    /// Add handler for 'PUT' method
    pub fn put<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Controller,
    {
        self.route(Method::Put, pattern, handler)
    }

    /// Add handler for 'DELETE' method
    pub fn delete<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Controller,
    {
        self.route(Method::Delete, pattern, handler)
    }

    /// Add handler for 'HEAD' method
    pub fn head<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Controller,
    {
        self.route(Method::Head, pattern, handler)
    }

    /// Add handler for 'OPTIONS' method
    pub fn options<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Controller,
    {
        self.route(Method::Options, pattern, handler)
    }

    /// Create recoginizer
    pub fn finish(self) -> Router {
        Router { routes: self.routes }
    }
}



pub struct Router {
    routes: HashMap<Method, Vec<Route>>,
}

impl Router {
    pub fn recognize(
        &self,
        method: &Method,
        path: &str,
    ) -> Result<(Arc<Controller>, Captures), StatusCode> {
        let routes = self.routes.get(method).ok_or(
            StatusCode::NotFound,
        )?;
        for route in routes {
            if let Some(caps) = get_owned_captures(&route.pattern, path) {
                return Ok((route.handler.clone(), caps));
            }
        }
        Err(StatusCode::NotFound)
    }
}



fn get_owned_captures(re: &Regex, path: &str) -> Option<Vec<(Option<String>, String)>> {
    re.captures(path).map(|caps| {
        let mut res = Vec::with_capacity(caps.len());
        for (i, name) in re.capture_names().enumerate() {
            let val = match name {
                Some(name) => caps.name(name).unwrap(),
                None => caps.get(i).unwrap(),
            };
            res.push((name.map(|s| s.to_owned()), val.as_str().to_owned()));
        }
        res
    })
}

fn normalize_pattern(pattern: &str) -> Cow<str> {
    let pattern = pattern
        .trim()
        .trim_left_matches("^")
        .trim_right_matches("$")
        .trim_right_matches("/");
    match pattern {
        "" => "^/$".into(),
        s => format!("^{}/?$", s).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_pattern;

    #[test]
    fn normalize_cases() {
        assert_eq!(normalize_pattern("/"), "^/$");
        assert_eq!(normalize_pattern("/path/to"), "^/path/to/?$");
        assert_eq!(normalize_pattern("/path/to/"), "^/path/to/?$");
    }
}

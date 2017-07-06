use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Method, StatusCode};
use regex::Regex;

use context::Captures;
use controller::Controller;

struct Route {
    pattern: Regex,
    handler: Arc<Controller>,
}


#[derive(Default)]
pub struct Router {
    routes: HashMap<Method, Vec<Route>>,
}

impl Router {
    pub fn add_route<S, H>(&mut self, method: Method, pattern: S, handler: H) -> &mut Self
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

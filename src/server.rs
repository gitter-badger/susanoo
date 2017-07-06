use std::io;
use std::sync::Arc;

use futures::{future, Future};
use futures::future::BoxFuture;
use hyper::Error as HyperError;
use hyper::{Method, StatusCode};
use hyper::server::{Http, Service, NewService, Response};
use hyper::server::Request;
use typemap::{TypeMap, Key};
use unsafe_any::UnsafeAny;

use context::Context;
use controller::Controller;
use middleware::Middleware;
use router::Router;


pub type State = TypeMap<UnsafeAny + 'static + Send + Sync>;


/// Internal state of server
#[doc(visible)]
pub(crate) struct ServerInner {
    router: Router,
    middlewares: Vec<Arc<Middleware>>,
    state: Arc<State>,
}


pub struct Server {
    inner: Arc<ServerInner>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            inner: Arc::new(ServerInner {
                router: Router::default(),
                middlewares: Vec::new(),
                state: Arc::new(State::custom()),
            }),
        }
    }

    pub fn with_route<S, H>(mut self, method: Method, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Controller,
    {
        Arc::get_mut(&mut self.inner)
            .unwrap()
            .router
            .add_route(method, pattern, handler);
        self
    }


    pub fn with_middleware<M: Middleware>(mut self, middleware: M) -> Self {
        Arc::get_mut(&mut self.inner)
            .unwrap()
            .middlewares
            .push(Arc::new(middleware));
        self
    }

    pub fn insert<T: Key<Value = T> + Send + Sync>(mut self, value: T) -> Self {
        {
            let inner = Arc::get_mut(&mut self.inner).unwrap();
            let mut state = Arc::get_mut(&mut inner.state).unwrap();
            state.insert::<T>(value);
        }
        self
    }

    pub fn run(self, addr: &str) {
        let addr = addr.parse().unwrap();
        let http = Http::new().bind(&addr, self).unwrap();
        http.run().unwrap();
    }
}

impl NewService for Server {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Instance = RootService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(RootService { inner: self.inner.clone() })
    }
}


/// An asynchronous task executed by hyper.
pub struct RootService {
    inner: Arc<ServerInner>,
}

impl Service for RootService {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Future = BoxFuture<Response, HyperError>;

    fn call(&self, req: Request) -> Self::Future {
        // apply router
        let method = req.method().clone();
        let path = req.path().to_owned();
        match self.inner.router.recognize(
            &method,
            &path,
        ) {
            Ok((controller, cap)) => {
                let ctx = future::ok(Context::new(req, cap, self.inner.state.clone())).boxed();

                // apply middlewares
                let ctx = self.inner.middlewares.iter().fold(
                    ctx,
                    |ctx, middleware| {
                        let middleware = middleware.clone();
                        ctx.and_then(move |ctx| middleware.call(ctx))
                            .boxed()
                    },
                );

                // apply controller
                let ctx = ctx.and_then(move |ctx| controller.call(ctx));

                // convert to Hyper response
                ctx.then(|resp| match resp {
                    Ok(resp) => Ok(resp),
                    Err(failure) => Ok(
                        failure.response.unwrap_or(
                            Response::new()
                                .with_status(StatusCode::InternalServerError)
                                .with_body(format!("Internal Server Error: {:?}", failure.err)),
                        ),
                    ),
                }).boxed()
            }
            Err(code) => future::ok(Response::new().with_status(code)).boxed(),
        }
    }
}

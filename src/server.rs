use std::io;
use std::sync::Arc;

use futures::{future, Future};
use futures::future::BoxFuture;
use hyper::Error as HyperError;
use hyper::server::{Http, Service, NewService, Request, Response};
use typemap::{ShareMap, TypeMap};

use context::Context;
use router::Router;


// TODO: use typemap
pub type State = ShareMap;


/// Internal state of server
#[doc(visible)]
pub(crate) struct ServerInner {
    router: Router,
    state: Arc<State>,
}


pub struct Server {
    inner: Arc<ServerInner>,
}

impl Server {
    pub fn new(router: Router, state: Option<State>) -> Self {
        Server {
            inner: Arc::new(ServerInner {
                router,
                state: Arc::new(state.unwrap_or_else(|| State::custom())),
            }),
        }
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
        let method = req.method().clone();
        let path = req.path().to_owned();
        match self.inner.router.recognize(
            &method,
            &path,
        ) {
            Ok((controller, cap)) => {
                let ctx = Context {
                    req,
                    cap,
                    map: TypeMap::new(),
                    state: self.inner.state.clone(),
                };
                controller.call(ctx)
            }
            Err(code) => future::ok(Response::new().with_status(code)).boxed(),
        }
    }
}

use hyper::server::Response;
use hyper::Error as HyperError;
use futures::future::BoxFuture;
use context::Context;

pub trait Controller: 'static + Send + Sync {
    fn call(&self, ctx: Context) -> BoxFuture<Response, HyperError>;
}

impl<F> Controller for F
where
    F: 'static + Send + Sync + Fn(Context) -> BoxFuture<Response, HyperError>,
{
    fn call(&self, ctx: Context) -> BoxFuture<Response, HyperError> {
        (*self)(ctx)
    }
}

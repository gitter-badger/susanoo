use std::sync::Arc;
use context::Context;
use response::{Success, AsyncResult};
use futures::{future, Future};

pub trait Middleware: 'static + Send + Sync {
    fn call(&self, ctx: Context) -> AsyncResult;
}

impl<F> Middleware for F
where
    F: 'static + Send + Sync + Fn(Context) -> AsyncResult,
{
    fn call(&self, ctx: Context) -> AsyncResult {
        (*self)(ctx)
    }
}


/// The chain of middlewares.
#[derive(Default)]
pub struct MiddlewareStack {
    middlewares: Vec<Arc<Middleware>>,
}

impl MiddlewareStack {
    pub fn with<M: Middleware>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }
}

impl Middleware for MiddlewareStack {
    fn call(&self, ctx: Context) -> AsyncResult {
        self.middlewares.iter().fold(
            future::ok(ctx.into())
                .boxed(),
            |ctx, middleware| {
                let middleware = middleware.clone();
                ctx.and_then(move |ctx| match ctx {
                    Success::Continue(ctx) => middleware.call(ctx),
                    Success::Finished(resp) => future::ok(resp.into()).boxed(),
                }).boxed()
            },
        )
    }
}

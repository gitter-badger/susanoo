use context::Context;
use response::{Response, AsyncResult};

pub trait Controller: 'static + Send + Sync {
    fn call(&self, ctx: Context) -> AsyncResult<Response>;
}

impl<F> Controller for F
where
    F: 'static + Send + Sync + Fn(Context) -> AsyncResult<Response>,
{
    fn call(&self, ctx: Context) -> AsyncResult<Response> {
        (*self)(ctx)
    }
}

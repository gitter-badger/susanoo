use context::Context;
use response::AsyncResult;

pub trait Middleware: 'static + Send + Sync {
    fn call(&self, ctx: Context) -> AsyncResult<Context>;
}

impl<F> Middleware for F
where
    F: 'static + Send + Sync + Fn(Context) -> AsyncResult<Context>,
{
    fn call(&self, ctx: Context) -> AsyncResult<Context> {
        (*self)(ctx)
    }
}

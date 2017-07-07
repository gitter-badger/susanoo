use std::error::Error as StdError;
use futures::future::BoxFuture;
use context::Context;

/// response type
pub use hyper::server::Response;

/// asynchronous result type
pub type AsyncResult = BoxFuture<Success, Failure>;

/// success type
pub enum Success {
    Finished(Response),
    Continue(Context),
}

impl From<Response> for Success {
    fn from(response: Response) -> Success {
        Success::Finished(response)
    }
}

impl From<Context> for Success {
    fn from(context: Context) -> Success {
        Success::Continue(context)
    }
}


/// error type
pub struct Failure {
    pub err: Box<StdError + Send + 'static>,
    pub response: Option<Response>,
}

impl Failure {
    pub fn new<E: 'static + StdError + Send>(err: E) -> Failure {
        Failure {
            err: Box::new(err),
            response: None,
        }
    }

    pub fn with_response(mut self, response: Response) -> Self {
        self.response = Some(response);
        self
    }
}

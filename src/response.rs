use std::error::Error as StdError;
use futures::future::BoxFuture;

/// response type
pub use hyper::server::Response;

/// asynchronous result type
pub type AsyncResult<T = Response> = BoxFuture<T, Failure>;

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

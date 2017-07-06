use std::sync::Arc;
use hyper::server::Request;
use typemap::TypeMap;
use server::State;
use unsafe_any::UnsafeAny;

pub type Captures = Vec<(Option<String>, String)>;

pub struct Context {
    pub req: Request,
    pub cap: Captures,
    pub map: TypeMap<UnsafeAny + Send>,
    pub state: Arc<State>,
}

impl Context {
    pub fn new(req: Request, cap: Captures, state: Arc<State>) -> Self {
        Context {
            req,
            cap,
            map: TypeMap::custom(),
            state,
        }
    }
}

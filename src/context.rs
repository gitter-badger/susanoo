use std::sync::Arc;
use hyper::server::Request;
use typemap::TypeMap;
use server::States;
use unsafe_any::UnsafeAny;

pub type Captures = Vec<(Option<String>, String)>;

pub struct Context {
    pub req: Request,
    pub cap: Captures,
    pub map: TypeMap<UnsafeAny + Send>,
    pub states: Arc<States>,
}

impl Context {
    pub fn new(req: Request, cap: Captures, states: Arc<States>) -> Self {
        Context {
            req,
            cap,
            map: TypeMap::custom(),
            states,
        }
    }
}

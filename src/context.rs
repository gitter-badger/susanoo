use std::sync::Arc;
use hyper::server::Request;
use typemap::TypeMap;
use server::State;

pub type Captures = Vec<(Option<String>, String)>;

pub struct Context {
    pub req: Request,
    pub cap: Captures,
    pub map: TypeMap,
    pub state: Arc<State>,
}

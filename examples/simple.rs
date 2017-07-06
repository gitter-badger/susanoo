extern crate susanoo;

use susanoo::context::Context;
use susanoo::router::RoutesBuilder;
use susanoo::server::Server;
use susanoo::response::{Response, Failure, AsyncResult};
use susanoo::contrib::hyper::StatusCode;
use susanoo::contrib::futures::{future, Future, Stream};


fn index(_ctx: Context) -> AsyncResult {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("Hello, world"),
    ).boxed()
}

fn index_post(ctx: Context) -> AsyncResult {
    ctx.req
        .body()
        .collect()
        .and_then(|chunks| {
            let mut body = Vec::new();
            for chunk in chunks {
                body.extend_from_slice(&chunk);
            }
            future::ok(
                Response::new()
                    .with_status(StatusCode::Ok)
                    .with_body(format!("Posted: {}", String::from_utf8_lossy(&body))),
            )
        })
        .map_err(|err| Failure::new(err))
        .boxed()
}

fn show_captures(ctx: Context) -> AsyncResult {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("Captures: {:?}", ctx.cap)),
    ).boxed()
}

fn main() {
    let router = RoutesBuilder::default()
        .get("/", index)
        .post("/", index_post)
        .post("/post", index_post)
        .get(r"/echo/([^/]+)/(?P<hoge>[^/]+)/([^/]+)", show_captures)
        .finish();

    let server = Server::new(router, Vec::new(), None);
    server.run("0.0.0.0:4000");
}

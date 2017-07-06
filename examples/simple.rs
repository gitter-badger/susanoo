extern crate susanoo;

use susanoo::context::Context;
use susanoo::router::RoutesBuilder;
use susanoo::server::Server;
use susanoo::contrib::hyper::server::Response;
use susanoo::contrib::hyper::{Error as HyperError, StatusCode};
use susanoo::contrib::futures::{future, Future, Stream};
use susanoo::contrib::futures::future::BoxFuture;


fn index(_ctx: Context) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("Hello, world"),
    ).boxed()
}

fn index_post(ctx: Context) -> BoxFuture<Response, HyperError> {
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
        .boxed()
}

fn show_captures(ctx: Context) -> BoxFuture<Response, HyperError> {
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

    let server = Server::new(router);
    server.run("0.0.0.0:4000");
}

extern crate susanoo;

use susanoo::{Context, Server, Response, AsyncResult};
use susanoo::contrib::hyper::{Get, Post, StatusCode};
use susanoo::contrib::futures::{future, Future, Stream};


fn index(_ctx: Context) -> AsyncResult {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("Hello, world")
            .into(),
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
                    .with_body(format!("Posted: {}", String::from_utf8_lossy(&body)))
                    .into(),
            )
        })
        .map_err(Into::into)
        .boxed()
}

fn show_captures(ctx: Context) -> AsyncResult {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("Captures: {:?}", ctx.cap))
            .into(),
    ).boxed()
}

fn main() {
    let server = Server::new()
        .with_route(Get, "/", index)
        .with_route(Post, "/", index_post)
        .with_route(Post, "/post", index_post)
        .with_route(Get, r"/echo/([^/]+)/(?P<hoge>[^/]+)/([^/]+)", show_captures);
    server.run("0.0.0.0:4000");
}

extern crate susanoo;
#[macro_use]
extern crate hyper;

use susanoo::{Server, Context, Response, AsyncResult};
use susanoo::middleware::MiddlewareStack;
use susanoo::contrib::hyper::{Get, StatusCode};
use susanoo::contrib::hyper::header::{Authorization, Basic};
use susanoo::contrib::futures::{future, Future};
use susanoo::contrib::typemap::Key;

header! {
    (WWWAuthenticate, "WWW-Authenticate") => [String]
}


#[derive(Clone)]
struct User {
    username: String,
    password: String,
}

impl User {
    fn verify(&self, username: &str, password: &str) -> bool {
        &self.username == username && &self.password == password
    }
}

impl Key for User {
    type Value = Self;
}


struct UserList(Vec<User>);

impl std::ops::Deref for UserList {
    type Target = Vec<User>;
    fn deref(&self) -> &Vec<User> {
        &self.0
    }
}
impl Key for UserList {
    type Value = Self;
}



fn check_auth(mut ctx: Context) -> AsyncResult {
    {
        let auth = ctx.req.headers().get::<Authorization<Basic>>();
        let (username, password) = match auth {
            Some(&Authorization(Basic {
                                    ref username,
                                    password: Some(ref password),
                                })) => (username, password),
            _ => {
                return future::ok(
                    Response::new()
                        .with_status(StatusCode::Unauthorized)
                        .with_header(WWWAuthenticate("Basic realm=\"main\"".to_owned()))
                        .into(),
                ).boxed()
            }
        };

        let users = ctx.states.get::<UserList>().unwrap();
        let found: Option<User> = users
            .iter()
            .find(|&user| user.verify(username, password))
            .map(|u| u.clone());
        match found {
            Some(user) => {
                ctx.map.insert::<User>(user);
            }
            None => {
                return future::ok(
                    Response::new()
                        .with_status(StatusCode::Unauthorized)
                        .with_header(WWWAuthenticate("Basic realm=\"main\"".to_owned()))
                        .into(),
                ).boxed();
            }
        }
    }

    future::ok(ctx.into()).boxed()
}



fn index(ctx: Context) -> AsyncResult {
    let user = ctx.map.get::<User>().unwrap();
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("<h1>Welcome, {}!</h1>", user.username))
            .into(),
    ).boxed()
}

fn public(_ctx: Context) -> AsyncResult {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("<h1>Public page</h1>")
            .into(),
    ).boxed()
}

fn main() {
    let users = vec![
        User {
            username: "alice".to_owned(),
            password: "wonderland".to_owned(),
        },
    ];

    let index = MiddlewareStack::default().with(check_auth).with(
        index,
    );

    let server = Server::new()
        .with_route(Get, "/", index)
        .with_route(Get, "/public", public)
        .with_state(UserList(users));
    server.run("0.0.0.0:4000")
}

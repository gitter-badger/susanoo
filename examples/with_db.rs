extern crate susanoo;
extern crate typemap;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;


use susanoo::context::Context;
use susanoo::router::RoutesBuilder;
use susanoo::server::{Server, State};
use susanoo::contrib::hyper::server::Response;
use susanoo::contrib::hyper::{Error as HyperError, StatusCode};
use susanoo::contrib::futures::{future, Future};
use susanoo::contrib::futures::future::BoxFuture;

use std::ops::Deref;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;


struct DB(Pool<SqliteConnectionManager>);

impl Deref for DB {
    type Target = Pool<SqliteConnectionManager>;
    fn deref(&self) -> &Pool<SqliteConnectionManager> {
        &self.0
    }
}

impl typemap::Key for DB {
    type Value = DB;
}


#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}


fn index(ctx: Context) -> BoxFuture<Response, HyperError> {
    let db = ctx.state.get::<DB>().unwrap();
    let conn = db.get().unwrap();
    let mut stmt = conn.prepare("SELECT id,name,data FROM persons")
        .unwrap();
    let persons: Vec<_> = stmt.query_map(&[], |row| {
        Person {
            id: row.get(0),
            name: row.get(1),
            data: row.get(2),
        }
    }).unwrap()
        .collect();
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("persons: {:?}", persons)),
    ).boxed()
}


fn main() {
    let _ = std::fs::remove_file("app.sqlite");
    let manager = SqliteConnectionManager::new("app.sqlite");
    let pool = r2d2::Pool::new(Default::default(), manager).unwrap();
    {
        let conn = pool.get().unwrap();
        conn.execute(
            r#"CREATE TABLE persons (
                id    INTEGER   PRIMARY KEY
              , name  TEXT      NOT NULL
              , data  BLOB
            )"#,
            &[],
        ).unwrap();

        let me = Person {
            id: 0,
            name: "Bob".to_owned(),
            data: None,
        };
        conn.execute(
            "INSERT INTO persons (name, data) VALUES (?1, ?2)",
            &[&me.name, &me.data],
        ).unwrap();
    }
    let db = DB(pool);

    let router = RoutesBuilder::default()
        .get("/", index)
        .finish();

    let mut state = State::custom();
    state.insert::<DB>(db);

    let server = Server::new(router, Some(state));
    server.run("0.0.0.0:4000");
}
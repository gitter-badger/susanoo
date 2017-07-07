#[macro_use(try_f)]
extern crate susanoo;
extern crate typemap;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use susanoo::{Context, Server, Response, AsyncResult};
use susanoo::contrib::hyper::{Get, StatusCode};
use susanoo::contrib::futures::{future, Future};

use std::ops::Deref;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection as SqliteConnection;


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


fn index(ctx: Context) -> AsyncResult {
    let db = ctx.states.get::<DB>().unwrap();
    let conn = try_f!(db.get());
    let mut stmt = try_f!(conn.prepare("SELECT id,name,data FROM persons"));
    let persons: Vec<_> = try_f!(stmt.query_map(&[], |row| {
        Person {
            id: row.get(0),
            name: row.get(1),
            data: row.get(2),
        }
    })).collect();
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("persons: {:?}", persons))
            .into(),
    ).boxed()
}


fn init_db(path: &str) {
    let _ = std::fs::remove_file(path);

    let conn = SqliteConnection::open(path).unwrap();

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


fn main() {
    init_db("app.sqlite");

    // create DB connection pool
    let manager = SqliteConnectionManager::new("app.sqlite");
    let pool = r2d2::Pool::new(Default::default(), manager).unwrap();
    let db = DB(pool);

    let server = Server::new()
        .with_route(Get, "/", index)
        .with_state(db);

    server.run("0.0.0.0:4000");
}

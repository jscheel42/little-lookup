#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

extern crate dotenv;

pub mod schema;
pub mod models;

// extern crate self as littlelookup;

use diesel::prelude::*;
// use diesel::pg::PgConnection;
use diesel::sqlite::SqliteConnection;
// use diesel::connection::Connection;
use dotenv::dotenv;
use std::env;
use self::models::Item;

pub fn get_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}




// #[get("/")]
// fn index() -> &'static str {
//     "Routes:
//   /item/<key>: Get val for <key>
//   /item/<key>/<val>: Update <val> for <key>
// "
// }

// #[get("/item/<key>")]
// fn get_item(key: &rocket::http::RawStr) -> String {
//     // use sqlite::State;

//     let connection = get_connection();

//     let result = items.filter(key.eq(key.as_str()))
//         .limit(1)
//         .load::<Item>(&connection)
//         .expect("Error loading item");

//     item
// }

    // let query = format!(
    //     "SELECT val FROM items WHERE key = '{}' LIMIT ?;",
    //     key.as_str()
    // );

    // let mut statement = connection
    //     .prepare(query)
    //     .unwrap();

    // statement.bind(1, 1).unwrap();

    // let mut out: String = String::from("Undefined");
    // while let State::Row = statement.next().unwrap() {
    //     out = statement.read::<String>(0).unwrap();
    // }
    // out

// #[get("/item/<key>/<val>")]
// fn update_item(key: &rocket::http::RawStr, val: &rocket::http::RawStr) -> String {
//     let connection = get_connection();
//     let query = format!("REPLACE INTO items VALUES ('{}', '{}');", key.as_str(), val.as_str());

//     connection
//         .execute(query)
//         .unwrap();
    
//     let result = format!("{}: {}", key.as_str(), val.as_str());
//     result
// }

// fn main() {
//     prepare_database();

//     rocket::ignite().mount("/", routes![index, get_item, update_item]).launch();
// }

// fn get_database() -> String {
//     let key = "LITTLE_LOOKUP_DATABASE";
//     let db_name = match std::env::var(key) {
//         Ok(val) => val,
//         Err(_) => String::from("default.db")
//     };
//     db_name
// }

// fn get_connection() -> sqlite::Connection {
//     let db_name = get_database();
//     let connection = sqlite::open(db_name).unwrap();
//     connection
// }

// fn prepare_database() {
//     let connection = get_connection();
//     connection.execute(
//         "CREATE TABLE IF NOT EXISTS items (
//             key TEXT,
//             val TEXT
//         );
//         CREATE UNIQUE INDEX IF NOT EXISTS idx_items_key ON items (key);"
//     ).unwrap();
// }

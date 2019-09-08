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
use self::models::{Item, NewItem};

pub fn get_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[get("/")]
fn index() -> &'static str {
    "Routes:
  /item/<key>: Get val for <key>
  /item/<key>/<val>: Update <val> for <key>
"
}

#[get("/item/<id>")]
fn get_item(id: &rocket::http::RawStr) -> String {
    use self::schema::items::dsl::*;

    let connection = get_connection();

    let results = items.filter(key.eq(id.as_str()))
        .limit(1)
        .load::<Item>(&connection)
        .expect("Error loading item");

    match results.get(0) {
        Some(x) => String::from(x.val.clone()),
        None    => String::from("Undefined")
    }
}

#[get("/item/<id>/<value>")]
fn update_item(id: &rocket::http::RawStr, value: &rocket::http::RawStr) -> String {
    use self::schema::items;

    let connection = get_connection();

    let new_item = NewItem {
        key: id,
        val: value
    };

    diesel::insert_into(items::table)
        .values(&new_item)
        .execute(&connection)
        .expect("Error creating new item");

    let result = format!("{}: {}", id.as_str(), value.as_str());
    result
}

fn main() {
    // prepare_database();

    rocket::ignite().mount("/", routes![index, get_item, update_item]).launch();
}

// fn get_database() -> String {
//     let key = "LITTLE_LOOKUP_DATABASE";
//     let db_name = match std::env::var(key) {
//         Ok(val) => val,
//         Err(_) => String::from("default.db")
//     };
//     db_name
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

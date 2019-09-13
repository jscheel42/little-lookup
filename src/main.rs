#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod schema;
pub mod models;

use rocket::http::RawStr;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use self::models::{Item, NewItem};

diesel_migrations::embed_migrations!();

pub fn get_connection() -> SqliteConnection {
    let key = "LITTLE_LOOKUP_DATABASE";
    let database_url = match std::env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("default.db")
    };

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[get("/")]
fn index() -> &'static str {
    "Routes:
  /item/<key>: Get val for <key>
  /item/<key>/<val>: Update <val> for <key>
  /list?filter=<x>&delim=<y>: List all keys, optional filter (sql like %<x>%), optional custom delimiter <y> (defaults to space)
"
}

#[get("/item/<id>")]
fn get_item(id: &RawStr) -> String {
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

#[get("/list?<filter>&<delim>")]
fn list_items(filter: Option<&RawStr>, delim: Option<&RawStr>) -> String {
    use self::schema::items::dsl::*;

    let connection = get_connection();

    let results =
        match filter {
            Some(f) => {
                let sql_filter = format!("%{}%", f);
                items.filter(key.like(sql_filter)).load::<Item>(&connection).expect("Error loading items")
            }
            None => items.load::<Item>(&connection).expect("Error loading items")
        };

    let delimiter =
        match delim {
            Some(d) => d.as_str(),
            None => " "
        };

    let result_collection: String = results.iter().fold(String::from(""), |mut acc, result| {
            &acc.push_str(&result.key);
            &acc.push_str(delimiter);
            &acc.push_str(&result.val);
            &acc.push_str("\n");
            acc
        }
    );

    result_collection
}

#[get("/item/<id>/<value>")]
fn update_item(id: &RawStr, value: &RawStr) -> String {
    use self::schema::items;

    let connection = get_connection();

    let new_item = NewItem {
        key: id,
        val: value
    };

    diesel::replace_into(items::table)
        .values(&new_item)
        .execute(&connection)
        .expect("Error creating new item");

    let result = format!("{}", value.as_str());
    result
}

fn prepare_database() {
    let connection = get_connection();
    let _ = embedded_migrations::run_with_output(&connection, &mut std::io::stdout());
}

fn main() {
    prepare_database();
    rocket::ignite().mount("/", routes![index, get_item, update_item, list_items]).launch();
}

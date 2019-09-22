#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod schema;
pub mod models;

use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder};
use std::collections::HashMap;
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

fn index() -> impl Responder {
    let body = "Routes:
  /item/<key>: Get val for <key>
  /item/<key>/<val>: Update <val> for <key>
  /list?filter=<x>&delim=<y>: List all keys, optional filter (sql like %<x>%), optional custom delimiter <y> (defaults to space)";
    HttpResponse::Ok().body(body)
}

fn get_item(id: web::Path<(String)>) -> impl Responder {
    use self::schema::items::dsl::*;

    let connection = get_connection();

    let results = items.filter(key.eq(id.as_str()))
        .limit(1)
        .load::<Item>(&connection)
        .expect("Error loading item");

    let body = match results.get(0) {
        Some(x) => String::from(x.val.clone()),
        None    => String::from("Undefined")
    };

    HttpResponse::Ok().body(body)
}

fn list_items(req: HttpRequest) -> impl Responder {
    use self::schema::items::dsl::*;

    let query_options: String = req.query_string().to_string();
    let query_options_map = req_query_to_map(query_options);

    let connection = get_connection();

    let results =
        match query_options_map.get("filter") {
            Some(f) => {
                let sql_filter = format!("%{}%", f);
                items.filter(key.like(sql_filter)).load::<Item>(&connection).expect("Error loading items")
            }
            None => items.load::<Item>(&connection).expect("Error loading items")
        };

    let delimiter =
        match query_options_map.get("delim") {
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

    HttpResponse::Ok().body(result_collection)
}

fn update_item(info: web::Path<(String, String)>) -> impl Responder {
    use self::schema::items;

    let id = &info.0;
    let value = &info.1;

    let connection = get_connection();

    let new_item = NewItem {
        key: id.as_str(),
        val: value.as_str()
    };

    diesel::replace_into(items::table)
        .values(&new_item)
        .execute(&connection)
        .expect("Error creating new item");

    let body = format!("{}", value.as_str());

    HttpResponse::Ok().body(body)
}

fn req_query_to_map(query_string: String) -> HashMap<String, String> {
    let query_map: HashMap<String, String> = match query_string.as_str() {
        "" => HashMap::new(),
        _ => {
            let query_vec: Vec<&str> = query_string.split('&').collect();

            query_vec.iter().fold(HashMap::new(), |mut acc, query_element| {
                let query_element_vec: Vec<&str> = query_element.split('=').collect();
                let key = query_element_vec.get(0).unwrap();
                let val = query_element_vec.get(1).unwrap();
                acc.insert(String::from(*key), String::from(*val));
                acc
            })
        }
    };

    query_map
}

fn prepare_database() {
    let connection = get_connection();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap()
}

fn main() {
    prepare_database();
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/item/{id}", web::get().to(get_item))
            .route("/item/{id}/{val}", web::get().to(update_item))
            .route("/list", web::get().to(list_items))
    })
    .bind("0.0.0.0:8088")
    .unwrap()
    .run()
    .unwrap();
}

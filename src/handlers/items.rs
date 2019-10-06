use crate::models::item::ItemList;
use crate::db_connection::{ SLPool, SLPooledConnection };
// use crate::schema::items::dsl::*;

// pub mod models;
// pub mod schema;

use crate::models::item::{Item, NewItem};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
// use diesel::prelude::*;
// use diesel::sqlite::SqliteConnection;
use std::collections::HashMap;

diesel_migrations::embed_migrations!();

// Utility functions

fn check_psk(query_options_map: &HashMap<String, String>) -> String {
    let server_psk = get_psk();
    if server_psk != String::from("") {
        let client_psk = match query_options_map.get("psk") {
            Some(psk) => String::from(psk),
            None => String::from("")
        };

        if client_psk != server_psk {
            match client_psk.as_str() {
                "" => return String::from("PSK required"),
                _ => return String::from("Incorrect PSK")
            }
        }
    };
    String::from("")
}

fn get_psk() -> String {
    let key = "LITTLE_LOOKUP_PSK";
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("")
    }
}

fn req_query_to_map(query_string: String) -> HashMap<String, String> {
    let query_map: HashMap<String, String> = match query_string.as_str() {
        "" => HashMap::new(),
        _ => {
            let query_vec: Vec<&str> = query_string.split('&').collect();

            query_vec
                .iter()
                .fold(HashMap::new(), |mut acc, query_element| {
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

fn sl_pool_handler(pool: web::Data<SLPool>) -> Result<SLPooledConnection, HttpResponse> {
    pool
    .get()
    .map_err(|e| {
        HttpResponse::InternalServerError().json(e.to_string())
    })
}

// Route handler functions

fn index() -> impl Responder {
    let body = "Routes:
  /item/<key>: Get val for <key>
  /item/<key>/<val>: Update <val> for <key>
  /list?filter=<x>&delim=<y>: List all keys, optional filter (sql like %<x>%), optional custom delimiter <y> (defaults to space)
  /delete/<key>: Delete <val> for <key>";
    HttpResponse::Ok().body(body)
}

fn delete_item(id: web::Path<(String)>, req: HttpRequest, pool: web::Data<SLPool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let sl_pool = sl_pool_handler(pool)?;

    match Item::destroy(id.as_str(), &sl_pool) {
        Ok(delete_count) => Ok(HttpResponse::Ok().body(format!("{} items deleted", delete_count))),
        Error => Err(HttpResponse::Unauthorized().body("Delete failed"))
    }
}

fn get_item(id: web::Path<(String)>, req: HttpRequest, pool: web::Data<SLPool>) -> impl Responder {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    let sl_pool = sl_pool_handler(pool)?;

    let body = match Item::find(id.as_str(), &sl_pool) {
        Item(item) => item.val.as_str(),
        Error => "Undefined"
    }

    // let results = items
    //     .filter(key.eq(id.as_str()))
    //     .limit(1)
    //     .load::<Item>(&connection)
    //     .expect("Error loading item");

    // let body = match results.get(0) {
    //     Some(x) => String::from(x.val.clone()),
    //     None => String::from("Undefined"),
    // };

    HttpResponse::Ok().body(body)
}

fn list_items(req: HttpRequest) -> impl Responder {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    let connection = get_connection();

    let results = match query_options_map.get("filter") {
        Some(f) => {
            let sql_filter = format!("%{}%", f);
            items
                .filter(key.like(sql_filter))
                .load::<Item>(&connection)
                .expect("Error loading items")
        }
        None => items
            .load::<Item>(&connection)
            .expect("Error loading items"),
    };

    let delimiter = match query_options_map.get("delim") {
        Some(d) => d.as_str(),
        None => " ",
    };

    let result_collection: String = results.iter().fold(String::from(""), |mut acc, result| {
        &acc.push_str(&result.key);
        &acc.push_str(delimiter);
        &acc.push_str(&result.val);
        &acc.push_str("\n");
        acc
    });

    HttpResponse::Ok().body(result_collection)
}

fn update_item(info: web::Path<(String, String)>, req: HttpRequest) -> impl Responder {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    use self::schema::items;

    let id = &info.0;
    let value = &info.1;

    let connection = get_connection();

    let new_item = NewItem {
        key: id.as_str(),
        val: value.as_str(),
    };

    diesel::replace_into(items::table)
        .values(&new_item)
        .execute(&connection)
        .expect("Error creating new item");

    let body = format!("{}", value.as_str());

    HttpResponse::Ok().body(body)
}
use crate::db_connection::{ Pool, PooledConnection };
use crate::models::item::{Item, ItemList};

use actix_web::{web, HttpRequest, HttpResponse};
use std::collections::HashMap;

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

fn sl_pool_handler(pool: web::Data<Pool>) -> Result<PooledConnection, HttpResponse> {
    pool
    .get()
    .map_err(|e| {
        HttpResponse::InternalServerError().body(e.to_string())
    })
}

// Route handler functions

pub fn index() -> Result<HttpResponse, HttpResponse> {
    let body = "Routes:
  /item/<key>: Get val for <key>
  /item/<key>/<val>: Update <val> for <key>
  /list?filter=<x>&delim=<y>: List all keys, optional filter (sql like %<x>%), optional custom delimiter <y> (defaults to space)
  /delete/<key>: Delete <val> for <key>";
    Ok(HttpResponse::Ok().body(body))
}

pub fn delete_item(id: web::Path<(String)>, req: HttpRequest, pool: web::Data<Pool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let sl_pool = sl_pool_handler(pool)?;

    let delete_count = Item::destroy(id.as_str(), &sl_pool).unwrap();
    Ok(HttpResponse::Ok().body(format!("{} items deleted", delete_count)))
}

pub fn get_item(id: web::Path<(String)>, req: HttpRequest, pool: web::Data<Pool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let sl_pool = sl_pool_handler(pool)?;

    match Item::find(id.as_str(), &sl_pool) {
        Ok(item) => Ok(HttpResponse::Ok().body(item.val)),
        _ => Err(HttpResponse::NotFound().body("Undefined"))
    }
}

pub fn list_items(req: HttpRequest, pool: web::Data<Pool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let sl_pool = sl_pool_handler(pool)?;

    let default_filter = String::from("");
    let filter: String = query_options_map
            .get("filter")
            .unwrap_or_else(|| {&default_filter})
            .to_string();

    let results = ItemList::list(&sl_pool, filter).unwrap();

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

    Ok(HttpResponse::Ok().body(result_collection))
}

pub fn update_item(info: web::Path<(String, String)>, req: HttpRequest, pool: web::Data<Pool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let id = &info.0;
    let value = &info.1;

    let sl_pool = sl_pool_handler(pool)?;

    Item::replace_into(id.as_str(), value.as_str(), &sl_pool).unwrap();

    Ok(HttpResponse::Ok().body(
        String::from(&info.1)
    ))
}
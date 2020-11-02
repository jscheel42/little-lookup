use actix_web::{web, HttpRequest, HttpResponse};
use std::collections::HashMap;

use crate::db_connection::{Pool, PooledConnection};
use crate::models::item::{Item, ItemList};
use crate::util::{get_psk, PSKType};

// Utility functions

fn check_psk(query_options_map: &HashMap<String, String>, psk_type: PSKType) -> String {
    let server_psk = get_psk(psk_type);
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

fn sql_pool_handler(pool: web::Data<Pool>) -> Result<PooledConnection, HttpResponse> {
    pool
    .get()
    .map_err(|e| {
        HttpResponse::InternalServerError().body(e.to_string())
    })
}

// Route handler functions

pub async fn index() -> Result<HttpResponse, HttpResponse> {
    let body = "Routes:
  /get/<key>: Get val for <key>
  /update/<key>/<val>: Update <val> for <key>
  /list?filter=<x>&delim=<y>: List all keys, optional filter (sql like %<x>%), optional custom delimiter <y> (defaults to space)
  /delete/<key>: Delete <val> for <key>";
    Ok(HttpResponse::Ok().body(body))
}

pub async fn delete_item(id: web::Path<String>, req: HttpRequest, pool: web::Data<Pool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map, PSKType::WRITE);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let sql_pool = sql_pool_handler(pool)?;

    let delete_count = Item::destroy(id.as_str(), &sql_pool).unwrap();
    Ok(HttpResponse::Ok().body(format!("{} items deleted", delete_count)))
}

pub async fn get_item(id: web::Path<String>, req: HttpRequest, pool: web::Data<Pool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let sql_pool = sql_pool_handler(pool)?;

    match Item::find(id.as_str(), &sql_pool) {
        Ok(item) => Ok(HttpResponse::Ok().body(item.val)),
        _ => Err(HttpResponse::NotFound().body("Undefined"))
    }
}

pub async fn list_items(req: HttpRequest, pool: web::Data<Pool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let sql_pool = sql_pool_handler(pool)?;

    let default_filter = String::from("");
    let filter: String = query_options_map
            .get("filter")
            .unwrap_or_else(|| {&default_filter})
            .to_string();

    let results = ItemList::list(&sql_pool, filter).unwrap();

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

pub async fn update_item(web::Path((id, val)): web::Path<(String, String)>, req: HttpRequest, pool: web::Data<Pool>) -> Result<HttpResponse, HttpResponse> {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let psk_result = check_psk(&query_options_map, PSKType::WRITE);
    if psk_result.len() > 0 {
        return Err(HttpResponse::Unauthorized().body(psk_result))
    };

    let sql_pool = sql_pool_handler(pool)?;
    Item::replace_into(id.as_str(), val.as_str(), &sql_pool).unwrap();

    Ok(HttpResponse::Ok().body(
        val
    ))
}
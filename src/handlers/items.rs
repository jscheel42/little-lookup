use actix_web::{web, HttpRequest, HttpResponse};
use diesel::r2d2::PoolError;
use log::error;
use std::collections::HashMap;

use crate::db_connection::{Pool, PooledConnection};
use crate::models::item::{Item, ItemList};
use crate::util::{get_namespace, get_psk, PSKType};

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

fn sql_pool_handler(pool: web::Data<Pool>) -> Result<PooledConnection, PoolError> {
    match pool.get() {
        Ok(sql_pooled_connection) => Ok(sql_pooled_connection),
        Err(e) => {
            error!("{}", e);
            Err(e)   
        }
    }
}

// Route handler functions

pub async fn index() -> HttpResponse {
    let body = "
<p>Routes:</p>
<ul>
<li>/get/$KEY: Get val for $KEY</li>
<li>/history/$KEY: Get history for $KEY</li>
<li>/update/$KEY/$VAL: Update $VAL for $KEY</li>
<li>/list?filter=$FOO&delim=$BAR: List all keys, optional filter (sql like %$FOO%), optional custom delimiter $BAR (defaults to space)</li>
<li>/script?filter=$FOO: Get bash script to export all keys, optional filter (sql like %$FOO%)</li>
<li>/delete/$KEY: Delete $VAL for $KEY</li>
</ul>";
    HttpResponse::Ok().body(body)
}

pub async fn delete_item(id: web::Path<String>, req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::WRITE);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    let mut sql_pooled_connection =
        match sql_pool_handler(pool) {
            Ok(sql_pooled_connection) => sql_pooled_connection,
            Err(_) => return HttpResponse::Unauthorized().body("SQL Error")
        };

    let delete_count = Item::destroy(id.as_str(), namespace, &mut sql_pooled_connection).unwrap();
    HttpResponse::Ok().body(format!("{} items deleted", delete_count))
}

pub async fn get_item(id: web::Path<String>, req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    let mut sql_pooled_connection =
        match sql_pool_handler(pool) {
            Ok(sql_pooled_connection) => sql_pooled_connection,
            Err(_) => return HttpResponse::Unauthorized().body("SQL Error")
        };

    match Item::find(id.as_str(), namespace, &mut sql_pooled_connection) {
        Ok(item) => HttpResponse::Ok().body(item.val),
        _ => HttpResponse::NotFound().body("Undefined")
    }
}

pub async fn history_item(id: web::Path<String>, req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    let mut sql_pooled_connection =
        match sql_pool_handler(pool) {
            Ok(sql_pooled_connection) => sql_pooled_connection,
            Err(_) => return HttpResponse::Unauthorized().body("SQL Error")
        };

    match Item::history(id.as_str(), namespace, &mut sql_pooled_connection) {
        Ok(item_list) => {
            let mut val_list: Vec<String> = Vec::new();
            for item in item_list.iter() {
                val_list.push(item.val.clone())
            }
            HttpResponse::Ok().body(
                val_list.join("\n")
            )
        },
        _ => HttpResponse::NotFound().body("Undefined")
    }
}

pub async fn list_items(req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    let mut sql_pooled_connection =
        match sql_pool_handler(pool) {
            Ok(sql_pooled_connection) => sql_pooled_connection,
            Err(_) => return HttpResponse::Unauthorized().body("SQL Error")
        };

    let default_filter = String::from("");
    let filter: String = query_options_map
            .get("filter")
            .unwrap_or_else(|| {&default_filter})
            .to_string();

    let results = ItemList::list(&mut sql_pooled_connection, filter, namespace).unwrap();

    let delimiter: &str = match query_options_map.get("delim") {
        Some(d) => d.as_str(),
        None => " ",
    };

    let result_collection: String = results.iter().fold(String::from(""), |mut acc, result| {
        let _ = &acc.push_str(&result.key);
        let _ = &acc.push_str(delimiter);
        let _ = &acc.push_str(&result.val);
        let _ = &acc.push_str("\n");
        acc
    });

    HttpResponse::Ok().body(result_collection)
}

pub async fn script(req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    let mut sql_pooled_connection =
        match sql_pool_handler(pool) {
            Ok(sql_pooled_connection) => sql_pooled_connection,
            Err(_) => return HttpResponse::Unauthorized().body("SQL Error")
        };

    let default_filter = String::from("");
    let filter: String = query_options_map
            .get("filter")
            .unwrap_or_else(|| {&default_filter})
            .to_string();

    let results = ItemList::list(&mut sql_pooled_connection, filter, namespace).unwrap();

    let result_collection: String = results.iter().fold(String::from("#!/bin/bash\n"),
    |mut acc, result| {
        let _ = &acc.push_str("export ");
        let _ = &acc.push_str(&result.key);
        let _ = &acc.push_str("='");
        let _ = &acc.push_str(&result.val);
        let _ = &acc.push_str("'");
        let _ = &acc.push_str("\n");
        acc
    });

    HttpResponse::Ok().body(result_collection)
}

pub async fn update_item(params: web::Path<(String, String)>, req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    let (id, val) = params.into_inner();
    let query_options_map = req_query_to_map(
        req.query_string().to_string()
    );
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::WRITE);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result)
    };

    let mut sql_pooled_connection =
        match sql_pool_handler(pool) {
            Ok(sql_pooled_connection) => sql_pooled_connection,
            Err(_) => return HttpResponse::Unauthorized().body("SQL Error")
        };

    Item::replace_into(id.as_str(), val.as_str(), namespace, &mut sql_pooled_connection).unwrap();

    HttpResponse::Ok().body(
        val
    )
}


#[cfg(test)]
mod tests {
    use actix_web::{http, test, App, web::{self,Data}};

    use crate::db_connection::establish_connection;

    use super::*;
    #[actix_rt::test]
    async fn test_script() {
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(establish_connection().unwrap()))
                .route("/script", web::get().to(script)),
        )
        .await;

        let req = test::TestRequest::get().uri("/script?filter=test").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: web::Bytes = test::read_body(resp).await;
        assert_eq!(body, "#!/bin/bash\n");
    }

    #[actix_rt::test]
    async fn test_update_item() {
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(establish_connection().unwrap()))
                .route("/update/{id}/{val}", web::put().to(update_item)),
        )
        .await;

        let req = test::TestRequest::put()
            .uri("/update/item1/value1?filter=test")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        assert_eq!(body, "value1");
    }
}

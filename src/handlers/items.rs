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
            None => String::from(""),
        };

        if client_psk != server_psk {
            match client_psk.as_str() {
                "" => return String::from("PSK required"),
                _ => return String::from("Incorrect PSK"),
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
                    // Only process valid query parameters with key and value
                    if let (Some(key), Some(val)) =
                        (query_element_vec.get(0), query_element_vec.get(1))
                    {
                        acc.insert(String::from(*key), String::from(*val));
                    }
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
<p>Testing CD</p>
<p>Routes:</p>
<ul>
<li>/get/$KEY : Get val for $KEY</li>
<li>/history/$KEY : Get history for $KEY</li>
<li>/update/$KEY/$VAL : Update $VAL for $KEY</li>
<li>/list?delim=$FOO : List all keys, optional custom delimiter $BAR (defaults to space)</li>
<li>/script : Get bash script to export all keys</li>
<li>/delete/$KEY : Delete $VAL for $KEY</li>
</ul>";
    HttpResponse::Ok().body(body)
}

pub async fn delete_item(
    id: web::Path<String>,
    req: HttpRequest,
    pool: web::Data<Pool>,
) -> HttpResponse {
    let query_options_map = req_query_to_map(req.query_string().to_string());
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::WRITE);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result);
    };

    let mut sql_pooled_connection = match sql_pool_handler(pool) {
        Ok(sql_pooled_connection) => sql_pooled_connection,
        Err(_) => return HttpResponse::InternalServerError().body("Database connection failed"),
    };

    match Item::destroy(id.as_str(), namespace, &mut sql_pooled_connection) {
        Ok(delete_count) => HttpResponse::Ok().body(format!("{} items deleted", delete_count)),
        Err(e) => {
            error!("Failed to delete item '{}': {}", id, e);
            HttpResponse::InternalServerError().body("Failed to delete item")
        }
    }
}

pub async fn get_item(
    id: web::Path<String>,
    req: HttpRequest,
    pool: web::Data<Pool>,
) -> HttpResponse {
    let query_options_map = req_query_to_map(req.query_string().to_string());
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result);
    };

    let mut sql_pooled_connection = match sql_pool_handler(pool) {
        Ok(sql_pooled_connection) => sql_pooled_connection,
        Err(_) => return HttpResponse::Unauthorized().body("SQL Error"),
    };

    match Item::find(id.as_str(), namespace, &mut sql_pooled_connection) {
        Ok(item) => HttpResponse::Ok().body(item.val),
        _ => HttpResponse::NotFound().body("Undefined"),
    }
}

pub async fn history_item(
    id: web::Path<String>,
    req: HttpRequest,
    pool: web::Data<Pool>,
) -> HttpResponse {
    let query_options_map = req_query_to_map(req.query_string().to_string());
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result);
    };

    let mut sql_pooled_connection = match sql_pool_handler(pool) {
        Ok(sql_pooled_connection) => sql_pooled_connection,
        Err(_) => return HttpResponse::Unauthorized().body("SQL Error"),
    };

    match Item::history(id.as_str(), namespace, &mut sql_pooled_connection) {
        Ok(item_list) => {
            let mut val_list: Vec<String> = Vec::new();
            for item in item_list.iter() {
                val_list.push(item.val.clone())
            }

            let body_string: String = format!("<pre>\n{}</pre>", val_list.join("\n"));

            HttpResponse::Ok().body(body_string)
        }
        _ => HttpResponse::NotFound().body("Undefined"),
    }
}

pub async fn list_items(req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    let query_options_map = req_query_to_map(req.query_string().to_string());
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result);
    };

    let mut sql_pooled_connection = match sql_pool_handler(pool) {
        Ok(sql_pooled_connection) => sql_pooled_connection,
        Err(_) => return HttpResponse::InternalServerError().body("Database connection failed"),
    };

    let results = match ItemList::list(&mut sql_pooled_connection, namespace) {
        Ok(items) => items,
        Err(e) => {
            error!("Failed to list items: {}", e);
            return HttpResponse::InternalServerError().body("Failed to retrieve items");
        }
    };

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

    let body_string: String = format!("<pre>\n{}</pre>", result_collection);

    HttpResponse::Ok().body(body_string)
}

pub async fn script(req: HttpRequest, pool: web::Data<Pool>) -> HttpResponse {
    let query_options_map = req_query_to_map(req.query_string().to_string());
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::READ);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result);
    };

    let mut sql_pooled_connection = match sql_pool_handler(pool) {
        Ok(sql_pooled_connection) => sql_pooled_connection,
        Err(_) => return HttpResponse::InternalServerError().body("Database connection failed"),
    };

    let results = match ItemList::list(&mut sql_pooled_connection, namespace) {
        Ok(items) => items,
        Err(e) => {
            error!("Failed to generate script: {}", e);
            return HttpResponse::InternalServerError().body("Failed to generate script");
        }
    };

    let result_collection: String =
        results
            .iter()
            .fold(String::from("#!/bin/bash\n"), |mut acc, result| {
                let _ = &acc.push_str("export ");
                let _ = &acc.push_str(&result.key);
                let _ = &acc.push_str("='");
                let _ = &acc.push_str(&result.val);
                let _ = &acc.push_str("'");
                let _ = &acc.push_str("\n");
                acc
            });

    let body_string: String = format!("<pre>\n{}</pre>", result_collection);

    HttpResponse::Ok().body(body_string)
}

pub async fn update_item(
    params: web::Path<(String, String)>,
    req: HttpRequest,
    pool: web::Data<Pool>,
) -> HttpResponse {
    let (id, val) = params.into_inner();
    let query_options_map = req_query_to_map(req.query_string().to_string());
    let namespace: &str = get_namespace(&query_options_map);
    let psk_result = check_psk(&query_options_map, PSKType::WRITE);
    if psk_result.len() > 0 {
        return HttpResponse::Unauthorized().body(psk_result);
    };

    let mut sql_pooled_connection = match sql_pool_handler(pool) {
        Ok(sql_pooled_connection) => sql_pooled_connection,
        Err(_) => return HttpResponse::InternalServerError().body("Database connection failed"),
    };

    match Item::replace_into(
        id.as_str(),
        val.as_str(),
        namespace,
        &mut sql_pooled_connection,
    ) {
        Ok(_) => HttpResponse::Ok().body(val),
        Err(e) => {
            error!("Failed to update item '{}': {}", id, e);
            HttpResponse::InternalServerError().body("Failed to update item")
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{
        http, test,
        web::{self, Data},
        App,
    };
    use serial_test::serial;

    use crate::db_connection::establish_connection;

    use super::*;

    #[actix_rt::test]
    #[serial]
    async fn test_index() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/", web::get().to(index)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: web::Bytes = test::read_body(resp).await;
        assert!(body
            .to_vec()
            .windows(b"Testing CD".len())
            .any(|w| w == b"Testing CD"));
    }

    #[actix_rt::test]
    #[serial]
    async fn test_script() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/script", web::get().to(script)),
        )
        .await;

        let req = test::TestRequest::get().uri("/script?ns=test").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: web::Bytes = test::read_body(resp).await;
        assert_eq!(body, "<pre>\n#!/bin/bash\n</pre>");
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_update_item() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::put().to(update_item)),
        )
        .await;

        let req = test::TestRequest::put()
            .uri("/update/testkey/testvalue")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        assert_eq!(body, "testvalue");
    }

    #[actix_rt::test]
    #[serial]
    async fn test_get_item() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item))
                .route("/get/{id}", web::get().to(get_item)),
        )
        .await;

        let update_req = test::TestRequest::get()
            .uri("/update/test_get_key/test_get_value?ns=test_get_ns")
            .to_request();
        let _ = test::call_service(&mut app, update_req).await;

        let req = test::TestRequest::get()
            .uri("/get/test_get_key?ns=test_get_ns")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        assert_eq!(body, "test_get_value");
    }

    #[actix_rt::test]
    #[serial]
    async fn test_get_item_not_found() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/get/{id}", web::get().to(get_item)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/get/nonexistent_key_xyz?ns=nonexistent_ns")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);

        let body = test::read_body(resp).await;
        assert_eq!(body, "Undefined");
    }

    #[actix_rt::test]
    #[serial]
    async fn test_history_item() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item))
                .route("/history/{id}", web::get().to(history_item)),
        )
        .await;

        let ns = "test_history_ns";
        let key = "test_history_key";

        let update1 = test::TestRequest::get()
            .uri(&format!("/update/{}/value1?ns={}", key, ns))
            .to_request();
        let _ = test::call_service(&mut app, update1).await;

        let update2 = test::TestRequest::get()
            .uri(&format!("/update/{}/value2?ns={}", key, ns))
            .to_request();
        let _ = test::call_service(&mut app, update2).await;

        let req = test::TestRequest::get()
            .uri(&format!("/history/{}?ns={}", key, ns))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("value1"));
        assert!(body_str.contains("value2"));
    }

    #[actix_rt::test]
    #[serial]
    async fn test_list_items_with_delimiter() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item))
                .route("/list", web::get().to(list_items)),
        )
        .await;

        let ns = "test_list_delim_ns";

        let update1 = test::TestRequest::get()
            .uri(&format!("/update/key1/val1?ns={}", ns))
            .to_request();
        let _ = test::call_service(&mut app, update1).await;

        let update2 = test::TestRequest::get()
            .uri(&format!("/update/key2/val2?ns={}", ns))
            .to_request();
        let _ = test::call_service(&mut app, update2).await;

        let req = test::TestRequest::get()
            .uri(&format!("/list?ns={}&delim=|", ns))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("key1|val1"));
        assert!(body_str.contains("key2|val2"));
    }

    #[actix_rt::test]
    #[serial]
    async fn test_delete_item() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item))
                .route("/delete/{id}", web::get().to(delete_item))
                .route("/get/{id}", web::get().to(get_item)),
        )
        .await;

        let ns = "test_delete_ns";
        let key = "test_delete_key";

        let update_req = test::TestRequest::get()
            .uri(&format!("/update/{}/test_value?ns={}", key, ns))
            .to_request();
        let _ = test::call_service(&mut app, update_req).await;

        let delete_req = test::TestRequest::get()
            .uri(&format!("/delete/{}?ns={}", key, ns))
            .to_request();
        let delete_resp = test::call_service(&mut app, delete_req).await;
        assert_eq!(delete_resp.status(), http::StatusCode::OK);

        let get_req = test::TestRequest::get()
            .uri(&format!("/get/{}?ns={}", key, ns))
            .to_request();
        let get_resp = test::call_service(&mut app, get_req).await;
        assert_eq!(get_resp.status(), http::StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    #[serial]
    async fn test_namespace_isolation_handlers() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item))
                .route("/get/{id}", web::get().to(get_item)),
        )
        .await;

        let key = "shared_key_handler";

        let update_ns1 = test::TestRequest::get()
            .uri(&format!("/update/{}/value_ns1?ns=ns1", key))
            .to_request();
        let _ = test::call_service(&mut app, update_ns1).await;

        let update_ns2 = test::TestRequest::get()
            .uri(&format!("/update/{}/value_ns2?ns=ns2", key))
            .to_request();
        let _ = test::call_service(&mut app, update_ns2).await;

        let get_ns1 = test::TestRequest::get()
            .uri(&format!("/get/{}?ns=ns1", key))
            .to_request();
        let resp_ns1 = test::call_service(&mut app, get_ns1).await;
        let body_ns1 = test::read_body(resp_ns1).await;
        assert_eq!(body_ns1, "value_ns1");

        let get_ns2 = test::TestRequest::get()
            .uri(&format!("/get/{}?ns=ns2", key))
            .to_request();
        let resp_ns2 = test::call_service(&mut app, get_ns2).await;
        let body_ns2 = test::read_body(resp_ns2).await;
        assert_eq!(body_ns2, "value_ns2");
    }

    #[actix_rt::test]
    #[serial]
    async fn test_req_query_to_map_basic() {
        let query = String::from("key1=value1&key2=value2");
        let map = req_query_to_map(query);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("key1"), Some(&String::from("value1")));
        assert_eq!(map.get("key2"), Some(&String::from("value2")));
    }

    #[actix_rt::test]
    #[serial]
    async fn test_req_query_to_map_empty() {
        let query = String::from("");
        let map = req_query_to_map(query);
        assert_eq!(map.len(), 0);
    }

    #[actix_rt::test]
    #[serial]
    async fn test_req_query_to_map_malformed() {
        let query = String::from("key1=value1&malformed&key2=value2");
        let map = req_query_to_map(query);

        assert_eq!(map.get("key1"), Some(&String::from("value1")));
        assert_eq!(map.get("key2"), Some(&String::from("value2")));
        assert!(!map.contains_key("malformed"));
    }

    #[actix_rt::test]
    #[serial]
    async fn test_req_query_to_map_special_chars() {
        let query = String::from("ns=test&delim=|");
        let map = req_query_to_map(query);

        assert_eq!(map.get("ns"), Some(&String::from("test")));
        assert_eq!(map.get("delim"), Some(&String::from("|")));
    }

    #[actix_rt::test]
    #[serial]
    async fn test_req_query_to_map_duplicate_keys() {
        let query = String::from("key=value1&key=value2");
        let map = req_query_to_map(query);

        assert_eq!(map.len(), 1);
        assert!(map.get("key").is_some());
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_check_psk_read() {
        use crate::util::PSKType;
        use std::collections::HashMap;

        std::env::set_var("LITTLE_LOOKUP_PSK_READ", "test_read_psk");

        let mut map = HashMap::new();
        map.insert(String::from("psk"), String::from("test_read_psk"));

        let result = check_psk(&map, PSKType::READ);
        assert_eq!(result, "");

        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_check_psk_write() {
        use crate::util::PSKType;
        use std::collections::HashMap;

        std::env::set_var("LITTLE_LOOKUP_PSK_WRITE", "test_write_psk");

        let mut map = HashMap::new();
        map.insert(String::from("psk"), String::from("test_write_psk"));

        let result = check_psk(&map, PSKType::WRITE);
        assert_eq!(result, "");

        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_check_psk_missing() {
        use crate::util::PSKType;
        use std::collections::HashMap;

        std::env::set_var("LITTLE_LOOKUP_PSK_READ", "test_psk");

        let map = HashMap::new();
        let result = check_psk(&map, PSKType::READ);
        assert_eq!(result, "PSK required");

        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_check_psk_incorrect() {
        use crate::util::PSKType;
        use std::collections::HashMap;

        std::env::set_var("LITTLE_LOOKUP_PSK_READ", "correct_psk");

        let mut map = HashMap::new();
        map.insert(String::from("psk"), String::from("wrong_psk"));

        let result = check_psk(&map, PSKType::READ);
        assert_eq!(result, "Incorrect PSK");

        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_check_psk_not_required() {
        use crate::util::PSKType;
        use std::collections::HashMap;

        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let map = HashMap::new();
        let result = check_psk(&map, PSKType::READ);
        assert_eq!(result, "");
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_update_with_read_psk_should_fail() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");
        std::env::set_var("LITTLE_LOOKUP_PSK_WRITE", "write_secret");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/update/key/value?ns=psk_test_ns_1")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let body = test::read_body(resp).await;
        assert_eq!(body, "PSK required");

        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_update_with_wrong_psk() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");
        std::env::set_var("LITTLE_LOOKUP_PSK_WRITE", "correct_write_psk");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/update/key/value?psk=wrong_psk&ns=psk_test_ns_2")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let body = test::read_body(resp).await;
        assert_eq!(body, "Incorrect PSK");

        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
    }

    #[actix_rt::test]
    #[serial]
    #[serial]
    async fn test_get_with_read_psk() {
        std::env::set_var("LITTLE_LOOKUP_PSK_READ", "read_secret");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/get/{id}", web::get().to(get_item)),
        )
        .await;

        let req = test::TestRequest::get().uri("/get/somekey").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let body = test::read_body(resp).await;
        assert_eq!(body, "PSK required");

        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");
    }

    #[actix_rt::test]
    #[serial]
    async fn test_sql_injection_prevention() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item))
                .route("/get/{id}", web::get().to(get_item)),
        )
        .await;

        let ns = "sql_injection_test";
        let malicious_key = "test'; DROP TABLE items; --";
        let safe_value = "safe_value";

        let update_req = test::TestRequest::get()
            .uri(&format!(
                "/update/{}/{}?ns={}",
                urlencoding::encode(malicious_key),
                safe_value,
                ns
            ))
            .to_request();
        let update_resp = test::call_service(&mut app, update_req).await;
        assert_eq!(update_resp.status(), http::StatusCode::OK);

        let get_req = test::TestRequest::get()
            .uri(&format!(
                "/get/{}?ns={}",
                urlencoding::encode(malicious_key),
                ns
            ))
            .to_request();
        let get_resp = test::call_service(&mut app, get_req).await;
        assert_eq!(get_resp.status(), http::StatusCode::OK);

        let body = test::read_body(get_resp).await;
        assert_eq!(body, safe_value);
    }

    #[actix_rt::test]
    #[serial]
    async fn test_unicode_keys_and_values() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item))
                .route("/get/{id}", web::get().to(get_item)),
        )
        .await;

        let ns = "unicode_test";
        let unicode_key = "emoji_key_🚀";
        let unicode_value = "Hello_世界_🌍";

        let update_req = test::TestRequest::get()
            .uri(&format!(
                "/update/{}/{}?ns={}",
                urlencoding::encode(unicode_key),
                urlencoding::encode(unicode_value),
                ns
            ))
            .to_request();
        let update_resp = test::call_service(&mut app, update_req).await;
        assert_eq!(update_resp.status(), http::StatusCode::OK);

        let get_req = test::TestRequest::get()
            .uri(&format!(
                "/get/{}?ns={}",
                urlencoding::encode(unicode_key),
                ns
            ))
            .to_request();
        let get_resp = test::call_service(&mut app, get_req).await;
        assert_eq!(get_resp.status(), http::StatusCode::OK);

        let body = test::read_body(get_resp).await;
        assert_eq!(String::from_utf8(body.to_vec()).unwrap(), unicode_value);
    }

    #[actix_rt::test]
    #[serial]
    async fn test_full_crud_workflow() {
        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");

        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .route("/update/{id}/{val}", web::get().to(update_item))
                .route("/get/{id}", web::get().to(get_item))
                .route("/history/{id}", web::get().to(history_item))
                .route("/delete/{id}", web::get().to(delete_item)),
        )
        .await;

        let ns = "crud_workflow_test";
        let key = "workflow_key";

        let update1 = test::TestRequest::get()
            .uri(&format!("/update/{}/value1?ns={}", key, ns))
            .to_request();
        let resp1 = test::call_service(&mut app, update1).await;
        assert_eq!(resp1.status(), http::StatusCode::OK);

        let get1 = test::TestRequest::get()
            .uri(&format!("/get/{}?ns={}", key, ns))
            .to_request();
        let resp_get1 = test::call_service(&mut app, get1).await;
        assert_eq!(resp_get1.status(), http::StatusCode::OK);
        let body1 = test::read_body(resp_get1).await;
        assert_eq!(body1, "value1");

        let update2 = test::TestRequest::get()
            .uri(&format!("/update/{}/value2?ns={}", key, ns))
            .to_request();
        let resp2 = test::call_service(&mut app, update2).await;
        assert_eq!(resp2.status(), http::StatusCode::OK);

        let get2 = test::TestRequest::get()
            .uri(&format!("/get/{}?ns={}", key, ns))
            .to_request();
        let resp_get2 = test::call_service(&mut app, get2).await;
        assert_eq!(resp_get2.status(), http::StatusCode::OK);
        let body2 = test::read_body(resp_get2).await;
        assert_eq!(body2, "value2");

        let history = test::TestRequest::get()
            .uri(&format!("/history/{}?ns={}", key, ns))
            .to_request();
        let resp_history = test::call_service(&mut app, history).await;
        assert_eq!(resp_history.status(), http::StatusCode::OK);
        let body_history = test::read_body(resp_history).await;
        let history_str = String::from_utf8(body_history.to_vec()).unwrap();
        assert!(history_str.contains("value1"));
        assert!(history_str.contains("value2"));

        let delete = test::TestRequest::get()
            .uri(&format!("/delete/{}?ns={}", key, ns))
            .to_request();
        let resp_delete = test::call_service(&mut app, delete).await;
        assert_eq!(resp_delete.status(), http::StatusCode::OK);

        let get_deleted = test::TestRequest::get()
            .uri(&format!("/get/{}?ns={}", key, ns))
            .to_request();
        let resp_deleted = test::call_service(&mut app, get_deleted).await;
        assert_eq!(resp_deleted.status(), http::StatusCode::NOT_FOUND);
    }
}

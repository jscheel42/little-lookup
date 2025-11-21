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
                    // Only process valid query parameters with key and value
                    if let (Some(key), Some(val)) = (query_element_vec.get(0), query_element_vec.get(1)) {
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
            Err(_) => return HttpResponse::InternalServerError().body("Database connection failed")
        };

    match Item::destroy(id.as_str(), namespace, &mut sql_pooled_connection) {
        Ok(delete_count) => HttpResponse::Ok().body(format!("{} items deleted", delete_count)),
        Err(e) => {
            error!("Failed to delete item '{}': {}", id, e);
            HttpResponse::InternalServerError().body("Failed to delete item")
        }
    }
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

            let body_string: String = format!("<pre>\n{}</pre>", val_list.join("\n"));

            HttpResponse::Ok().body(body_string)
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
            Err(_) => return HttpResponse::InternalServerError().body("Database connection failed")
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
            Err(_) => return HttpResponse::InternalServerError().body("Database connection failed")
        };

    let results = match ItemList::list(&mut sql_pooled_connection, namespace) {
        Ok(items) => items,
        Err(e) => {
            error!("Failed to generate script: {}", e);
            return HttpResponse::InternalServerError().body("Failed to generate script");
        }
    };

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

    let body_string: String = format!("<pre>\n{}</pre>", result_collection);

    HttpResponse::Ok().body(body_string)
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
            Err(_) => return HttpResponse::InternalServerError().body("Database connection failed")
        };

    match Item::replace_into(id.as_str(), val.as_str(), namespace, &mut sql_pooled_connection) {
        Ok(_) => HttpResponse::Ok().body(val),
        Err(e) => {
            error!("Failed to update item '{}': {}", id, e);
            HttpResponse::InternalServerError().body("Failed to update item")
        }
    }
}


#[cfg(test)]
mod tests {
    use actix_web::{http, test, App, web::{self,Data}};

    use crate::db_connection::establish_connection;

    use super::*;
    // Helper function to create a test app with all routes
    fn create_test_app(pool: web::Data<Pool>) -> test::TestServiceFactory<impl actix_web::dev::ServiceFactory<actix_web::dev::ServiceRequest, Config = (), Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>, Error = actix_web::Error, InitError = (), Transform = (), InitError = ()>> {
        test::init_service(
            App::new()
                .app_data(pool)
                .route("/", web::get().to(index))
                .route("/get/{id}", web::get().to(get_item))
                .route("/history/{id}", web::get().to(history_item))
                .route("/update/{id}/{val}", web::put().to(update_item))
                .route("/list", web::get().to(list_items))
                .route("/delete/{id}", web::get().to(delete_item))
                .route("/script", web::get().to(script)),
        )
    }

    #[actix_rt::test]
    async fn test_index() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: web::Bytes = test::read_body(resp).await;
        assert!(body.to_vec().windows(b"Testing CD".len()).any(|w| w == b"Testing CD"));
    }

    #[actix_rt::test]
    async fn test_script() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        let req = test::TestRequest::get().uri("/script?ns=test").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: web::Bytes = test::read_body(resp).await;
        assert_eq!(body, "<pre>\n#!/bin/bash\n</pre>");
    }

    #[actix_rt::test]
    async fn test_update_item() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        let req = test::TestRequest::put()
            .uri("/update/testkey/testvalue")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        assert_eq!(body, "testvalue");
    }

    #[actix_rt::test]
    async fn test_get_item_not_found() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        let req = test::TestRequest::get()
            .uri("/get/nonexistent_key_12345")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn test_get_item_found() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // First update an item
        let update_req = test::TestRequest::put()
            .uri("/update/testkey123/testvalue123")
            .to_request();
        let resp = test::call_service(&mut app, update_req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        // Then retrieve it
        let get_req = test::TestRequest::get()
            .uri("/get/testkey123")
            .to_request();
        let resp = test::call_service(&mut app, get_req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        assert_eq!(body, "testvalue123");
    }

    #[actix_rt::test]
    async fn test_list_items() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        let req = test::TestRequest::get().uri("/list").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: web::Bytes = test::read_body(resp).await;
        assert!(body.starts_with(b"<pre>"));
        assert!(body.ends_with(b"</pre>"));
    }

    #[actix_rt::test]
    async fn test_list_items_with_custom_delimiter() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        let req = test::TestRequest::get()
            .uri("/list?delim=:")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_history_item() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Update an item
        let update_req = test::TestRequest::put()
            .uri("/update/historykey/value1")
            .to_request();
        let _ = test::call_service(&mut app, update_req).await;

        // Get history
        let history_req = test::TestRequest::get()
            .uri("/history/historykey")
            .to_request();
        let resp = test::call_service(&mut app, history_req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: web::Bytes = test::read_body(resp).await;
        assert!(body.starts_with(b"<pre>"));
    }

    #[actix_rt::test]
    async fn test_delete_item() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // First update an item
        let update_req = test::TestRequest::put()
            .uri("/update/deletekey/deleteme")
            .to_request();
        let _ = test::call_service(&mut app, update_req).await;

        // Delete it
        let delete_req = test::TestRequest::get()
            .uri("/delete/deletekey")
            .to_request();
        let resp = test::call_service(&mut app, delete_req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap_or("");
        assert!(body_str.contains("deleted"));
    }

    #[actix_rt::test]
    async fn test_namespace_isolation() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Create item in namespace1
        let req1 = test::TestRequest::put()
            .uri("/update/nskey/value1?ns=namespace1")
            .to_request();
        let resp1 = test::call_service(&mut app, req1).await;
        assert_eq!(resp1.status(), http::StatusCode::OK);

        // Create item with same key in namespace2
        let req2 = test::TestRequest::put()
            .uri("/update/nskey/value2?ns=namespace2")
            .to_request();
        let resp2 = test::call_service(&mut app, req2).await;
        assert_eq!(resp2.status(), http::StatusCode::OK);

        // Retrieve from namespace1 and verify it has value1
        let get_req1 = test::TestRequest::get()
            .uri("/get/nskey?ns=namespace1")
            .to_request();
        let resp_get1 = test::call_service(&mut app, get_req1).await;
        assert_eq!(resp_get1.status(), http::StatusCode::OK);
        let body1 = test::read_body(resp_get1).await;
        assert_eq!(body1, "value1");

        // Retrieve from namespace2 and verify it has value2
        let get_req2 = test::TestRequest::get()
            .uri("/get/nskey?ns=namespace2")
            .to_request();
        let resp_get2 = test::call_service(&mut app, get_req2).await;
        assert_eq!(resp_get2.status(), http::StatusCode::OK);
        let body2 = test::read_body(resp_get2).await;
        assert_eq!(body2, "value2");
    }

    #[actix_rt::test]
    async fn test_psk_write_required() {
        std::env::set_var("LITTLE_LOOKUP_PSK_WRITE", "test_secret");
        
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Try to update without PSK
        let req = test::TestRequest::put()
            .uri("/update/pskkey/pskvalue")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap_or("");
        assert!(body_str.contains("PSK required"));

        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
    }

    #[actix_rt::test]
    async fn test_psk_write_valid() {
        std::env::set_var("LITTLE_LOOKUP_PSK_WRITE", "correct_secret");
        
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Update with correct PSK
        let req = test::TestRequest::put()
            .uri("/update/pskkey/pskvalue?psk=correct_secret")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
    }

    #[actix_rt::test]
    async fn test_psk_write_incorrect() {
        std::env::set_var("LITTLE_LOOKUP_PSK_WRITE", "correct_secret");
        
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Try to update with incorrect PSK
        let req = test::TestRequest::put()
            .uri("/update/pskkey/pskvalue?psk=wrong_secret")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap_or("");
        assert!(body_str.contains("Incorrect PSK"));

        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
    }

    #[actix_rt::test]
    async fn test_psk_read_required() {
        std::env::set_var("LITTLE_LOOKUP_PSK_READ", "read_secret");
        
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Try to read without PSK
        let req = test::TestRequest::get()
            .uri("/get/somekey")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");
    }

    #[actix_rt::test]
    async fn test_malformed_query_string() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Test with malformed query string (missing value)
        let req = test::TestRequest::get()
            .uri("/list?delim")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        // Should still return OK, just ignoring the malformed parameter
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_query_string_with_multiple_params() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Test with multiple query parameters
        let req = test::TestRequest::get()
            .uri("/list?ns=testns&delim=|")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_update_overwrites_existing() {
        let pool = establish_connection().expect("Failed to establish connection");
        let mut app = create_test_app(pool).await;

        // Create initial value
        let req1 = test::TestRequest::put()
            .uri("/update/overwritekey/value1")
            .to_request();
        let resp1 = test::call_service(&mut app, req1).await;
        assert_eq!(resp1.status(), http::StatusCode::OK);

        // Overwrite with new value
        let req2 = test::TestRequest::put()
            .uri("/update/overwritekey/value2")
            .to_request();
        let resp2 = test::call_service(&mut app, req2).await;
        assert_eq!(resp2.status(), http::StatusCode::OK);

        // Verify new value is retrieved
        let get_req = test::TestRequest::get()
            .uri("/get/overwritekey")
            .to_request();
        let resp_get = test::call_service(&mut app, get_req).await;
        let body = test::read_body(resp_get).await;
        assert_eq!(body, "value2");
    }
}

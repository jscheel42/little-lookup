// openssl needs to come before diesel https://github.com/emk/rust-musl-builder
extern crate openssl;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate actix_web;
extern crate dotenv;
extern crate openssl_probe;

pub mod db_connection;
pub mod handlers;
pub mod models;
pub mod schema;
pub mod util;

use actix_web::{App, HttpServer, web::{self,Data}};
use db_connection::{establish_connection, run_sql_schema_migrations};
use util::get_worker_num;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // // Toggle for debugging
    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    openssl_probe::init_ssl_cert_env_vars();
    dotenv::dotenv().unwrap_or_default();

    run_sql_schema_migrations();

    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(establish_connection().unwrap()))
            .service(
                web::resource("/")
                    .route(web::get().to(handlers::items::index))
            )
            .service(
                web::resource("/delete/{id}")
                    .route(web::get().to(handlers::items::delete_item))
            )
            .service(
                web::resource("/get/{id}")
                    .route(web::get().to(handlers::items::get_item))
            )
            .service(
                web::resource("/history/{id}")
                    .route(web::get().to(handlers::items::history_item))
            )
            .service(
                web::resource("/update/{id}/{val}")
                    .route(web::get().to(handlers::items::update_item))
            )
            .service(
                web::resource("/list")
                    .route(web::get().to(handlers::items::list_items))
            )
            .service(
                web::resource("/script")
                    .route(web::get().to(handlers::items::script))
            )
    })
    .bind("0.0.0.0:8088")?
    .workers(get_worker_num())
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::test;

    use super::*;
    #[actix_rt::test]
    async fn test_index() {
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(establish_connection().unwrap()))
                .service(web::resource("/").route(web::get().to(handlers::items::index))),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_item() {
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(establish_connection().unwrap()))
                .service(
                    web::resource("/delete/{id}")
                        .route(web::get().to(handlers::items::delete_item)),
                ),
        )
        .await;

        let req = test::TestRequest::get().uri("/delete/1").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    // Add more test cases for other handlers...

    #[actix_rt::test]
    async fn test_list_items() {
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(establish_connection().unwrap()))
                .service(web::resource("/list").route(web::get().to(handlers::items::list_items))),
        )
        .await;

        let req = test::TestRequest::get().uri("/list").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}

// openssl needs to come before diesel https://github.com/emk/rust-musl-builder
extern crate openssl;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate actix;
extern crate actix_web;
extern crate dotenv;
extern crate futures;
extern crate openssl_probe;

pub mod db_connection;
pub mod handlers;
pub mod models;
pub mod schema;
pub mod util;

use actix_web::{App, HttpServer, web};
use db_connection::{establish_connection};
use util::{get_worker_num};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    openssl_probe::init_ssl_cert_env_vars();
    dotenv::dotenv().unwrap_or_default();

    HttpServer::new(|| {
        App::new()
            .data(establish_connection())
            .service(
                web::resource("/")
                    .route(web::get().to(handlers::items::index))
            )
            .service(
                web::resource("/delete/{id}")
                    .route(web::get().to(handlers::items::delete_item))
            )
            .service(
                web::resource("/item/{id}")
                    .route(web::get().to(handlers::items::get_item))
            )
            .service(
                web::resource("/item/{id}/{val}")
                    .route(web::get().to(handlers::items::update_item))
            )
            .service(
                web::resource("/list")
                    .route(web::get().to(handlers::items::list_items))
            )
    })
    .bind("0.0.0.0:8088")?
    .workers(get_worker_num())
    .run()
    .await
}

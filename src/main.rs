#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate actix;
extern crate actix_web;
extern crate futures;

pub mod db_connection;
pub mod handlers;
pub mod models;
pub mod schema;

use actix_web::{App, HttpServer, web};
use db_connection::establish_connection;

fn main() {
    HttpServer::new(|| {
        App::new()
            .data(establish_connection())
            .service(
                web::resource("/")
                    .route(web::get().to_async(handlers::items::index))
            )
            .service(
                web::resource("/delete/{id}")
                    .route(web::get().to_async(handlers::items::delete_item))
            )
            .service(
                web::resource("/item/{id}")
                    .route(web::get().to_async(handlers::items::get_item))
            )
            .service(
                web::resource("/item/{id}/{val}")
                    .route(web::get().to_async(handlers::items::update_item))
            )
            .service(
                web::resource("/list")
                    .route(web::get().to_async(handlers::items::list_items))
            )
    })
    .bind("0.0.0.0:8088")
    .unwrap()
    .run()
    .unwrap();
}

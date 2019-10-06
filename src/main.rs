#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod models;
pub mod schema;
pub mod db_connection;
pub mod handlers;

extern crate actix;
extern crate actix_web;
extern crate futures;

use actix_web::{App, HttpServer, web};
use db_connection::establish_connection;

diesel_migrations::embed_migrations!();


fn main() {
    // prepare_database();
    // let connection = get_connection();
    
    HttpServer::new(|| {
        App::new()
            .data(establish_connection())
            .service(
                web::resource("/")
                    .route(web::get().to(index))
            )
            .service(
                web::resource("/delete/{id}")
                    .route(web::get().to(delete_item))
            )
            .service(
                web::resource("/item/{id}")
                    .route(web::get().to(get_item))
            )
            .service(
                web::resource("/item/{id}/{val}")
                    .route(web::get().to(update_item))
            )
            .service(
                web::resource("/list")
                    .route(web::get().to(list_items))
            )
    })
    .bind("0.0.0.0:8088")
    .unwrap()
    .run()
    .unwrap();
}

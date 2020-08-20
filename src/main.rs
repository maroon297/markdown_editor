#[macro_use]
extern crate diesel;
extern crate dotenv;

use actix_web::{middleware, web, App, HttpServer};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
mod editors;
mod articles;
mod schema;
mod models;
mod payloads;
mod responses;
mod actions;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<MysqlConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind = "127.0.0.1:8080";

    println!("Starting server at: {}", &bind);
    
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(
                IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(false)
                        .max_age(300)
                )
            )
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/user")
                    .service(web::resource("/get/{user_id}").route(web::get().to(actions::get_editor)))
                    .service(web::resource("/add").route(web::post().to(actions::add_editor)))
                    .service(web::resource("/login").route(web::post().to(actions::login)))
                    .service(web::resource("/update").route(web::put().to(actions::update_password)))
            )
            .service(
                web::scope("/article")
                    .service(web::resource("/add").route(web::post().to(actions::create_article)))
                    .service(web::resource("/titles").route(web::get().to(actions::get_title_list)))
                    .service(web::resource("/get/{article_id}").route(web::get().to(actions::get_article)))
                    .service(web::resource("/update").route(web::put().to(actions::update_article)))
                    .service(web::resource("/delete").route(web::post().to(actions::delete_article)))
            )
    })
    .bind(&bind)?
    .run()
    .await
}



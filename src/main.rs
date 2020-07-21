#[macro_use]
extern crate diesel;
extern crate dotenv;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use bcrypt::{DEFAULT_COST, hash, verify};
mod editors;
mod schema;
mod models;
mod payloads;

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

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
            .wrap(middleware::Logger::default())
            .service(web::resource("/user/{user_id}").route(web::get().to(get_editor)))
            .service(web::resource("/user").route(web::post().to(add_editor)))
            .service(web::resource("/login").route(web::post().to(auth_editor)))
    })
    .bind(&bind)?
    .run()
    .await
}


async fn get_editor(
    pool: web::Data<DbPool>,
    editor_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let id = editor_id.into_inner();
    println!("editor_id:{}", id);
    let editor = web::block(move || editors::find_editor(&id, &conn))
        .await 
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(editor) = editor {
        Ok(HttpResponse::Ok().json(editor))
    } else {
        let res = HttpResponse::NotFound() 
            .body(format!("No user found with uid"));
        Ok(res)
    }
}

async fn auth_editor(
    pool: web::Data<DbPool>,
    login_req: web::Json<payloads::LoginReq>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let inner = login_req.into_inner();
    let id = inner.editor_name;
    println!("editor_id:{}", id);
    let editor = web::block(move || editors::find_editor(&id, &conn))
        .await 
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(editor) = editor {
        let hashed_pass = editor.password;
        let valid = verify(inner.password, &hashed_pass).expect("verify faild.");
        if valid {
            Ok(HttpResponse::Ok().body(format!("OK")))
        } else{
            Ok(HttpResponse::Ok().body(format!("NG")))
        }            
    } else {
        let res = HttpResponse::NotFound() 
            .body(format!("No user found with uid"));
        Ok(res)
    }
}

async fn add_editor(
    pool: web::Data<DbPool>,
    info: web::Json<payloads::CreateEditorReq>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let inner = info.into_inner();
    let hashed_password = hash(inner.password,DEFAULT_COST).expect("password hash failed");
    let add_data = payloads::CreateEditorReq{
        editor_name : inner.editor_name,
        editor_call_name : inner.editor_call_name,
        password : hashed_password,
    };
    // use web::block to offload blocking Diesel code without blocking server thread
    let editor = web::block(move || editors::add_editor(add_data, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(editor))
}

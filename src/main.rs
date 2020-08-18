#[macro_use]
extern crate diesel;
extern crate dotenv;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use bcrypt::{DEFAULT_COST, hash, verify};
mod editors;
mod articles;
mod schema;
mod models;
mod payloads;
mod responses;

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
                    .service(web::resource("/get/{user_id}").route(web::get().to(get_editor)))
                    .service(web::resource("/add").route(web::post().to(add_editor)))
                    .service(web::resource("/login").route(web::post().to(login)))
                    .service(web::resource("/update").route(web::put().to(update_password)))
            )
            .service(
                web::scope("/article")
                    .service(web::resource("/add").route(web::post().to(create_article)))
                    .service(web::resource("/titles").route(web::get().to(get_title_list)))
                    .service(web::resource("/get/{article_id}").route(web::get().to(get_article)))
                    .service(web::resource("/update").route(web::put().to(update_article)))
                    .service(web::resource("/delete").route(web::post().to(delete_article)))
            )
    })
    .bind(&bind)?
    .run()
    .await
}

// エディター情報取得
async fn get_editor(
    pool: web::Data<DbPool>,
    editor_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    //コネクションプール取得
    let conn = pool.get().expect("couldn't get db connection from pool");
    //idを変数に格納
    let id = editor_id.into_inner();
    let editor = web::block(move || editors::find_editor(id, &conn))
        .await 
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    //エディターが存在する場合はそれを返す
    if let Some(editor) = editor {
        Ok(HttpResponse::Ok().json(editor))
    } else {
        let res = HttpResponse::NotFound() 
            .body(format!("No user found with uid"));
        Ok(res)
    }
}

async fn login(
    pool: web::Data<DbPool>,
    auth_id: Identity,
    login_req: web::Json<payloads::LoginReq>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let inner = login_req.into_inner();
    let password = inner.password.clone();
    let editor = web::block(move || editors::find_editor(inner.editor_name.clone(), &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(editor) = editor {
        let hashed_pass = editor.password;
        let valid = verify(password, &hashed_pass).expect("verify faild.");
        if valid {
            auth_id.remember(editor.editor_name);
            Ok(HttpResponse::NoContent().finish())
        } else{
            Ok(HttpResponse::Unauthorized().finish())
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
    web::block(move || editors::add_editor(add_data, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Created().finish())
}

async fn update_password(
    pool: web::Data<DbPool>,
    auth_id: Identity,
    update_req: web::Json<payloads::UpdatePasswordReq>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let inner = update_req.into_inner();
    //sessionからeditor_idを取得
    let id = if let Some(editor_id) = auth_id.identity() {
        auth_id.remember(editor_id.clone());
        editor_id.clone()
    } else {
        auth_id.forget();
       return Ok(HttpResponse::Unauthorized().finish());
    };
    let id_clone = id.clone();
    let editor = web::block(move || editors::find_editor(id.clone(), &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    
    let conn = pool.get().expect("couldn't get db connection from pool");
    if let Some(editor) = editor {
        let hashed_pass = editor.password;
        let valid = verify(inner.password, &hashed_pass).expect("verify faild.");
        if valid {
            let hashed_pass_again = hash(inner.password_again,DEFAULT_COST).expect("password_again hash failed");
            web::block(move || editors::update_password(id_clone,hashed_pass_again,&conn))
                .await
                .map_err(|e| {
                    eprintln!("{}", e);
                    HttpResponse::InternalServerError().finish()
                })?;
            Ok(HttpResponse::NoContent().finish())
        } else{
            auth_id.forget();
            Ok(HttpResponse::Unauthorized().finish())
        }            
    } else {
        let res = HttpResponse::NotFound() 
            .body(format!("No user found with uid"));
        Ok(res)
    }
}

async fn create_article(
    pool: web::Data<DbPool>,
    auth_id: Identity,
    create_req: web::Json<payloads::CreateArticleReq>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let inner = create_req.into_inner();
    //sessionからeditor_idを取得
    let id = if let Some(editor_id) = auth_id.identity() {
        auth_id.remember(editor_id.clone());
        editor_id.clone()
    } else {
        auth_id.forget();
       return Ok(HttpResponse::Unauthorized().finish());
    };
    let editor = web::block(move || editors::find_editor(id, &conn))
        .await 
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    let editor_id = if let Some(editor) = editor {
        editor.editor_id
    } else {
        return Ok(HttpResponse::NotFound().finish())
    };
    let add_data = payloads::CreateArticleDbReq{
        author_id : editor_id,
        title : inner.title,
        content : inner.content,
    };    
    
    // use web::block to offload blocking Diesel code without blocking server thread
    let conn = pool.get().expect("couldn't get db connection from pool");
    web::block(move || articles::add_article(add_data, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Created().finish())
}

async fn get_title_list(
    pool: web::Data<DbPool>,
    auth_id: Identity,
) -> Result<HttpResponse, Error> {
    //コネクションプール取得
    let conn = pool.get().expect("couldn't get db connection from pool");
    //sessionからeditor_idを取得
    let id = if let Some(editor_id) = auth_id.identity() {
        auth_id.remember(editor_id.clone());
        editor_id.clone()
    } else {
        auth_id.forget();
       return Ok(HttpResponse::Unauthorized().finish());
    };
    let editor_res = web::block(move || editors::find_editor(id, &conn))
        .await 
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    
    let author_id = if let Some(editor) = editor_res {
        editor.editor_id
    } else {
        let res = HttpResponse::NotFound() 
            .body(format!("No user found with uid"));
        return Ok(res)
    };

    let conn = pool.get().expect("couldn't get db connection from pool");
    let titles_res = web::block(move || articles::get_titles(author_id, &conn))
        .await 
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    let mut res_list: Vec<responses::TitlesRes> = Vec::new();
    for title in titles_res {
        res_list.push(
            responses::TitlesRes {
                article_id : title.id,
                title : title.title,
            }
        );
    }
    Ok(HttpResponse::Ok().json(res_list))
}

// 記事情報取得
async fn get_article(
    pool: web::Data<DbPool>,
    article_id: web::Path<i64>,
    auth_id: Identity,
) -> Result<HttpResponse, Error> {
    //sessionからeditor_idを取得
    if let Some(editor_id) = auth_id.identity() {
        auth_id.remember(editor_id.clone());
    } else {
        auth_id.forget();
       return Ok(HttpResponse::Unauthorized().finish());
    };
    //コネクションプール取得
    let conn = pool.get().expect("couldn't get db connection from pool");
    //idを変数に格納
    let id = article_id.into_inner();
    let article = web::block(move || articles::find_article(id, &conn))
        .await 
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    //エディターが存在する場合はそれを返す
    if let Some(article) = article {
        Ok(HttpResponse::Ok().json(article))
    } else {
        let res = HttpResponse::NotFound() 
            .body(format!("No user found with uid"));
        Ok(res)
    }
}

async fn update_article(
    pool: web::Data<DbPool>,
    auth_id: Identity,
    update_req: web::Json<payloads::UpdateArticleReq>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let inner = update_req.into_inner();
    //sessionからeditor_idを取得
    if let Some(editor_id) = auth_id.identity() {
        auth_id.remember(editor_id.clone());
    } else {
        auth_id.forget();
       return Ok(HttpResponse::Unauthorized().finish());
    };
    web::block(move || articles::update_article(inner.id,inner.title,inner.content,&conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(HttpResponse::NoContent().finish())
}

async fn delete_article(
    pool: web::Data<DbPool>,
    auth_id: Identity,
    delete_req: web::Json<payloads::DeleteArticleReq>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let inner = delete_req.into_inner();
    //sessionからeditor_idを取得
    if let Some(editor_id) = auth_id.identity() {
        auth_id.remember(editor_id.clone());
    } else {
        auth_id.forget();
       return Ok(HttpResponse::Unauthorized().finish());
    };
    web::block(move || articles::delete_article(inner.id,&conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(HttpResponse::NoContent().finish())
}

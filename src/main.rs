mod middleware;
mod services;
mod models;
mod repository;

use std::io;
use std::time::Duration;
// use std::io::Write;

// use chrono::Local;
// use log::LevelFilter;
use log4rs;
use futures::{future::ok, stream::once};
use derive_more::{Display, Error};

use actix_cors::Cors;
use actix_web::http::{StatusCode};
use actix_web::{http, get, post, web, error, web::Data, App, Error, Result, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use middleware::access_filter::Logger;

use services::user_service::{create_user, get_user, update_user, delete_user, get_all_users};
use repository::mongodb_repo::MongoRepo;

#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", name)]
struct MyError {
    name: &'static str,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for MyError {}


#[get("/favicon.ico")]
async fn favicon(_req: HttpRequest) -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/favicon.ico")?)
}

#[get("/favicon.svg")]
async fn favicon_svg(_req: HttpRequest) -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/favicon.svg")?)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/readme")]
async fn readme(_req: HttpRequest) -> io::Result<NamedFile> {
    Ok(NamedFile::open("README.md")?)
}

// Response body can be generated asynchronously. 
// In this case, body must implement the stream trait Stream<Item=Bytes, Error=Error>, i.e.:
#[get("/stream")]
async fn stream() -> HttpResponse {

    let body = once(ok::<_, Error>(web::Bytes::from_static(b"test")));

    HttpResponse::Ok()
        .content_type("text/plain;charset=utf-8")
        .streaming(body)
}

#[get("/errors")]
async fn errors() -> Result<&'static str, MyError> {
    Err(MyError { name: "MyError,粗欧文" })
}

#[get("/throw-error/{id}")]
async fn throw_error(id: web::Path<u32>) -> Result<HttpResponse, MyError> {
    let user_id: u32 = id.into_inner();
    log::info!("userId: {}", user_id);
    Err(MyError { name: "MyError,粗欧文" })
}

async fn about() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html;charset=utf-8")
        .body("<h1>About</h1>"))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok()
    .content_type("text/plain;charset=utf-8")
    .body("Hey there! 啊啊送积分啦；送积分啦")
}

async fn not_found() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html;charset=utf-8")
        .body("<h1>404 - Page not found</h1>"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // std::env::set_var("RUST_LOG", "info");
    // std::env::set_var("RUST_BACKTRACE", "1");
    
    log4rs::init_file("resources/log4rs.yaml", Default::default()).unwrap();
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();


    let db = MongoRepo::init().await;
    let db_data = Data::new(db);

    HttpServer::new(move || {
    
        let logger = Logger::new("%{r}a \"%r\" %s %b/bytes %Dms")
                           // .log_target("http_log")
                           .exclude("/favicon.ico");

        let cors = Cors::default()
              .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "PATCH"])
              .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
              .allowed_header(http::header::CONTENT_TYPE)
              .max_age(3600);

        App::new()
            .app_data(db_data.clone())
            .wrap(cors)
            .wrap(logger)
            .service(favicon)
            .service(favicon_svg)
            .service(hello)
            .service(echo)
            .service(stream)
            .service(readme)
            .service(create_user)
            .service(get_user) //add this
            .service(update_user) //add this
            .service(delete_user) //add this
            .service(get_all_users)//add this
            .service(errors)
            .service(throw_error)
            .default_service(
                web::route().to(not_found)
            )
            .route("/hey", web::get().to(manual_hello))
            .route("/about", web::get().to(about))
            .route("/throw-error", web::get().to(about))

        log::info!("booting up");
    
    })
    .keep_alive(Duration::from_secs(75))
    .bind(("127.0.0.1", 8000))?
    .bind_openssl("127.0.0.1:8443", builder)?
    .run()
    .await

}


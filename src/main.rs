mod middlewares;
mod services;
mod models;
mod repository;

use std::io;
use std::{time::Duration};

use log4rs;
use futures::{future::ok, stream::once};
use derive_more::{Display, Error};

use actix_cors::Cors;
use actix_web::web::ServiceConfig;
use actix_web::http::{StatusCode};
use actix_web::{http, get, web, middleware, error, web::Data, App, Error, Result, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use actix_extensible_rate_limit::{backend::memory::InMemoryBackend, backend::SimpleInputFunctionBuilder, RateLimiter};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use middlewares::access_filter::Logger;

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

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn readme(_req: HttpRequest) -> io::Result<NamedFile> {
    Ok(NamedFile::open("README.md")?)
}

// Response body can be generated asynchronously. 
// In this case, body must implement the stream trait Stream<Item=Bytes, Error=Error>, i.e.:
async fn stream() -> HttpResponse {
    let body = once(ok::<_, Error>(web::Bytes::from_static(b"test")));
    HttpResponse::Ok()
        .content_type("text/plain;charset=utf-8")
        .streaming(body)
}

async fn greet(req: HttpRequest) -> impl Responder{
    let name = req.match_info().get("name").unwrap_or("World!");
    format!("Hello {}!", &name)
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

async fn dashboard() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html;charset=utf-8")
        .body("<h1>Dashboard</h1>"))
}

async fn not_found() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html;charset=utf-8")
        .body("<h1>404 - Page not found</h1>"))
}

pub fn static_handler(config: &mut ServiceConfig) {
    // let static_path =
    //     std::env::var("STATIC_ROOT").expect("Running in debug without STATIC_ROOT set!");
    let static_path = "static";
    let fs = actix_files::Files::new("/static", &static_path);
    config.service(fs);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // std::env::set_var("RUST_LOG", "info");
    // std::env::set_var("RUST_BACKTRACE", "1");
    
    log4rs::init_file("resources/log4rs.yaml", Default::default()).unwrap();
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    // let store = MemoryStore::new();

    let db = MongoRepo::init().await;
    let db_data = Data::new(db);

    log::info!("booting up");

    HttpServer::new(move || {
    
        let logger = Logger::new("%{r}a \"%r\" %s %b %D")
                           .exclude("/favicon.ico");

        let cors = Cors::default()
              .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "PATCH"])
              .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
              .allowed_header(http::header::CONTENT_TYPE)
              .max_age(3600);

        let access_limiter = RateLimiter::builder(
            InMemoryBackend::builder().build(), 
            SimpleInputFunctionBuilder::new(Duration::from_secs(1), 5).real_ip_key().build()
        ).add_headers().build();

        App::new()
            .route("/", web::get().to(greet))
            .app_data(db_data.clone())
            .wrap(cors)
            .wrap(logger)
            .wrap(access_limiter)
            .wrap(middleware::NormalizePath::new(middleware::TrailingSlash::Trim))
            .service(favicon)
            .service(favicon_svg)
            .route("/", web::get().to(index))
            .route("/index", web::get().to(index))
            .route("/home", web::get().to(index))
            .route("/stream", web::get().to(stream))
            .route("/readme", web::get().to(readme))
            .route("/echo", web::post().to(echo))
            // user
            .service(create_user)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .service(get_all_users)
            .service(
                web::scope("/developer")
                    // .wrap(RateLimiter::default())
                    // .app_data(limiter.clone())
                    .route("/", web::get().to(dashboard))
                    .route("/index", web::get().to(dashboard))
                    .route("/home", web::get().to(dashboard))
            )
            // error
            .service(errors)
            .service(throw_error)
            .default_service(
                web::route().to(not_found)
            )
            .route("/hey", web::get().to(|| async { "Hey there! 啊啊送积分啦；送积分啦" }))
            .route("/about", web::get().to(about))
            .route("/throw-error", web::get().to(about))
            .configure(static_handler)
    
    })
    .keep_alive(Duration::from_secs(75))
    .bind(("127.0.0.1", 8000))?
    .bind_openssl("127.0.0.1:8443", builder)?
    .run()
    .await

}

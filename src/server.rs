mod middlewares;
mod services;
mod models;
mod repository;

extern crate log;
use std::{io, time::Duration};

use log4rs;
use futures::{future::ok, stream::once};
use derive_more::{Display, Error};

use actix_cors::Cors;
use actix_web::web::ServiceConfig;
use actix_web::{http, get, web, middleware, error, web::Data, App, Error, Result, dev::ServiceRequest, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use actix_extensible_rate_limit::{backend::memory::InMemoryBackend, backend::SimpleInputFunctionBuilder, backend::SimpleInput, backend::SimpleOutput, RateLimiter};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslAcceptorBuilder};

use crate::middlewares::access_filter::Logger;
use crate::services::user_service::{create_user, get_user, update_user, delete_user, get_all_users};
use crate::repository::mongodb_repo::MongoRepo;


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

#[get("/errors")]
async fn errors() -> Result<&'static str, MyError> {
    Err(MyError { name: "MyError,粗欧文" })
}

async fn throw_error(id: web::Path<u32>) -> Result<HttpResponse, MyError> {
    let user_id: u32 = id.into_inner();
    log::info!("userId: {}", user_id);
    Err(MyError { name: "MyError,粗欧文" })
}

async fn about() -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html;charset=utf-8")
        .body("<h1>About</h1>"))
}

async fn developer() -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html;charset=utf-8")
        .body("<h1>Developer</h1>"))
}

async fn not_found() -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
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

pub fn tls_builder() -> SslAcceptorBuilder {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    return builder
}

pub fn cors() -> Cors{
    return Cors::default()
    .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "PATCH"])
    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
    .allowed_header(http::header::CONTENT_TYPE)
    .max_age(3600);
}

pub fn access_limiter() -> RateLimiter<InMemoryBackend, SimpleOutput, impl Fn(&ServiceRequest) -> std::future::Ready<Result<SimpleInput, Error>>>{
    return RateLimiter::builder(
        InMemoryBackend::builder().build(), 
        SimpleInputFunctionBuilder::new(Duration::from_secs(1), 5).real_ip_key().build()
    ).add_headers().build();
}

/// This struct provides a slightly simpler way to write `main.rs` in
/// the root project, and forces more coupling to app-specific modules.
pub struct Server {
    // apps: Vec<Box<dyn Fn(&mut ServiceConfig) + Send + Sync + 'static>>,
}

impl Server {

    // Creates a new Server struct to configure.
    pub fn new() -> Self {
        Self {
            // apps: vec![],
        }
    }

    pub async fn run(self) {

        // std::env::set_var("RUST_LOG", "info");
        // std::env::set_var("RUST_BACKTRACE", "1");
        
        log4rs::init_file("resources/log4rs.yaml", Default::default()).unwrap();

        let db_data = Data::new(MongoRepo::init().await);

        log::info!("booting up");

        HttpServer::new(move || {
        
            let logger = Logger::new("%{r}a \"%r\" %s %b %D")
                .exclude("/favicon.ico")
                .exclude("/favicon.svg")
                .exclude_regex("^/static")
            ;

            App::new()
                .configure(static_handler)
                .app_data(db_data.clone())
                .wrap(cors())
                .wrap(logger)
                .wrap(middleware::NormalizePath::new(middleware::TrailingSlash::Trim))
                .service(favicon)
                .service(favicon_svg)
                .route("/", web::get().to(index))
                .route("/index", web::get().to(index))
                .route("/home", web::get().to(index))
                .route("/stream", web::get().to(stream))
                .route("/readme", web::get().to(readme))
                .route("/echo", web::post().to(echo))
                .route("/hey", web::get().to(|| async { "Hey there! 啊啊送积分啦；送积分啦" }))
                .route("/about", web::get().to(about))
                .route("/throw-error/{id}", web::get().to(throw_error))
                // user
                .service(create_user)
                .service(get_user)
                .service(update_user)
                .service(delete_user)
                .service(get_all_users)
                // developers
                .service(
                    web::scope("/developer")
                        .wrap(access_limiter())
                        .route("", web::get().to(developer))
                        .route("/index", web::get().to(developer))
                        .route("/home", web::get().to(developer))
                )
                // error handle
                .service(errors)
                .default_service(
                    web::route().to(not_found)
                )
        
        })
        .backlog(8192)
        .workers(4)
        .keep_alive(Duration::from_secs(75))
        .bind(format!("0.0.0.0:{}", 8000))
        .unwrap_or_else(|_| panic!("🔥 Couldn't start the server at port {}", 8000))
        .bind_openssl("127.0.0.1:8443", tls_builder())
        .expect("Failed to bind to port: 8443")
        .run()
        .await
        .expect("Failed to run server")
        
    }

}

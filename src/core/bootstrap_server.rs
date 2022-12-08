extern crate log;
use std::{time::Duration};

// log
use log4rs;

// tls
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslAcceptorBuilder};

// actix
use actix_web;
use actix_web::http::Method;
use actix_web::{http, web, dev, middleware, web::Data, Route, App, HttpServer, };
use actix_cors::Cors;

// actix extens
use actix_extensible_rate_limit::{backend::memory::InMemoryBackend, backend::SimpleInputFunctionBuilder, backend::SimpleInput, backend::SimpleOutput, RateLimiter};

// html template
use tera::{Tera};

// middlewares
use crate::middlewares::access_filter::Logger;

use crate::utils;
use crate::core::builtin_handles;

use crate::repository::mongodb_repo::MongoRepo;
use crate::services::user_service::{create_user, get_user, update_user, delete_user, get_all_users};

pub struct Server {
    // apps: Vec<Box<dyn Fn(&mut ServiceConfig) + Send + Sync + 'static>>,
}

fn config(cfg: &mut web::ServiceConfig) {

    let h1: Route = Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::stream);
    let r1 = ("/stream", h1);

    cfg.service(builtin_handles::favicon)
       .service(builtin_handles::favicon_svg);

    cfg.route("/", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::index))
       .route("/index", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::index))
       .route("/home", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::index))
       .route(r1.0, r1.1)
       .route("/readme", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::readme))
       .route("/info", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::info))
       .route("/hey", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(|| async { "Hey there! 啊啊送积分啦；送积分啦" }))
       .route("/about", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::about))
       .route("/throw-error/{id}", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::throw_error))
       .route("/graphiql", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::graphiql))
       .route("/mandelbrot", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::mandelbrot)); // 曼德布洛特集合绘制的灰度图片

    // user
    cfg.service(create_user)
       .service(get_user)
       .service(update_user)
       .service(delete_user)
       .service(get_all_users);

    // developers
    cfg.service(
           web::scope("/developer")
               .wrap(access_limiter())
               .route("", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::developer))
               .route("/index", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::developer))
               .route("/home", Route::new().method(Method::from_bytes(b"GET").unwrap()).to(builtin_handles::developer))
       )
       // error handle
       .service(builtin_handles::errors);

    cfg.configure(builtin_handles::static_handler);
    
}

fn cors() -> Cors{
    Cors::default() 
    .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "PATCH"])
    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
    .allowed_header(http::header::CONTENT_TYPE)
    .max_age(3600)
}

fn tls_builder() -> SslAcceptorBuilder {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    return builder
}

fn access_limiter() -> RateLimiter<InMemoryBackend, SimpleOutput, impl Fn(&dev::ServiceRequest) -> std::future::Ready<Result<SimpleInput, actix_web::Error>>>{
    return RateLimiter::builder(
        InMemoryBackend::builder().build(), 
        SimpleInputFunctionBuilder::new(Duration::from_secs(1), 5).real_ip_key().build()
    ).add_headers().build();
}

impl Server {

    // Creates a new Server struct to configure.
    pub fn new() -> Self {
        Self {
            // apps: vec![],
        }
    }

    pub async fn run(self) {

        // std::env::set_var("RUST_LOG", "debug");
        
        log4rs::init_file("resources/log4rs.yaml", Default::default()).unwrap();

        let db_data = Data::new(MongoRepo::init().await);
        let tmpl_data = Data::new(Tera::new(&[utils::file::ROOT_DIR, "/templates/**/*"].concat().to_string()[..]).unwrap());

        let new_app = move || {
        
            let logger = Logger::new("%{r}a \"%r\" %s %b %D")
                .exclude("/favicon.ico")
                .exclude("/favicon.svg")
                .exclude_regex("^/static")
            ;

            App::new()
                .app_data(tmpl_data.clone())
                .app_data(db_data.clone())
                .wrap(cors())
                .wrap(logger)
                .wrap(middleware::NormalizePath::new(middleware::TrailingSlash::Trim))
                .configure(|wc|config(wc))
                .default_service(
                    web::route().to(builtin_handles::not_found) 
                )
        
        };

        HttpServer::new(new_app)
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
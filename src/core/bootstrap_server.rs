extern crate log;
use std::{time::Duration};

use log4rs;

use actix_web::{web, middleware, web::Data, App, HttpServer};
use tera::{Tera};

use crate::middlewares::access_filter::Logger;

use crate::core::builtin_handles::{favicon, favicon_svg}; 
use crate::core::builtin_handles::{index, echo, readme, stream, about, developer}; 
use crate::core::builtin_handles::{errors, throw_error, not_found}; 
use crate::core::builtin_handles::{static_handler, tls_builder, cors, access_limiter}; 

use crate::repository::mongodb_repo::MongoRepo;
use crate::services::user_service::{create_user, get_user, update_user, delete_user, get_all_users};

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

        // std::env::set_var("RUST_LOG", "debug");
        
        log4rs::init_file("resources/log4rs.yaml", Default::default()).unwrap();

        let db_data = Data::new(MongoRepo::init().await);
        let tmpl_data = Data::new(Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap());
        
        log::info!("booting up, CARGO_MANIFEST_DIR={}", concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"));

        HttpServer::new(move || {
        
            let logger = Logger::new("%{r}a \"%r\" %s %b %D")
                .exclude("/favicon.ico")
                .exclude("/favicon.svg")
                .exclude_regex("^/static")
            ;

            App::new()
                .configure(static_handler)
                .app_data(tmpl_data.clone())
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
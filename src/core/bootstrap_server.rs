extern crate log;
use std::time::Duration;

// log
use log4rs;

// tls
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

// actix
use actix_cors::Cors;
use actix_web;
use actix_web::http::Method;
use actix_web::{dev, http, middleware, web, web::Data, App, HttpServer, Route};

// actix extens
use actix_extensible_rate_limit::{
    backend::memory::InMemoryBackend, backend::SimpleInput, backend::SimpleInputFunctionBuilder,
    backend::SimpleOutput, RateLimiter,
};

// html template
use tera::Tera;

// middlewares
use crate::middlewares::access_filter;
// use crate::websocket::lobby::Lobby; // as well as this

use crate::core::builtin_handles;
use crate::utils;

use crate::repository::mongodb_repo::MongoRepo;
use crate::services::user_service::{
    create_user, delete_user, get_all_users, get_user, update_user,
};

pub struct Server {
    // apps: Vec<Box<dyn Fn(&mut ServiceConfig) + Send + Sync + 'static>>,
}

// fn c2(method: &str, path: &str, f: &dyn std::any::Any) -> (&[u8], &str, &(dyn std::any::Any + 'static)){
//     (method.as_bytes(), path, f)
// }

pub fn config(cfg: &mut web::ServiceConfig) {
    let routes = vec![(b"GET", "/developer2", builtin_handles::developer)];

    let r1 = (b"GET", "/stream", builtin_handles::stream);
    let r2 = (b"GET", "/readme", builtin_handles::readme);
    let r3 = (b"GET", "/info", builtin_handles::info);

    cfg.service(builtin_handles::favicon)
        .service(builtin_handles::favicon_svg);

    cfg.route(
        "/",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(builtin_handles::index),
    )
    .route(
        "/index",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(builtin_handles::index),
    )
    .route(
        "/home",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(builtin_handles::index),
    )
    .route(
        r1.1,
        Route::new()
            .method(Method::from_bytes(r1.0).unwrap())
            .to(r1.2),
    )
    .route(
        r2.1,
        Route::new()
            .method(Method::from_bytes(r2.0).unwrap())
            .to(r2.2),
    )
    .route(
        r3.1,
        Route::new()
            .method(Method::from_bytes(r3.0).unwrap())
            .to(r3.2),
    )
    .route(
        "/hey",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(|| async { "Hey there! 啊啊送积分啦；送积分啦" }),
    )
    .route(
        "/about",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(builtin_handles::about),
    )
    .route(
        "/throw-error/{id}",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(builtin_handles::throw_error),
    )
    .route(
        "/graphiql",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(builtin_handles::graphiql),
    )
    .route(
        "/mandelbrot",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(builtin_handles::mandelbrot),
    ) // 曼德布洛特集合绘制的灰度图片
    .route(
        "/speed",
        Route::new()
            .method(Method::from_bytes(b"GET").unwrap())
            .to(builtin_handles::speed),
    );

    log::info!("Router Count {}", routes.len());

    for r in routes {
        log::info!(
            "Added a router: {} {}",
            std::str::from_utf8(r.0).unwrap(),
            r.1
        );
        cfg.route(
            r.1,
            Route::new()
                .method(Method::from_bytes(r.0).unwrap())
                .to(r.2),
        );
    }

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
            .route(
                "",
                Route::new()
                    .method(Method::from_bytes(b"GET").unwrap())
                    .to(builtin_handles::developer),
            )
            .route(
                "/index",
                Route::new()
                    .method(Method::from_bytes(b"GET").unwrap())
                    .to(builtin_handles::developer),
            )
            .route(
                "/home",
                Route::new()
                    .method(Method::from_bytes(b"GET").unwrap())
                    .to(builtin_handles::developer),
            )
            .route(
                "/maxium",
                Route::new()
                    .method(Method::from_bytes(b"GET").unwrap())
                    .to(builtin_handles::maxium),
            )
            .route(
                "/typeOf",
                Route::new()
                    .method(Method::from_bytes(b"GET").unwrap())
                    .to(builtin_handles::type_of),
            ),
    );

    // Add the WebSocket route
    cfg.service(web::resource("/ws").route(web::get().to(builtin_handles::websocket)));
    // cfg.service(web::resource("/ws").route(web::get().to(echo_ws)));

    // simple error handle
    cfg.service(builtin_handles::errors);

    cfg.configure(builtin_handles::static_handler);
}

pub fn cors() -> Cors {
    Cors::default()
        .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "PATCH"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ])
        .supports_credentials()
        .max_age(3600)
}

pub fn tls_builder() -> SslAcceptorBuilder {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    return builder;
}

pub fn access_limiter() -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&dev::ServiceRequest) -> std::future::Ready<Result<SimpleInput, actix_web::Error>>,
> {
    return RateLimiter::builder(
        InMemoryBackend::builder().build(),
        SimpleInputFunctionBuilder::new(Duration::from_secs(1), 5)
            .real_ip_key()
            .build(),
    )
    .add_headers()
    .build();
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

        let server_port = 8001;
        let tls_enable = false;

        log4rs::init_file("resources/log4rs.yaml", Default::default()).unwrap();

        let db_data = Data::new(MongoRepo::init().await);
        let tmpl_data =
            Data::new(Tera::new(&[utils::file::ROOT_DIR, "/templates/**/*"].concat()[..]).unwrap());

        let new_app = move || {
            let logger = access_filter::Logger::new("%{r}a \"%r\" %s %b %D")
                .exclude("/favicon.ico")
                .exclude("/favicon.svg")
                .exclude_regex("^/static");

            App::new()
                .app_data(tmpl_data.clone())
                .app_data(db_data.clone())
                // .wrap(cors())
                .wrap(logger)
                .wrap(middleware::NormalizePath::new(
                    middleware::TrailingSlash::Trim,
                ))
                .configure(|wc| config(wc))
                .default_service(web::route().to(builtin_handles::not_found))
        };

        let server = HttpServer::new(new_app)
            .backlog(8192)
            .workers(1)
            .keep_alive(Duration::from_secs(75));

        let bind_result = if !tls_enable {
            server.bind(format!("0.0.0.0:{}", server_port))
        } else {
            server.bind_openssl(format!("0.0.0.0:{}", server_port), tls_builder())
        };

        match bind_result {
            Ok(svr) => {
                if !tls_enable {
                    log::info!(
                        "Congratulations! Your server will be running at http://0.0.0.0:{}",
                        server_port
                    );
                } else {
                    log::info!(
                        "Congratulations! Your server will be running at https://0.0.0.0:{}",
                        server_port
                    );
                }
                svr.run().await.expect("Failed to run server")
            }
            _ => log::info!("🔥 Couldn't start the server at port {}", server_port),
        }
    }
}

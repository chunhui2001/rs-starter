use std::{io, time::Duration};
use actix_web::{http, get, web, error, web::ServiceConfig, Error, web::{Data}, Result, dev::ServiceRequest, HttpRequest, HttpResponse, Responder};
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_extensible_rate_limit::{backend::memory::InMemoryBackend, backend::SimpleInputFunctionBuilder, backend::SimpleInput, backend::SimpleOutput, RateLimiter};

use futures::{future::ok, stream::once};
use derive_more::{Display, Error};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslAcceptorBuilder};

use tera::{Tera, Context};

use lazy_static::lazy_static;
use regex::Regex;


#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", name)]
pub struct MyError {
    name: &'static str,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for MyError {
    
}

/// Prettify HTML input
pub fn prettify(input: &str) -> String {

    lazy_static! {
        static ref OPEN_TAG: Regex = Regex::new("(?P<tag><[A-z])").unwrap();
        static ref EMPTY_LINE: Regex = Regex::new("(\\s*\n){1,}").unwrap();
        static ref CLOSE_TAG: Regex = Regex::new("([^>\n]\\s*</)").unwrap();
    }

    // First get all tags on their own lines
    let mut stage1 = input.to_string();
    stage1 = stage1.replace("<!--", "\n<!--");
    stage1 = stage1.replace("-->", "-->\n");
    stage1 = stage1.replace("</", "\n</");
    stage1 = OPEN_TAG.replace_all(&stage1, "\n$tag").to_string();
    stage1 = stage1.trim().to_string();

    // Now fix indentation
    let mut stage2: Vec<String> = vec![];
    let mut indent = 0;
    for line in stage1.split('\n') {
        let mut post_add = 0;
        if line.starts_with("</") {
            indent -= 1;
        } else if line.ends_with("/>") || line.starts_with("<!DOCTYPE") || line.starts_with("<meta ") {
            // Self-closing, nothing
            // or DOCTYPE, also nothing
        } else if line.starts_with('<') {
            post_add += 1;
        }

        stage2.push(format!("{}{}", "  ".repeat(indent), line));
        indent += post_add;
    }

    let pretty_html1 = stage2.join("\n");
    // let pretty_html2 = EMPTY_LINE.replace_all(&pretty_html1, "\n").to_string();
    // let pretty_html3 = CLOSE_TAG.replace_all(&pretty_html2, "</").to_string();

    pretty_html1

}


#[get("/favicon.ico")]
pub async fn favicon(_req: HttpRequest) -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/favicon.ico")?)
}

#[get("/favicon.svg")]
pub async fn favicon_svg(_req: HttpRequest) -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/favicon.svg")?)
}

pub async fn index(tmpl: Data<Tera>) -> impl Responder {

    let mut ctx = Context::new();
    ctx.insert("name", "啦啦发啦");

    let render_result = tmpl.render("index.html", &ctx);

    match render_result {
        Ok(rendered) => {
            HttpResponse::Ok().body(rendered)
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }

}

pub async fn graphiql(tmpl: Data<Tera>) -> impl Responder {

    let mut ctx = Context::new();
    ctx.insert("title", "QraphiQl");

    let render_result = tmpl.render("graphiql.html", &ctx);

    match render_result {
        Ok(rendered) => {
            HttpResponse::Ok().body(rendered)
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }

}

pub async fn info() -> impl Responder {
    HttpResponse::Ok().json("Hello, server is alive and kicking.")
}

pub async fn readme(_req: HttpRequest) -> io::Result<NamedFile> {
    Ok(NamedFile::open("README.md")?)
}

pub async fn about() -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html;charset=utf-8")
        .body("<h1>About</h1>"))
}

pub async fn developer(_req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html;charset=utf-8")
        .body("<h1>Developer</h1>"))
}

// Response body can be generated asynchronously. 
// In this case, body must implement the stream trait Stream<Item=Bytes, Error=Error>, i.e.:
pub async fn stream() -> HttpResponse {
    let body = once(ok::<_, Error>(web::Bytes::from_static(b"test")));
    HttpResponse::Ok()
        .content_type("text/plain;charset=utf-8")
        .streaming(body)
}

#[get("/errors")]
pub async fn errors() -> Result<&'static str, MyError> {
    Err(MyError { name: "MyError,粗欧文" })
}

pub async fn throw_error(id: web::Path<u32>) -> Result<HttpResponse, MyError> {
    let user_id: u32 = id.into_inner();
    log::info!("userId: {}", user_id);
    Err(MyError { name: "MyError,粗欧文" })
}

pub async fn not_found(_request: HttpRequest) -> Result<HttpResponse> {
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

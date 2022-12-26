pub mod core;
pub mod mandelbrot;
pub mod middlewares;
pub mod models;
pub mod repository;
pub mod services;
pub mod utils;
pub mod websocket;

use actix_web;

use crate::core::bootstrap_server::Server;

#[actix_web::main]
async fn main() {
    Server::new().run().await
}

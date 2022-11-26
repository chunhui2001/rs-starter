pub mod core;
pub mod middlewares;
pub mod services;
pub mod models;
pub mod repository;

use actix_web;

use crate::core::bootstrap_server::Server;

#[actix_web::main]
async fn main() {
    Server::new().run().await
}
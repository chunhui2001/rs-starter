use mainlib::Server; 
use actix_web;

#[actix_web::main]
async fn main() {
    Server::new().run()
    .await
}
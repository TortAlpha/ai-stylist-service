
use actix_web::{self, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    println!("Starting PRODUCT SERVICE http://localhost:8080 ");

    HttpServer::new(move || {
        App::new()
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
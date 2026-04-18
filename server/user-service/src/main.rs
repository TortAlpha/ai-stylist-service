use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use tracing::info;
use user_service::config::Config;
use user_service::handler;
use user_service::repo::impls::address_repo::PgAddressRepo;
use user_service::repo::impls::user_repo::PgUserRepo;
use user_service::service::user_service::UserService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,actix_web=info"));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .init();

    let config = Config::from_env();

    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let user_service = web::Data::new(UserService::new(
        Arc::new(PgUserRepo::new(pool.clone())),
        Arc::new(PgAddressRepo::new(pool.clone())),
    ));

    info!("Starting USER SERVICE on port {}", config.port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(user_service.clone())
            .service(web::scope("/api").configure(handler::user_handler::configure))
            .configure(handler::internal_handler::configure)
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

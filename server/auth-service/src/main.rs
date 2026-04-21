use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use tracing::info;

use auth_service::auth::jwt::JwtManager;
use auth_service::clients::user_client::UserClient;
use auth_service::config::Config;
use auth_service::handler;
use auth_service::repo::impls::session_repo::PgSessionRepo;
use auth_service::service::auth_service::AuthService;

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

    let jwt_manager = Arc::new(JwtManager::new(
        config.jwt_secret.clone(),
        config.access_token_ttl_seconds,
    ));
    let user_client = Arc::new(UserClient::new(config.user_service_url.clone()));

    let auth_service = web::Data::new(AuthService::new(
        Arc::new(PgSessionRepo::new(pool.clone())),
        jwt_manager,
        user_client,
    ));

    info!("Starting AUTH SERVICE on port {}", config.port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(auth_service.clone())
            .service(web::scope("/api").configure(handler::auth_handler::configure))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

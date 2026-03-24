use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use tracing::info;
use product_service::config::Config;
use product_service::handler;
use product_service::repo::impls::{
    brand_repo::PgBrandRepo, category_repo::PgCategoryRepo,
    product_repo::PgProductRepo, tag_repo::PgTagRepo,
};
use product_service::service::{
    brand_service::BrandService, category_service::CategoryService,
    product_service::ProductService, tag_service::TagService,
};
use product_service::storage::s3::S3ImageStorage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let config = Config::from_env();

    // ── Database ────────────────────────────────────────────
    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // ── S3 ──────────────────────────────────────────────────
    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(true)
        .build();
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
    let image_storage = Arc::new(S3ImageStorage::new(
        s3_client,
        config.s3_internal_url.clone(),
        config.s3_public_url.clone(),
    ));

    // ── Services ────────────────────────────────────────────
    let product_service = web::Data::new(ProductService::new(
        Arc::new(PgProductRepo::new(pool.clone())),
        image_storage.clone(),
        config.s3_images_bucket.clone(),
        config.s3_preview_bucket.clone(),
    ));
    let brand_service = web::Data::new(BrandService::new(Arc::new(PgBrandRepo::new(
        pool.clone(),
    ))));
    let category_service = web::Data::new(CategoryService::new(Arc::new(PgCategoryRepo::new(
        pool.clone(),
    ))));
    let tag_service = web::Data::new(TagService::new(Arc::new(PgTagRepo::new(pool.clone()))));

    // ── HTTP Server ─────────────────────────────────────────
    info!("Starting PRODUCT SERVICE on port {}", config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(product_service.clone())
            .app_data(brand_service.clone())
            .app_data(category_service.clone())
            .app_data(tag_service.clone())
            .service(
                web::scope("/api")
                    .configure(handler::product_handler::configure)
                    .configure(handler::brand_handler::configure)
                    .configure(handler::category_handler::configure)
                    .configure(handler::tag_handler::configure),
            )
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

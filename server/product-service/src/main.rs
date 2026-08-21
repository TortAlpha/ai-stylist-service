use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use product_service::config::{Config, ServiceMode};
use product_service::embeddings::{CohereEmbedder, MultimodalEmbedder};
use product_service::handler;
use product_service::jobs;
use product_service::jobs::{EmbedWorkerDeps, reindex_images, text_embedder::TextEmbedderWorker};
use product_service::openapi::ApiDoc;
use product_service::repo::impls::{
    brand_repo::PgBrandRepo, category_repo::PgCategoryRepo, product_photo_repo::PgProductPhotoRepo,
    product_repo::PgProductRepo, purchase_location_repo::PgPurchaseLocationRepo,
    tag_repo::PgTagRepo,
};
use product_service::service::{
    brand_service::BrandService, category_service::CategoryService, product::ProductService,
    product_photo_service::ProductPhotoService, purchase_location_service::PurchaseLocationService,
    tag_service::TagService,
};
use product_service::storage::s3::S3ImageStorage;
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

    // ── Database ────────────────────────────────────────────
    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations/runtime")
        .run(&pool)
        .await
        .expect("Failed to apply database migrations");
    info!("Database migrations applied");

    match config.mode {
        ServiceMode::TextEmbedder => run_text_embedder(config, pool).await,
        ServiceMode::ReindexImages => run_reindex_images(pool).await,
        ServiceMode::Server => run_http_server(config, pool).await,
    }
}

async fn run_reindex_images(pool: sqlx::PgPool) -> std::io::Result<()> {
    info!("Starting product-service in reindex-images mode");
    reindex_images::run(pool)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    info!("reindex_images: done — exiting");
    Ok(())
}

async fn run_text_embedder(config: Config, pool: sqlx::PgPool) -> std::io::Result<()> {
    let embedder = match CohereEmbedder::new(
        config.embed.api_key.clone(),
        config.embed.base_url.clone(),
        config.embed.model.clone(),
        config.embed.dim,
        config.embed.timeout,
        config.embed.image_throttle,
    ) {
        Ok(e) => Arc::new(e),
        Err(e) => {
            error!(error = %e, "text-embedder: failed to init Cohere client");
            // Surface as a non-zero exit so the orchestrator can restart.
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    };
    let worker = TextEmbedderWorker::new(pool, embedder, config.text_embedder.clone());
    info!("Starting product-service in text-embedder mode");
    worker.run().await;
    Ok(())
}

async fn run_http_server(config: Config, pool: sqlx::PgPool) -> std::io::Result<()> {
    // ── S3 ──────────────────────────────────────────────────
    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_internal_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .endpoint_url(config.s3_internal_url.clone())
        .force_path_style(true)
        .build();
    let s3_public_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .endpoint_url(config.s3_public_url.clone())
        .force_path_style(true)
        .build();

    let s3_client = aws_sdk_s3::Client::from_conf(s3_internal_config);
    let s3_presign_client = aws_sdk_s3::Client::from_conf(s3_public_config);
    let image_storage = Arc::new(S3ImageStorage::new(
        s3_client,
        s3_presign_client,
        config.s3_public_url.clone(),
    ));

    // ── Services ────────────────────────────────────────────
    let category_repo: Arc<PgCategoryRepo> = Arc::new(PgCategoryRepo::new(pool.clone()));
    let product_repo: Arc<PgProductRepo> = Arc::new(PgProductRepo::new(pool.clone()));
    let product_photo_repo: Arc<PgProductPhotoRepo> =
        Arc::new(PgProductPhotoRepo::new(pool.clone()));

    let photo_service = Arc::new(ProductPhotoService::new(
        product_photo_repo,
        image_storage.clone(),
        config.s3_images_bucket.clone(),
        config.s3_preview_bucket.clone(),
    ));

    let product_service = web::Data::new(ProductService::new(
        pool.clone(),
        product_repo,
        category_repo.clone(),
        photo_service.clone(),
    ));

    // ── Background workers ──────────────────────────────────
    // If MULTIMODAL_EMBED_API_KEY is set, attach embedder deps so the
    // shared queue can dispatch `embed_product_images` jobs. Otherwise
    // start the worker without embedding support — those jobs will fail
    // until an operator configures the key.
    let embed_deps: Option<EmbedWorkerDeps> = if config.embed.api_key.is_empty() {
        info!("MULTIMODAL_EMBED_API_KEY not set — embed_product_images jobs will fail until configured");
        None
    } else {
        match CohereEmbedder::new(
            config.embed.api_key.clone(),
            config.embed.base_url.clone(),
            config.embed.model.clone(),
            config.embed.dim,
            config.embed.timeout,
            config.embed.image_throttle,
        ) {
            Ok(e) => {
                let embedder: Arc<dyn MultimodalEmbedder> = Arc::new(e);
                Some(EmbedWorkerDeps {
                    storage: image_storage.clone(),
                    images_bucket: config.s3_images_bucket.clone(),
                    embedder,
                })
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to init image embedder; embed jobs disabled");
                None
            }
        }
    };
    jobs::spawn_worker(pool.clone(), photo_service.clone(), embed_deps);

    let photo_service_data = web::Data::from(photo_service);
    let brand_service = web::Data::new(BrandService::new(Arc::new(PgBrandRepo::new(pool.clone()))));
    let category_service = web::Data::new(CategoryService::new(category_repo));
    let tag_service = web::Data::new(TagService::new(Arc::new(PgTagRepo::new(pool.clone()))));
    let purchase_location_service = web::Data::new(PurchaseLocationService::new(Arc::new(
        PgPurchaseLocationRepo::new(pool.clone()),
    )));

    let config_data = web::Data::new(config.clone());
    let pool_data = web::Data::new(pool);

    // ── HTTP Server ─────────────────────────────────────────
    info!("Starting PRODUCT SERVICE on port {}", config.port);

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(product_service.clone())
            .app_data(photo_service_data.clone())
            .app_data(brand_service.clone())
            .app_data(category_service.clone())
            .app_data(tag_service.clone())
            .app_data(purchase_location_service.clone())
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(
                web::scope("/api")
                    .configure(handler::product_handler::configure)
                    .configure(handler::brand_handler::configure)
                    .configure(handler::category_handler::configure)
                    .configure(handler::tag_handler::configure)
                    .configure(handler::purchase_location_handler::configure)
                    .configure(handler::internal_handler::configure),
            )
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

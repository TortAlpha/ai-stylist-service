//! Internal (service-to-service) endpoints.
//!
//! Routed under `/api/internal/*`. Protected by `X-Internal-Token`
//! (see `extractors::internal::InternalAuth`). Nginx must not forward
//! `/api/internal/` to the outside world.

use actix_web::{HttpResponse, web};
use pgvector::Vector;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::config::Config;
use crate::domain::error::ServiceError;
use crate::extractors::internal::InternalAuth;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/internal").service(
            web::scope("/products")
                .route("/semantic-search", web::post().to(semantic_search)),
        ),
    );
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct SemanticSearchFilters {
    pub category: Option<String>,
    pub brand: Option<String>,
    pub tags: Vec<String>,
    pub size: Option<String>,
    pub condition: Option<String>,
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SemanticSearchRequest {
    pub vector: Vec<f32>,
    #[serde(default = "default_top_k")]
    pub top_k: i64,
    #[serde(default)]
    pub filters: SemanticSearchFilters,
}

fn default_top_k() -> i64 {
    10
}

#[derive(Debug, Serialize)]
pub struct ScoreBreakdown {
    pub text: f32,
    pub image: f32,
    pub best_image_idx: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct SemanticSearchItem {
    pub id: Uuid,
    pub score: f32,
    pub score_breakdown: ScoreBreakdown,
    pub name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub price: Option<i64>,
    pub highlight_fields: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SemanticSearchResponse {
    pub items: Vec<SemanticSearchItem>,
}

pub async fn semantic_search(
    _auth: InternalAuth,
    cfg: web::Data<Config>,
    pool: web::Data<PgPool>,
    body: web::Json<SemanticSearchRequest>,
) -> Result<HttpResponse, ServiceError> {
    let req = body.into_inner();
    if req.vector.len() != cfg.embed.dim {
        return Err(ServiceError::BadRequest(format!(
            "invalid_vector_dim: got {}, expected {}",
            req.vector.len(),
            cfg.embed.dim
        )));
    }
    if req.top_k <= 0 || req.top_k > 100 {
        return Err(ServiceError::BadRequest(
            "top_k must be in (0, 100]".into(),
        ));
    }

    let vec = Vector::from(req.vector);
    let w_text = cfg.hybrid.text_weight;
    let w_image = cfg.hybrid.image_weight;

    // Hybrid scoring (per_image mode):
    // - text_scores: one row per product with its (1 - cos_dist) to the query.
    // - image_scores: for each product, the single best image — DISTINCT ON
    //   gives both the MAX cosine and the matching image_idx in one pass.
    // COALESCE keeps products that only have one side (just text or just
    // images) in the result set instead of dropping them entirely.
    let rows = sqlx::query_as::<_, SemanticRow>(
        r#"
        WITH text_scores AS (
            SELECT product_id,
                   1 - (embedding <=> $1::vector) AS s
            FROM product_text_embeddings
        ),
        image_scores AS (
            SELECT DISTINCT ON (product_id)
                   product_id,
                   image_idx                                  AS best_idx,
                   1 - (embedding <=> $1::vector)             AS s
            FROM product_image_embeddings
            ORDER BY product_id, 1 - (embedding <=> $1::vector) DESC
        )
        SELECT
            p.id                                                 AS id,
            p.name                                               AS name,
            b.name                                               AS brand,
            c.name                                               AS category,
            p.purchase_price                                     AS price,
            COALESCE(t.s, 0)::real                               AS text_score,
            COALESCE(i.s, 0)::real                               AS image_score,
            i.best_idx                                           AS best_image_idx,
            ($2::real * COALESCE(t.s, 0) + $8::real * COALESCE(i.s, 0))::real AS score
        FROM product p
        LEFT JOIN text_scores  t ON t.product_id = p.id
        LEFT JOIN image_scores i ON i.product_id = p.id
        LEFT JOIN brand        b ON p.brand_id    = b.id
        LEFT JOIN category     c ON p.category_id = c.id
        WHERE p.status = 'ready' AND p.is_deleted = false
          AND ($4::text IS NULL OR c.name = $4)
          AND ($5::text IS NULL OR b.name = $5)
          AND ($6::bigint IS NULL OR (p.purchase_price IS NOT NULL AND p.purchase_price >= $6::numeric))
          AND ($7::bigint IS NULL OR (p.purchase_price IS NOT NULL AND p.purchase_price <= $7::numeric))
        ORDER BY score DESC
        LIMIT $3
        "#,
    )
    .bind(vec)
    .bind(w_text)
    .bind(req.top_k)
    .bind(req.filters.category)
    .bind(req.filters.brand)
    .bind(req.filters.price_min)
    .bind(req.filters.price_max)
    .bind(w_image)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        warn!(error = %e, "semantic_search: sql failed");
        ServiceError::Internal(format!("semantic-search sql: {e}"))
    })?;

    debug!(results = rows.len(), w_image = w_image, "semantic_search.results");
    let items = rows
        .into_iter()
        .map(|r| SemanticSearchItem {
            id: r.id,
            score: r.score,
            score_breakdown: ScoreBreakdown {
                text: r.text_score,
                image: r.image_score,
                best_image_idx: r.best_image_idx,
            },
            name: r.name,
            brand: r.brand,
            category: r.category,
            price: r.price.and_then(|d: Decimal| d.to_i64()),
            highlight_fields: vec![],
        })
        .collect();

    Ok(HttpResponse::Ok().json(SemanticSearchResponse { items }))
}

#[derive(sqlx::FromRow)]
struct SemanticRow {
    id: Uuid,
    name: String,
    brand: Option<String>,
    category: Option<String>,
    price: Option<Decimal>,
    text_score: f32,
    image_score: f32,
    best_image_idx: Option<i32>,
    score: f32,
}

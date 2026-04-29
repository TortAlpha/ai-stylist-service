use async_trait::async_trait;
use sqlx::{Arguments, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::product::ProductFull;
use crate::domain::product_details::TypeDetailsInput;
use crate::domain::request_dto::product::{CreateProductRequest, UpdateProductRequest};
use crate::domain::request_dto::product_details::UpdateProductDetailsRequest;
use crate::domain::response_dto::product::ProductFilterOptions;
use crate::domain::response_dto::product_details::AvailableSizesResponse;
use crate::domain::utils::csv::csv_values;
use crate::domain::utils::pagination::PaginationParams;
use crate::domain::utils::query::{AvailableSizesQuery, FilterOptionsQuery, ProductListQuery};
use crate::jobs;
use crate::repo::traits::product_repo::ProductRepository;

use super::product_query_builder::{build_filters, build_order_by};

pub struct PgProductRepo {
    pool: PgPool,
}

impl PgProductRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn fetch_full(
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
    ) -> Result<ProductFull, sqlx::Error> {
        sqlx::query_as::<_, ProductFull>("SELECT * FROM v_product_full WHERE id = $1")
            .bind(id)
            .fetch_one(tx.as_mut())
            .await
    }

    async fn insert_tags(
        tx: &mut Transaction<'_, Postgres>,
        product_id: Uuid,
        style_tag_ids: &[i32],
        vibe_tag_ids: &[i32],
        season_ids: &[i32],
    ) -> Result<(), sqlx::Error> {
        if !style_tag_ids.is_empty() {
            sqlx::query(
                "INSERT INTO product_style_tag (product_id, style_tag_id)
                 SELECT $1, UNNEST($2::int[])",
            )
            .bind(product_id)
            .bind(style_tag_ids)
            .execute(tx.as_mut())
            .await?;
        }

        if !vibe_tag_ids.is_empty() {
            sqlx::query(
                "INSERT INTO product_vibe_tag (product_id, vibe_tag_id)
                 SELECT $1, UNNEST($2::int[])",
            )
            .bind(product_id)
            .bind(vibe_tag_ids)
            .execute(tx.as_mut())
            .await?;
        }

        if !season_ids.is_empty() {
            sqlx::query(
                "INSERT INTO product_season (product_id, season_id)
                 SELECT $1, UNNEST($2::int[])",
            )
            .bind(product_id)
            .bind(season_ids)
            .execute(tx.as_mut())
            .await?;
        }

        Ok(())
    }

    async fn replace_tags(
        tx: &mut Transaction<'_, Postgres>,
        product_id: Uuid,
        style_tag_ids: Option<&Vec<i32>>,
        vibe_tag_ids: Option<&Vec<i32>>,
        season_ids: Option<&Vec<i32>>,
    ) -> Result<(), sqlx::Error> {
        if let Some(ids) = style_tag_ids {
            sqlx::query("DELETE FROM product_style_tag WHERE product_id = $1")
                .bind(product_id)
                .execute(tx.as_mut())
                .await?;
            if !ids.is_empty() {
                sqlx::query(
                    "INSERT INTO product_style_tag (product_id, style_tag_id)
                     SELECT $1, UNNEST($2::int[])",
                )
                .bind(product_id)
                .bind(ids.as_slice())
                .execute(tx.as_mut())
                .await?;
            }
        }

        if let Some(ids) = vibe_tag_ids {
            sqlx::query("DELETE FROM product_vibe_tag WHERE product_id = $1")
                .bind(product_id)
                .execute(tx.as_mut())
                .await?;
            if !ids.is_empty() {
                sqlx::query(
                    "INSERT INTO product_vibe_tag (product_id, vibe_tag_id)
                     SELECT $1, UNNEST($2::int[])",
                )
                .bind(product_id)
                .bind(ids.as_slice())
                .execute(tx.as_mut())
                .await?;
            }
        }

        if let Some(ids) = season_ids {
            sqlx::query("DELETE FROM product_season WHERE product_id = $1")
                .bind(product_id)
                .execute(tx.as_mut())
                .await?;
            if !ids.is_empty() {
                sqlx::query(
                    "INSERT INTO product_season (product_id, season_id)
                     SELECT $1, UNNEST($2::int[])",
                )
                .bind(product_id)
                .bind(ids.as_slice())
                .execute(tx.as_mut())
                .await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ProductRepository for PgProductRepo {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ProductFull>, sqlx::Error> {
        sqlx::query_as::<_, ProductFull>("SELECT * FROM v_product_full WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn search_by_query(
        &self,
        query: &ProductListQuery,
    ) -> Result<(Vec<ProductFull>, i64), sqlx::Error> {
        let pagination = PaginationParams {
            page: query.page,
            per_page: query.per_page,
        };
        let (page, per_page, offset) = pagination.resolve();
        let _ = page; // used only for response construction in service

        // Build filters for count query
        let count_filters = build_filters(query);
        let count_sql = format!(
            "SELECT COUNT(*) as count FROM v_product_full {}",
            count_filters.where_clause
        );

        let total: i64 = sqlx::query_scalar_with::<_, i64, _>(&count_sql, count_filters.args)
            .fetch_one(&self.pool)
            .await?;

        if total == 0 {
            return Ok((vec![], 0));
        }

        // Build filters for data query
        let mut data_filters = build_filters(query);
        let order_by = build_order_by(query);

        let next_idx = data_filters.param_count + 1;
        let data_sql = format!(
            "SELECT * FROM v_product_full {} {} LIMIT ${} OFFSET ${}",
            data_filters.where_clause,
            order_by,
            next_idx,
            next_idx + 1,
        );

        let _ = data_filters.args.add(per_page);
        let _ = data_filters.args.add(offset);

        let items = sqlx::query_as_with::<_, ProductFull, _>(&data_sql, data_filters.args)
            .fetch_all(&self.pool)
            .await?;

        Ok((items, total))
    }

    async fn create(&self, req: &CreateProductRequest) -> Result<ProductFull, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let product = self.create_in_tx(&mut tx, req).await?;
        tx.commit().await?;
        Ok(product)
    }

    async fn create_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        req: &CreateProductRequest,
    ) -> Result<ProductFull, sqlx::Error> {
        // 1. Insert product row
        let status_str = req.status.as_ref().map(|s| s.as_str()).unwrap_or("intake");
        let currency = req.currency.as_deref().unwrap_or("RSD");

        let product_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO product (
                name, brand_id, category_id, purchase_location_id,
                status, purchase_price, currency, ai_notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
        .bind(&req.name)
        .bind(req.brand_id)
        .bind(req.category_id)
        .bind(req.purchase_location_id)
        .bind(status_str)
        .bind(req.purchase_price)
        .bind(currency)
        .bind(&req.ai_notes)
        .fetch_one(tx.as_mut())
        .await?;

        // 2. Insert product_details
        let d = &req.details;
        let condition_str = d.condition.as_str();
        let is_vintage = d.is_vintage.unwrap_or(false);
        let is_collab = d.is_collab.unwrap_or(false);
        let is_limited_edition = d.is_limited_edition.unwrap_or(false);
        let size_system_str = d.size.size_system.as_ref().map(|s| s.as_str());

        // Extract type-specific fields
        let (
            fit,
            shoe_width,
            insole_length_cm,
            width_cm,
            height_cm,
            depth_cm,
            handle_type,
            bag_size_label,
            metal,
            stone,
            clasp_type,
        ) = extract_type_details(&d.type_details);

        sqlx::query(
            r#"
            INSERT INTO product_details (
                product_id, material, condition, color, year_of_release,
                is_vintage, is_collab, collab_name, is_limited_edition, special_notes,
                size_value, size_value2, size_system, measurement_cm,
                fit, shoe_width, insole_length_cm,
                width_cm, height_cm, depth_cm, handle_type, bag_size_label,
                metal, stone, clasp_type
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14,
                $15, $16, $17,
                $18, $19, $20, $21, $22,
                $23, $24, $25
            )
            "#,
        )
        .bind(product_id)
        .bind(&d.material)
        .bind(condition_str)
        .bind(&d.color)
        .bind(d.year_of_release)
        .bind(is_vintage)
        .bind(is_collab)
        .bind(&d.collab_name)
        .bind(is_limited_edition)
        .bind(&d.special_notes)
        .bind(&d.size.size_value)
        .bind(&d.size.size_value2)
        .bind(size_system_str)
        .bind(d.size.measurement_cm)
        .bind(fit)
        .bind(shoe_width)
        .bind(insole_length_cm)
        .bind(width_cm)
        .bind(height_cm)
        .bind(depth_cm)
        .bind(handle_type)
        .bind(bag_size_label)
        .bind(metal)
        .bind(stone)
        .bind(clasp_type)
        .execute(tx.as_mut())
        .await?;

        // 3. Insert tags
        Self::insert_tags(
            tx,
            product_id,
            &req.style_tag_ids,
            &req.vibe_tag_ids,
            &req.season_ids,
        )
        .await?;

        // 4. Fetch full product
        let product = Self::fetch_full(tx, product_id).await?;

        Ok(product)
    }

    async fn update(
        &self,
        id: Uuid,
        req: &UpdateProductRequest,
    ) -> Result<ProductFull, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let product = self.update_in_tx(&mut tx, id, req).await?;
        tx.commit().await?;
        Ok(product)
    }

    async fn update_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
        req: &UpdateProductRequest,
    ) -> Result<ProductFull, sqlx::Error> {
        // 1. Update product row with optimistic locking
        let status_str = req.status.as_ref().map(|s| s.as_str());
        let result = sqlx::query(
            r#"
            UPDATE product SET
                name                 = COALESCE($2, name),
                brand_id             = COALESCE($3, brand_id),
                category_id          = COALESCE($4, category_id),
                purchase_location_id = COALESCE($5, purchase_location_id),
                status               = COALESCE($6, status),
                purchase_price       = COALESCE($7, purchase_price),
                currency             = COALESCE($8, currency),
                ai_notes             = COALESCE($9, ai_notes)
            WHERE id = $1 AND version = $10
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(req.brand_id)
        .bind(req.category_id)
        .bind(req.purchase_location_id)
        .bind(status_str)
        .bind(req.purchase_price)
        .bind(&req.currency)
        .bind(&req.ai_notes)
        .bind(req.expected_version)
        .execute(tx.as_mut())
        .await?;

        if result.rows_affected() == 0 {
            // Check if product exists to distinguish NotFound vs StaleVersion
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM product WHERE id = $1 AND is_deleted = false)",
            )
            .bind(id)
            .fetch_one(tx.as_mut())
            .await?;

            if exists {
                return Err(sqlx::Error::Protocol("STALE_VERSION".to_string()));
            } else {
                return Err(sqlx::Error::RowNotFound);
            }
        }

        // 2. Update product_details if provided
        if let Some(d) = &req.details {
            update_details(tx, id, d).await?;
        }

        // 3. Replace tags if provided
        Self::replace_tags(
            tx,
            id,
            req.style_tag_ids.as_ref(),
            req.vibe_tag_ids.as_ref(),
            req.season_ids.as_ref(),
        )
        .await?;

        // 4. Fetch full product
        let product = Self::fetch_full(tx, id).await?;

        Ok(product)
    }

    async fn soft_delete(&self, id: Uuid, expected_version: i32) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            r#"
            UPDATE product
            SET is_deleted = true, deleted_at = now()
            WHERE id = $1 AND version = $2 AND is_deleted = false
            "#,
        )
        .bind(id)
        .bind(expected_version)
        .execute(tx.as_mut())
        .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            jobs::enqueue_delete_product_images(&mut tx, id).await?;
        }

        tx.commit().await?;
        Ok(deleted)
    }

    async fn filter_options(
        &self,
        query: &FilterOptionsQuery,
    ) -> Result<ProductFilterOptions, sqlx::Error> {
        // Build WHERE dynamically based on provided filters
        let mut conditions = vec!["p.is_deleted = false".to_string()];
        let mut bind_idx = 1u32;

        // We'll build the query string and use raw query with manual binds
        macro_rules! push_filter {
            ($field:expr, $column:expr) => {
                if $field.is_some() {
                    conditions.push(format!("{} = ${}", $column, bind_idx));
                    bind_idx += 1;
                }
            };
        }

        push_filter!(query.brand_id, "p.brand_id");
        push_filter!(query.category_id, "p.category_id");
        push_filter!(query.product_type, "c.product_type");
        push_filter!(query.gender, "c.gender");
        push_filter!(query.status, "p.status");
        push_filter!(query.condition, "pd.condition");
        push_filter!(query.color, "pd.color");

        if query.price_min.is_some() {
            conditions.push(format!("p.purchase_price >= ${}", bind_idx));
            bind_idx += 1;
        }
        if query.price_max.is_some() {
            conditions.push(format!("p.purchase_price <= ${}", bind_idx));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT
                COALESCE(array_agg(DISTINCT pd.color) FILTER (WHERE pd.color IS NOT NULL), '{{}}') AS colors,
                COALESCE(array_agg(DISTINCT pd.condition) FILTER (WHERE pd.condition IS NOT NULL), '{{}}') AS conditions,
                COALESCE(array_agg(DISTINCT pd.material) FILTER (WHERE pd.material IS NOT NULL), '{{}}') AS materials,
                MIN(p.purchase_price) AS price_min,
                MAX(p.purchase_price) AS price_max,
                COALESCE(array_agg(DISTINCT p.brand_id), '{{}}') AS brand_ids,
                COALESCE(array_agg(DISTINCT p.category_id), '{{}}') AS category_ids
            FROM product p
            JOIN product_details pd ON pd.product_id = p.id
            JOIN category c ON c.id = p.category_id
            WHERE {where_clause}
            "#
        );

        let mut q = sqlx::query_as::<
            _,
            (
                Vec<String>,                   // colors
                Vec<String>,                   // conditions
                Vec<String>,                   // materials
                Option<rust_decimal::Decimal>, // price_min
                Option<rust_decimal::Decimal>, // price_max
                Vec<i32>,                      // brand_ids
                Vec<i32>,                      // category_ids
            ),
        >(&sql);

        if let Some(v) = query.brand_id {
            q = q.bind(v);
        }
        if let Some(ref v) = query.category_id {
            q = q.bind(*v);
        }
        if let Some(ref v) = query.product_type {
            q = q.bind(v);
        }
        if let Some(ref v) = query.gender {
            q = q.bind(v);
        }
        if let Some(ref v) = query.status {
            q = q.bind(v);
        }
        if let Some(ref v) = query.condition {
            q = q.bind(v);
        }
        if let Some(ref v) = query.color {
            q = q.bind(v);
        }
        if let Some(v) = query.price_min {
            q = q.bind(v);
        }
        if let Some(v) = query.price_max {
            q = q.bind(v);
        }

        let row = q.fetch_one(&self.pool).await?;

        Ok(ProductFilterOptions {
            colors: row.0,
            conditions: row.1,
            materials: row.2,
            price_min: row.3,
            price_max: row.4,
            brand_ids: row.5,
            category_ids: row.6,
        })
    }

    async fn available_sizes(
        &self,
        query: &AvailableSizesQuery,
    ) -> Result<AvailableSizesResponse, sqlx::Error> {
        // First resolve size_group from category or directly
        let size_group: String = if let Some(cid) = query.category_id {
            sqlx::query_scalar("SELECT size_group FROM category WHERE id = $1")
                .bind(cid)
                .fetch_optional(&self.pool)
                .await?
                .unwrap_or_else(|| "one_size".to_string())
        } else if let Some(ref sg) = query.size_group {
            sg.clone()
        } else {
            return Ok(AvailableSizesResponse {
                size_group: "unknown".to_string(),
                values: vec![],
                values2: vec![],
                systems: vec![],
                widths: vec![],
            });
        };

        // Build WHERE
        let mut conditions = vec![
            "p.is_deleted = false".to_string(),
            format!("c.size_group = $1"),
        ];
        let mut bind_idx = 2u32;

        if query.category_id.is_some() {
            conditions.push(format!("p.category_id = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.brand_id.is_some() {
            conditions.push(format!("p.brand_id = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.gender.is_some() {
            conditions.push(format!("c.gender = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.status.is_some() {
            conditions.push(format!("p.status = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.condition.is_some() {
            conditions.push(format!("pd.condition = ${bind_idx}"));
            bind_idx += 1;
        }
        if query.color.is_some() {
            conditions.push(format!("pd.color = ${bind_idx}"));
            bind_idx += 1;
        }
        let size_systems = query
            .size_systems
            .as_deref()
            .map(csv_values)
            .unwrap_or_default();
        if query.size_system.is_some() {
            conditions.push(format!("pd.size_system = ${bind_idx}"));
            bind_idx += 1;
        }
        if !size_systems.is_empty() {
            conditions.push(format!("pd.size_system = ANY(${bind_idx})"));
            bind_idx += 1;
        }
        if query.price_min.is_some() {
            conditions.push(format!("p.purchase_price >= ${bind_idx}"));
            bind_idx += 1;
        }
        if query.price_max.is_some() {
            conditions.push(format!("p.purchase_price <= ${bind_idx}"));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT
                COALESCE(array_agg(DISTINCT pd.size_value) FILTER (WHERE pd.size_value IS NOT NULL), '{{}}'),
                COALESCE(array_agg(DISTINCT pd.size_value2) FILTER (WHERE pd.size_value2 IS NOT NULL), '{{}}'),
                COALESCE(array_agg(DISTINCT pd.size_system) FILTER (WHERE pd.size_system IS NOT NULL), '{{}}'),
                COALESCE(array_agg(DISTINCT pd.shoe_width) FILTER (WHERE pd.shoe_width IS NOT NULL), '{{}}')
            FROM product p
            JOIN product_details pd ON pd.product_id = p.id
            JOIN category c ON c.id = p.category_id
            WHERE {where_clause}
            "#
        );

        let mut q = sqlx::query_as::<_, (Vec<String>, Vec<String>, Vec<String>, Vec<String>)>(&sql);
        q = q.bind(&size_group);
        if let Some(cid) = query.category_id {
            q = q.bind(cid);
        }
        if let Some(bid) = query.brand_id {
            q = q.bind(bid);
        }
        if let Some(ref g) = query.gender {
            q = q.bind(g);
        }
        if let Some(ref status) = query.status {
            q = q.bind(status);
        }
        if let Some(ref condition) = query.condition {
            q = q.bind(condition);
        }
        if let Some(ref color) = query.color {
            q = q.bind(color);
        }
        if let Some(ref size_system) = query.size_system {
            q = q.bind(size_system);
        }
        if !size_systems.is_empty() {
            q = q.bind(size_systems);
        }
        if let Some(price_min) = query.price_min {
            q = q.bind(price_min);
        }
        if let Some(price_max) = query.price_max {
            q = q.bind(price_max);
        }

        let row = q.fetch_one(&self.pool).await?;

        Ok(AvailableSizesResponse {
            size_group,
            values: row.0,
            values2: row.1,
            systems: row.2,
            widths: row.3,
        })
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

type OptStr<'a> = Option<&'a str>;
type OptDec = Option<rust_decimal::Decimal>;

fn extract_type_details(
    input: &TypeDetailsInput,
) -> (
    OptStr<'_>,
    OptStr<'_>,
    OptDec,
    OptDec,
    OptDec,
    OptDec,
    OptStr<'_>,
    Option<&str>,
    Option<&str>,
    Option<&str>,
    Option<&str>,
) {
    match input {
        TypeDetailsInput::Clothing { fit } => (
            fit.as_ref().map(|f| f.as_str()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        TypeDetailsInput::Footwear {
            shoe_width,
            insole_length_cm,
        } => (
            None,
            shoe_width.as_ref().map(|w| w.as_str()),
            *insole_length_cm,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        TypeDetailsInput::Bags {
            width_cm,
            height_cm,
            depth_cm,
            handle_type,
            bag_size_label,
        } => (
            None,
            None,
            None,
            *width_cm,
            *height_cm,
            *depth_cm,
            handle_type.as_ref().map(|h| h.as_str()),
            bag_size_label.as_deref(),
            None,
            None,
            None,
        ),
        TypeDetailsInput::Jewelry {
            metal,
            stone,
            clasp_type,
        } => (
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            metal.as_deref(),
            stone.as_deref(),
            clasp_type.as_deref(),
        ),
        TypeDetailsInput::Accessories => (
            None, None, None, None, None, None, None, None, None, None, None,
        ),
    }
}

async fn update_details(
    tx: &mut Transaction<'_, Postgres>,
    product_id: Uuid,
    d: &UpdateProductDetailsRequest,
) -> Result<(), sqlx::Error> {
    let condition_str = d.condition.as_ref().map(|c| c.as_str());
    let size_system_str = d
        .size
        .as_ref()
        .and_then(|s| s.size_system.as_ref().map(|ss| ss.as_str()));

    // Type-specific: extract from type_details if present, else all None
    let (
        fit,
        shoe_width,
        insole_length_cm,
        width_cm,
        height_cm,
        depth_cm,
        handle_type,
        bag_size_label,
        metal,
        stone,
        clasp_type,
    ) = d
        .type_details
        .as_ref()
        .map(extract_type_details)
        .unwrap_or((
            None, None, None, None, None, None, None, None, None, None, None,
        ));

    sqlx::query(
        r#"
        UPDATE product_details SET
            material           = COALESCE($2, material),
            condition          = COALESCE($3, condition),
            color              = COALESCE($4, color),
            year_of_release    = COALESCE($5, year_of_release),
            is_vintage         = COALESCE($6, is_vintage),
            is_collab          = COALESCE($7, is_collab),
            collab_name        = COALESCE($8, collab_name),
            is_limited_edition = COALESCE($9, is_limited_edition),
            special_notes      = COALESCE($10, special_notes),
            size_value         = COALESCE($11, size_value),
            size_value2        = COALESCE($12, size_value2),
            size_system        = COALESCE($13, size_system),
            measurement_cm     = COALESCE($14, measurement_cm),
            fit                = COALESCE($15, fit),
            shoe_width         = COALESCE($16, shoe_width),
            insole_length_cm   = COALESCE($17, insole_length_cm),
            width_cm           = COALESCE($18, width_cm),
            height_cm          = COALESCE($19, height_cm),
            depth_cm           = COALESCE($20, depth_cm),
            handle_type        = COALESCE($21, handle_type),
            bag_size_label     = COALESCE($22, bag_size_label),
            metal              = COALESCE($23, metal),
            stone              = COALESCE($24, stone),
            clasp_type         = COALESCE($25, clasp_type)
        WHERE product_id = $1
        "#,
    )
    .bind(product_id)
    .bind(&d.material)
    .bind(condition_str)
    .bind(&d.color)
    .bind(d.year_of_release)
    .bind(d.is_vintage)
    .bind(d.is_collab)
    .bind(&d.collab_name)
    .bind(d.is_limited_edition)
    .bind(&d.special_notes)
    .bind(d.size.as_ref().and_then(|s| s.size_value.as_deref()))
    .bind(d.size.as_ref().and_then(|s| s.size_value2.as_deref()))
    .bind(size_system_str)
    .bind(d.size.as_ref().and_then(|s| s.measurement_cm))
    .bind(fit)
    .bind(shoe_width)
    .bind(insole_length_cm)
    .bind(width_cm)
    .bind(height_cm)
    .bind(depth_cm)
    .bind(handle_type)
    .bind(bag_size_label)
    .bind(metal)
    .bind(stone)
    .bind(clasp_type)
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

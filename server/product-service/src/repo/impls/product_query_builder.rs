use sqlx::Arguments;
use sqlx::postgres::PgArguments;

use crate::domain::utils::query::{ProductListQuery, ProductSortField, SortOrder};

pub struct FilterResult {
    pub where_clause: String,
    pub args: PgArguments,
    pub param_count: usize,
}

pub fn build_filters(query: &ProductListQuery) -> FilterResult {
    let mut where_clause = String::from("WHERE 1=1");
    let mut args = PgArguments::default();
    let mut idx = 0usize;

    macro_rules! add_filter {
        ($field:expr, $column:expr) => {
            if let Some(val) = &$field {
                idx += 1;
                where_clause.push_str(&format!(" AND {} = ${}", $column, idx));
                let _ = args.add(val);
            }
        };
    }

    add_filter!(query.brand_id, "brand_id");
    add_filter!(query.category_id, "category_id");
    add_filter!(query.product_type, "\"type\"");
    add_filter!(query.status, "status");
    add_filter!(query.gender, "gender");
    add_filter!(query.condition, "condition");
    add_filter!(query.size_value, "size_value");
    add_filter!(query.size_value2, "size_value2");
    add_filter!(query.size_system, "size_system");
    add_filter!(query.size_group, "size_group");
    if let Some(search) = &query.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            idx += 1;
            where_clause.push_str(&format!(" AND (name ILIKE ${0} OR sku ILIKE ${0})", idx));
            let pattern = format!("%{trimmed}%");
            let _ = args.add(pattern);
        }
    }

    if let Some(min) = &query.price_min {
        idx += 1;
        where_clause.push_str(&format!(" AND purchase_price >= ${idx}"));
        let _ = args.add(min);
    }

    if let Some(max) = &query.price_max {
        idx += 1;
        where_clause.push_str(&format!(" AND purchase_price <= ${idx}"));
        let _ = args.add(max);
    }

    FilterResult {
        where_clause,
        args,
        param_count: idx,
    }
}

pub fn build_order_by(query: &ProductListQuery) -> String {
    let column = match &query.sort_by {
        Some(ProductSortField::Price) => "purchase_price",
        Some(ProductSortField::Name) => "name",
        Some(ProductSortField::UpdatedAt) => "updated_at",
        Some(ProductSortField::CreatedAt) | None => "created_at",
    };

    let direction = match &query.sort_order {
        Some(SortOrder::Asc) => "ASC",
        Some(SortOrder::Desc) | None => "DESC",
    };

    format!("ORDER BY {column} {direction}")
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    fn empty_query() -> ProductListQuery {
        ProductListQuery {
            page: None,
            per_page: None,
            brand_id: None,
            category_id: None,
            product_type: None,
            status: None,
            gender: None,
            price_min: None,
            price_max: None,
            condition: None,
            size_value: None,
            size_value2: None,
            size_system: None,
            size_group: None,
            sort_by: None,
            sort_order: None,
            search: None,
        }
    }

    #[test]
    fn search_filter_targets_name_and_sku_with_single_placeholder() {
        let mut q = empty_query();
        q.search = Some("ava-001".to_string());

        let filters = build_filters(&q);

        assert_eq!(
            filters.where_clause,
            "WHERE 1=1 AND (name ILIKE $1 OR sku ILIKE $1)"
        );
        assert_eq!(filters.param_count, 1);
    }

    #[test]
    fn blank_search_is_ignored() {
        let mut q = empty_query();
        q.search = Some("   ".to_string());

        let filters = build_filters(&q);

        assert_eq!(filters.where_clause, "WHERE 1=1");
        assert_eq!(filters.param_count, 0);
    }

    #[test]
    fn search_placeholder_index_is_correct_with_other_filters() {
        let mut q = empty_query();
        q.brand_id = Some(10);
        q.search = Some("new".to_string());
        q.price_min = Some(Decimal::new(1000, 2));

        let filters = build_filters(&q);

        assert_eq!(
            filters.where_clause,
            "WHERE 1=1 AND brand_id = $1 AND (name ILIKE $2 OR sku ILIKE $2) AND purchase_price >= $3"
        );
        assert_eq!(filters.param_count, 3);
    }
}

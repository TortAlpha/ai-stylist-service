use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Product Service API",
        description = "Second-hand clothing store — product catalog management",
        version = "0.1.0"
    ),
    paths(
        // Products
        crate::handler::product_handler::get_product_previews_by_query_admin,
        crate::handler::product_handler::get_product_by_id_admin,
        crate::handler::product_handler::create_product_admin,
        crate::handler::product_handler::update_product_by_id_admin,
        crate::handler::product_handler::delete_product_by_id_admin,
        crate::handler::product_handler::filter_options,
        crate::handler::product_handler::available_sizes,
        // Product Photos
        crate::handler::product_photo_handler::upload_images,
        crate::handler::product_photo_handler::upload_preview,
        // Brands
        crate::handler::brand_handler::list_brands,
        crate::handler::brand_handler::search_brands,
        crate::handler::brand_handler::get_brand_by_id,
        crate::handler::brand_handler::create_brand,
        crate::handler::brand_handler::update_brand,
        crate::handler::brand_handler::delete_brand,
        // Categories
        crate::handler::category_handler::list_categories,
        crate::handler::category_handler::list_categories_admin,
        crate::handler::category_handler::get_category_by_id,
        crate::handler::category_handler::create_category,
        crate::handler::category_handler::update_category,
        crate::handler::category_handler::delete_category,
        // Tags
        crate::handler::tag_handler::list_style_tags,
        crate::handler::tag_handler::create_style_tag,
        crate::handler::tag_handler::list_vibe_tags,
        crate::handler::tag_handler::create_vibe_tag,
        crate::handler::tag_handler::list_seasons,
        crate::handler::tag_handler::create_season,
        crate::handler::tag_handler::delete_style_tag,
        crate::handler::tag_handler::delete_vibe_tag,
        crate::handler::tag_handler::delete_season,
        // Purchase Locations
        crate::handler::purchase_location_handler::list_purchase_locations,
        crate::handler::purchase_location_handler::get_purchase_location_by_id,
        crate::handler::purchase_location_handler::create_purchase_location,
        crate::handler::purchase_location_handler::update_purchase_location_by_id,
        crate::handler::purchase_location_handler::delete_purchase_location_by_id,
    ),
    components(schemas(
        // Response DTOs
        crate::domain::utils::mappers::ImageVariantUrls,
        crate::domain::response_dto::product::ProductPreviewResponse,
        crate::domain::response_dto::product::AdminProductDTO,
        crate::domain::response_dto::product::ProductFilterOptions,
        crate::domain::response_dto::product_details::ProductAttributesResponse,
        crate::domain::response_dto::product_details::SizeResponse,
        crate::domain::response_dto::product_details::TypeDetailsResponse,
        crate::domain::response_dto::product_details::AvailableSizesResponse,
        crate::domain::response_dto::brand::BrandResponse,
        crate::domain::response_dto::brand::BrandShortResponse,
        crate::domain::response_dto::category::CategoryFullResponse,
        crate::domain::response_dto::category::CategoryResponse,
        crate::domain::response_dto::tag::TagResponse,
        crate::domain::response_dto::tag::ProductTagsResponse,
        // Request DTOs
        crate::domain::request_dto::product::CreateProductRequest,
        crate::domain::request_dto::product::UpdateProductRequest,
        crate::domain::request_dto::product_details::CreateProductDetailsRequest,
        crate::domain::request_dto::product_details::UpdateProductDetailsRequest,
        crate::domain::request_dto::product_details::SizeInput,
        crate::domain::request_dto::brand::CreateBrandRequest,
        crate::domain::request_dto::brand::UpdateBrandRequest,
        crate::domain::request_dto::category::CreateCategoryRequest,
        crate::domain::request_dto::category::UpdateCategoryRequest,
        crate::domain::request_dto::tag::CreateTagRequest,
        crate::domain::request_dto::purchase_location::CreatePurchaseLocationRequest,
        crate::domain::request_dto::purchase_location::UpdatePurchaseLocationRequest,
        // Domain enums
        crate::domain::product_details::ProductStatus,
        crate::domain::product_details::ProductCondition,
        crate::domain::product_details::TypeDetailsInput,
        crate::domain::brand::BrandTier,
        crate::domain::category::Gender,
        crate::domain::category::ProductType,
        crate::domain::category::SizeGroup,
        crate::domain::clothing_details::ClothingFit,
        crate::domain::footwear_details::ShoeWidth,
        crate::domain::bags_details::HandleType,
        crate::domain::size_info::SizeSystem,
        crate::domain::purchase_location::PurchaseLocation,
    )),
    tags(
        (name = "Products", description = "Product CRUD and filters"),
        (name = "Product Photos", description = "Image and preview upload"),
        (name = "Brands", description = "Brand management"),
        (name = "Categories", description = "Category management"),
        (name = "Tags", description = "Style tags, vibe tags, and seasons"),
        (name = "Purchase Locations", description = "Purchase location management"),
    ),
    security(("bearer" = []))
)]
pub struct ApiDoc;

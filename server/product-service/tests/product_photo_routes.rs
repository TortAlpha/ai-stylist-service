use actix_web::{App, http::StatusCode, test, web};

use product_service::handler::product_handler;

#[actix_web::test]
async fn preview_route_is_registered_under_admin_products_scope() {
    let app = test::init_service(
        App::new().service(web::scope("/api").configure(product_handler::configure)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/admin/products/d88ccb3a-c294-45bc-b7e4-3cab18a08841/preview")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_ne!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn images_route_is_registered_under_admin_products_scope() {
    let app = test::init_service(
        App::new().service(web::scope("/api").configure(product_handler::configure)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/admin/products/d88ccb3a-c294-45bc-b7e4-3cab18a08841/images")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_ne!(resp.status(), StatusCode::NOT_FOUND);
}

use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest, web};
use std::future::Future;
use std::pin::Pin;

use crate::config::Config;
use crate::domain::error::ServiceError;

/// Extractor: enforces a shared-secret header (`X-Internal-Token`) on
/// endpoints that nginx is not allowed to proxy from the outside world.
///
/// Empty `INTERNAL_API_TOKEN` is treated as a misconfiguration and rejected,
/// so an operator can't accidentally expose internal endpoints to anyone
/// who can reach the container's port.
#[derive(Debug, Clone, Copy)]
pub struct InternalAuth;

impl FromRequest for InternalAuth {
    type Error = ServiceError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let provided = req
            .headers()
            .get("X-Internal-Token")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let cfg = req.app_data::<web::Data<Config>>().cloned();

        Box::pin(async move {
            let cfg = cfg.ok_or_else(|| {
                ServiceError::Internal("internal config not available in app_data".into())
            })?;
            if cfg.internal_api_token.is_empty() {
                return Err(ServiceError::Unauthorized(
                    "INTERNAL_API_TOKEN not configured".into(),
                ));
            }
            let provided = provided.ok_or_else(|| {
                ServiceError::Unauthorized("Missing X-Internal-Token header".into())
            })?;
            if !ct_eq(provided.as_bytes(), cfg.internal_api_token.as_bytes()) {
                return Err(ServiceError::Unauthorized("Invalid X-Internal-Token".into()));
            }
            Ok(InternalAuth)
        })
    }
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

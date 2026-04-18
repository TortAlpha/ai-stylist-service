use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

use crate::storage::error::StorageError;

#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    StaleVersion,
    BadRequest(String),
    Internal(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            Self::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            Self::Conflict(msg) => write!(f, "Conflict: {msg}"),
            Self::StaleVersion => write!(f, "Conflict: resource was modified by another request"),
            Self::BadRequest(msg) => write!(f, "Bad request: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    success: bool,
    error: String,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        let body = ErrorBody {
            success: false,
            error: self.to_string(),
        };

        match self {
            Self::NotFound(_) => HttpResponse::NotFound().json(body),
            Self::Unauthorized(_) => HttpResponse::Unauthorized().json(body),
            Self::Forbidden(_) => HttpResponse::Forbidden().json(body),
            Self::Conflict(_) | Self::StaleVersion => HttpResponse::Conflict().json(body),
            Self::BadRequest(_) => HttpResponse::BadRequest().json(body),
            Self::Internal(_) => HttpResponse::InternalServerError().json(body),
        }
    }
}

impl From<StorageError> for ServiceError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::NotFound(msg) => Self::NotFound(msg),
            StorageError::PermissionDenied(msg) => Self::Internal(msg),
            StorageError::Internal(msg) => Self::Internal(msg),
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => Self::NotFound("resource not found".into()),

            sqlx::Error::Database(db_err) => {
                // PostgreSQL error codes
                match db_err.code().as_deref() {
                    // 23505 = unique_violation
                    Some("23505") => {
                        let detail = db_err.message().to_string();
                        Self::Conflict(detail)
                    }
                    // 23503 = foreign_key_violation
                    Some("23503") => {
                        let detail = db_err.message().to_string();
                        Self::BadRequest(detail)
                    }
                    // 23502 = not_null_violation
                    Some("23502") => {
                        let detail = db_err.message().to_string();
                        Self::BadRequest(detail)
                    }
                    // 23514 = check_violation
                    Some("23514") => {
                        let detail = db_err.message().to_string();
                        Self::BadRequest(detail)
                    }
                    _ => Self::Internal(db_err.message().to_string()),
                }
            }

            _ => Self::Internal(err.to_string()),
        }
    }
}

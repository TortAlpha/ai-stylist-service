use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    Internal(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::Conflict(msg) => write!(f, "Conflict: {msg}"),
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
        let body = ErrorBody { success: false, error: self.to_string() };
        match self {
            Self::NotFound(_) => HttpResponse::NotFound().json(body),
            Self::Conflict(_) => HttpResponse::Conflict().json(body),
            Self::BadRequest(_) => HttpResponse::BadRequest().json(body),
            Self::Internal(_) => HttpResponse::InternalServerError().json(body),
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => Self::NotFound("resource not found".into()),
            sqlx::Error::Database(db_err) => {
                match db_err.code().as_deref() {
                    Some("23505") => Self::Conflict(db_err.message().to_string()),
                    Some("23503") => Self::BadRequest(db_err.message().to_string()),
                    _ => Self::Internal(db_err.message().to_string()),
                }
            }
            _ => Self::Internal(err.to_string()),
        }
    }
}

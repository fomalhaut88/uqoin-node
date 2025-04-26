use serde::Serialize;
use actix_web::{ResponseError, HttpResponse};
use actix_web::http::{StatusCode, header::ContentType};


#[derive(Debug, Serialize)]
pub struct JsonError {
    detail: String,
}


impl JsonError {
    pub fn new(detail: &str) -> Self {
        Self { detail: detail.to_string() }
    }
}


impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.detail)
    }
}


impl<E: std::error::Error> From<E> for JsonError {
    fn from(err: E) -> Self {
        Self::new(&err.to_string())
    }
}


impl ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self)
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}


/// Check condition and raise API error.
#[macro_export]
macro_rules! api_check {
    ($cond:expr, $detail:expr) => {
        if !$cond {
            return Err(crate::error::JsonError::new($detail));
        }
    }
}

use actix_web::{web, HttpResponse, Scope};

use crate::utils::*;


/// Get block information by `bix` of last one.
async fn block_view() -> APIResult {
    Ok(HttpResponse::Ok().finish())
}


/// Get transaction information by `tix` adding its group if requested.
async fn transaction_view() -> APIResult {
    Ok(HttpResponse::Ok().finish())
}


pub fn load_scope() -> Scope {
    web::scope("/blockchain")
        .route("/block", web::get().to(block_view))
        .route("/transaction", web::get().to(transaction_view))
}

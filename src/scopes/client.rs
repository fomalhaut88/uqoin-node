use actix_web::{web, HttpResponse, Scope};

use crate::utils::*;


/// Get coins belonging to the wallet at the current last block.
async fn coins_view() -> APIResult {
    Ok(HttpResponse::Ok().finish())
}


/// Send transaction group.
async fn send_view() -> APIResult {
    Ok(HttpResponse::Ok().finish())
}


pub fn load_scope() -> Scope {
    web::scope("/client")
        .route("/coins", web::get().to(coins_view))
        .route("/send", web::post().to(send_view))
}

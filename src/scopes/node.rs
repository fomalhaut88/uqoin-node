use actix_web::{web, HttpResponse, Scope};

use crate::utils::*;


/// Get list of known nodes.
async fn get_view(appdata: WebAppData) -> APIResult {
    let nodes = appdata.nodes.read().await.clone();
    Ok(HttpResponse::Ok().json(nodes))
}


pub fn load_scope() -> Scope {
    web::scope("/node")
        .route("/list", web::get().to(get_view))
}

use actix_web::{web, HttpResponse, Scope};

use crate::utils::*;


/// Get list of known validators.
async fn get_view(appdata: WebAppData) -> APIResult {
    let validators = appdata.validators.read().await.clone();
    Ok(HttpResponse::Ok().json(validators))
}


pub fn load_scope() -> Scope {
    web::scope("/validator")
        .route("/list", web::get().to(get_view))
}

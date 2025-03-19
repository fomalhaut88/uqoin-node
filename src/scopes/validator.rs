use actix_web::{web, HttpResponse, Resource};

use crate::utils::*;


/// Get all known validators.
async fn get_view() -> APIResult {
    Ok(HttpResponse::Ok().finish())
}


/// Create a new validator to sync (permission required).
async fn create_view() -> APIResult {
    Ok(HttpResponse::NoContent().finish())
}


/// Update the validator (permission required).
async fn update_view() -> APIResult {
    Ok(HttpResponse::NoContent().finish())
}


/// Delete the validator (permission required).
async fn delete_view() -> APIResult {
    Ok(HttpResponse::NoContent().finish())
}


pub fn load_resource() -> Resource {
    web::resource("/validator")
        .route(web::get().to(get_view))
        .route(web::post().to(create_view))
        .route(web::put().to(update_view))
        .route(web::delete().to(delete_view))
}

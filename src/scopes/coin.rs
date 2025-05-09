use serde::Deserialize;
use actix_web::{web, HttpResponse, Scope};
use actix_web::http::header::ContentType;
use uqoin_core::utils::U256;

use crate::api_check;
use crate::utils::*;


#[derive(Deserialize)]
struct Query {
    coin: String,
}


/// Get coin info.
async fn info_view(appdata: WebAppData, 
                   query: web::Query<Query>) -> APIResult {
    api_check!(!*appdata.is_syncing.read().await, "Syncing");
    let coin = U256::from_hex(&query.coin);
    let state = appdata.state.read().await;
    if let Some(coin_info) = state.get_coin_info(&coin) {
        Ok(HttpResponse::Ok().json(coin_info))
    } else {
        Ok(HttpResponse::Ok().insert_header(ContentType::json()).body("{}"))
    }
}


pub fn load_scope() -> Scope {
    web::scope("/coin")
        .route("/info", web::get().to(info_view))
}

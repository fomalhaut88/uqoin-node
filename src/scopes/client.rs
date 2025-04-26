use serde::Deserialize;
use actix_web::{web, HttpResponse, Scope};
use actix_web::http::header::ContentType;
use uqoin_core::utils::*;
use uqoin_core::transaction::{Type, Transaction, Group};

use crate::api_check;
use crate::utils::*;


#[derive(Deserialize)]
struct Query {
    wallet: String,
}


/// Get coins belonging to the wallet at the current last block.
async fn coins_view(appdata: WebAppData, 
                    query: web::Query<Query>) -> APIResult {
    api_check!(!*appdata.is_syncing.read().await, "Syncing");
    let wallet = U256::from_hex(&query.wallet);
    let state = appdata.state.read().await;
    if let Some(coins_map) = state.get_coins(&wallet) {
        Ok(HttpResponse::Ok().json(coins_map))
    } else {
        Ok(HttpResponse::Ok().insert_header(ContentType::json()).body("{}"))
    }
}


/// Send transaction group.
async fn send_view(appdata: WebAppData, 
                   transactions: web::Json<Vec<Transaction>>) -> APIResult {
    // Check syncing
    api_check!(!*appdata.is_syncing.read().await, "Syncing");

    // Check lite mode (if private key is not provided)
    api_check!(!appdata.config.lite_mode, "LiteMode");

    // Get state
    let state = appdata.state.read().await;

    // Calc senders
    let senders = Transaction::calc_senders(&transactions, &state, 
                                            &appdata.schema);

    // Try to create group from raw transactions
    let group = Group::new(transactions.to_vec(), &state, &senders)?;

    // Skip split transactions for fee check
    if (group.get_type() != Type::Split) || (!appdata.config.free_split) {
        // Get client fee
        let fee_order = group.get_fee()
            .map(|tr| tr.get_order(&state, &senders[0])).unwrap_or(0);

        // Check fee
        api_check!(fee_order >= appdata.config.fee_min_order, "Fee");
    }

    // Insert the group into pool
    appdata.pool.write().await.add(group, senders[0].clone());

    // Ok
    Ok(HttpResponse::Ok().finish())
}


pub fn load_scope() -> Scope {
    web::scope("/client")
        .route("/coins", web::get().to(coins_view))
        .route("/send", web::post().to(send_view))
}

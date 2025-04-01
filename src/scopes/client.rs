use serde::Deserialize;
use actix_web::{web, HttpResponse, Scope};
use uqoin_core::utils::*;
use uqoin_core::transaction::{Transaction, Group};

use crate::utils::*;


#[derive(Deserialize)]
struct Query {
    wallet: String,
}


/// Get coins belonging to the wallet at the current last block.
async fn coins_view(appdata: WebAppData, 
                    query: web::Query<Query>) -> APIResult {
    let wallet = U256::from_hex(&query.wallet);
    let state = appdata.state.read().await;
    if let Some(coins_map) = state.get_coins(&wallet) {
        Ok(HttpResponse::Ok().json(coins_map))
    } else {
        Ok(HttpResponse::Ok().body("{}"))
    }
}


/// Send transaction group.
async fn send_view(appdata: WebAppData, 
                   transactions: web::Json<Vec<Transaction>>) -> APIResult {
    // Check lite mode (if private key is not provided)
    if appdata.config.lite_mode {
        Ok(HttpResponse::BadRequest().json(ErrorResponse::new("LiteMode")))
    } else {
        // Get state
        let state = appdata.state.read().await;

        // Calc senders
        let senders = Transaction::calc_senders(&transactions, &state, 
                                                &appdata.schema);

        // Create group from raw transactions
        match Group::new(transactions.to_vec(), &state, &senders) {
            Ok(group) => {
                // Get client fee
                let fee_order = group.get_fee()
                    .map(|tr| tr.get_order(&state, &senders[0])).unwrap_or(0);

                // Check fee
                if fee_order >= appdata.config.fee_min_order {
                    // Insert the group into pool
                    match appdata.pool.write().await.add_group(&group, &state, 
                                                               &senders[0]) {
                        Ok(_) => Ok(HttpResponse::Ok().finish()),
                        Err(err) => Ok(HttpResponse::BadRequest()
                                            .json(ErrorResponse::from(err))),
                    }
                } else {
                    Ok(HttpResponse::BadRequest()
                            .json(ErrorResponse::new("Fee")))
                }
            },
            Err(err) => Ok(HttpResponse::BadRequest()
                                .json(ErrorResponse::from(err))),
        }
    }
}


pub fn load_scope() -> Scope {
    web::scope("/client")
        .route("/coins", web::get().to(coins_view))
        .route("/send", web::post().to(send_view))
}

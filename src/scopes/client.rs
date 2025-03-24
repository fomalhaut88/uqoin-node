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
    // Get state
    let state = appdata.state.read().await;

    // Create group from raw transactions
    if let Some(group) = Group::new(transactions.to_vec(), &appdata.schema, 
                                    &state) {
        // Get client fee
        let fee_order = group.get_fee()
            .map(|tr| tr.get_order(&state, &appdata.schema)).unwrap_or(0);

        // Check fee
        if fee_order >= appdata.config.fee_min_order {
            // Insert the group into pool
            let mut pool = appdata.pool.write().await;
            let added = pool.add_group(&group, &appdata.schema, &state);
            if added {
                return Ok(HttpResponse::Ok().finish());
            }
        }
    }

    // TODO: Implement more verbose information on error.
    Ok(HttpResponse::BadRequest().finish())
}


pub fn load_scope() -> Scope {
    web::scope("/client")
        .route("/coins", web::get().to(coins_view))
        .route("/send", web::post().to(send_view))
}

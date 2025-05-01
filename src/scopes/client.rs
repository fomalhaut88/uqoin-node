use std::collections::HashMap;

use serde::Deserialize;
use actix_web::{web, HttpResponse, Scope};
use actix_web::http::header::ContentType;
use uqoin_core::utils::*;
use uqoin_core::transaction::{Type, Transaction, Group};

use crate::api_check;
use crate::utils::*;


#[derive(Deserialize)]
struct CoinsQuery {
    wallet: String,
    order: Option<u64>,
}


/// Get coins belonging to the wallet at the last block. If `order` is  
/// specified, the result coins are represented as a list, else the full mapping
/// order - coin list will be returned. It is recommended not to use the 
/// endpoint often without `order` due to too big size of the coins map.
async fn coins_view(appdata: WebAppData, 
                    query: web::Query<CoinsQuery>) -> APIResult {
    // Check syncing
    api_check!(!*appdata.is_syncing.read().await, "Syncing");

    // Prepare wallet number
    let wallet = U256::from_hex(&query.wallet);

    // Get state that contains the coins information
    let state = appdata.state.read().await;

    // If order is specified in the query, else return full map
    if let Some(order) = query.order {
        // If coins map is found in the state, else return empty empty list
        if let Some(coins_map) = state.get_coins(&wallet) {
            // If coins is found for the order, else return empty list
            if let Some(coins) = coins_map.get(&order) {
                // Return coins as list
                Ok(HttpResponse::Ok().json(coins))
            } else {
                // Return empty list
                Ok(HttpResponse::Ok().insert_header(ContentType::json())
                                     .body("[]"))
            }
        } else {
            // Return empty list
            Ok(HttpResponse::Ok().insert_header(ContentType::json())
                                 .body("[]"))
        }
    } else {
        // If coins map is found in the state, else return empty map {}
        if let Some(coins_map) = state.get_coins(&wallet) {
            // Return full map
            Ok(HttpResponse::Ok().json(coins_map))
        } else {
            // Return empty map
            Ok(HttpResponse::Ok().insert_header(ContentType::json()).body("{}"))
        }
    }
}


/// Same as `coins_view` but returning XOR hash instead of full coin lists.
async fn coins_hash_view(appdata: WebAppData, 
                         query: web::Query<CoinsQuery>) -> APIResult {
    // Check syncing
    api_check!(!*appdata.is_syncing.read().await, "Syncing");

    // Prepare wallet number
    let wallet = U256::from_hex(&query.wallet);

    // Get state that contains the coins information
    let state = appdata.state.read().await;

    // If order is specified in the query, else return full map
    if let Some(order) = query.order {
        let xor = state.calc_coins_hash(&wallet, order)
                       .unwrap_or(U256::from(0));
        Ok(HttpResponse::Ok().json(xor))
    } else {
        // If coins map is found in the state, else return empty map {}
        if let Some(coins_map) = state.get_coins(&wallet) {
            // Return full map
            let xor_map: HashMap<u64, U256> = coins_map.keys()
            .map(|order| (
                *order,
                state.calc_coins_hash(&wallet, *order).unwrap()
            )).collect();
        Ok(HttpResponse::Ok().json(xor_map))
        } else {
            // Return empty map
            Ok(HttpResponse::Ok().insert_header(ContentType::json()).body("{}"))
        }
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
        .route("/coins/hash", web::get().to(coins_hash_view))
        .route("/send", web::post().to(send_view))
}

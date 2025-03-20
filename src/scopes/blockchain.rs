use serde::{Serialize, Deserialize};
use actix_web::{web, HttpResponse, Scope};
use uqoin_core::block::Block;
use uqoin_core::transaction::Transaction;

use crate::utils::*;


#[derive(Deserialize)]
struct BlockQuery {
    bix: Option<u64>,
    ext: Option<bool>,
}


#[derive(Deserialize)]
struct TransactionQuery {
    tix: u64,
}


#[derive(Serialize)]
struct BlockData {
    bix: u64,
    block: Block,
    transactions: Option<Vec<Transaction>>,
}


/// Get block information by `bix` of last one.
async fn block_view(appdata: WebAppData, 
                    query: web::Query<BlockQuery>) -> APIResult {
    let blockchain = appdata.blockchain.read().await;

    let bix = query.bix.unwrap_or(blockchain.get_block_count().await?);

    if bix > 0 {
        let block = blockchain.get_block(bix).await?;

        let transactions = if query.ext.unwrap_or(false) {
            Some(blockchain.get_transactions_of_block(bix).await?)
        } else {
            None
        };

        Ok(HttpResponse::Ok().json(BlockData { bix, block, transactions }))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}


/// Get transaction information by `tix` (TODO: adding its group if requested).
async fn transaction_view(appdata: WebAppData, 
                          query: web::Query<TransactionQuery>) -> APIResult {
    let blockchain = appdata.blockchain.read().await;
    let transaction = blockchain.get_transaction(query.tix).await?;
    Ok(HttpResponse::Ok().json(transaction))
}


pub fn load_scope() -> Scope {
    web::scope("/blockchain")
        .route("/block", web::get().to(block_view))
        .route("/transaction", web::get().to(transaction_view))
}

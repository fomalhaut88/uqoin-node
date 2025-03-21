use serde::{Serialize, Deserialize};
use actix_web::{web, HttpResponse, Scope};
use uqoin_core::utils::U256;
use uqoin_core::block::Block;
use uqoin_core::transaction::Transaction;
use uqoin_core::state::{BlockInfo, GENESIS_HASH};

use crate::utils::*;


#[derive(Deserialize)]
struct BlockQuery {
    bix: Option<u64>,
}


#[derive(Deserialize)]
struct TransactionQuery {
    tix: u64,
}


#[derive(Serialize)]
struct BlockData {
    bix: u64,
    block: Block,
    transactions: Vec<Transaction>,
}


/// Get block info by `bix`.
async fn block_info_view(appdata: WebAppData, 
                         query: web::Query<BlockQuery>) -> APIResult {
    let blockchain = appdata.blockchain.read().await;

    let bix = query.bix.unwrap_or(blockchain.get_block_count().await?);

    let block_info = if bix > 0 {
        let block = blockchain.get_block(bix).await?;
        BlockInfo {
            bix,
            offset: block.offset + block.size,
            hash: block.hash,
        }
    } else {
        BlockInfo {
            bix: 0,
            offset: 0,
            hash: U256::from_hex(GENESIS_HASH),
        }
    };

    Ok(HttpResponse::Ok().json(block_info))
}


/// Get block data by `bix`.
async fn block_data_view(appdata: WebAppData, 
                         query: web::Query<BlockQuery>) -> APIResult {
    let blockchain = appdata.blockchain.read().await;

    let bix = query.bix.unwrap_or(blockchain.get_block_count().await?);

    if bix > 0 {
        let block = blockchain.get_block(bix).await?;
        let transactions = blockchain.get_transactions_of_block(&block).await?;
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
        .route("/block-info", web::get().to(block_info_view))
        .route("/block-data", web::get().to(block_data_view))
        .route("/transaction", web::get().to(transaction_view))
}

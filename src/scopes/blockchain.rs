use serde::{Serialize, Deserialize};
use actix_web::{web, HttpResponse, Scope};
use uqoin_core::block::{BlockInfo, BlockData};

use crate::utils::*;


#[derive(Serialize, Deserialize)]
pub struct BlockQuery {
    pub bix: Option<u64>,
}


#[derive(Serialize, Deserialize)]
pub struct TransactionQuery {
    pub tix: u64,
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
        BlockInfo::genesis()
    };

    Ok(HttpResponse::Ok().json(block_info))
}


/// Get block data by `bix`.
async fn block_data_view(appdata: WebAppData, 
                         query: web::Query<BlockQuery>) -> APIResult {
    let blockchain = appdata.blockchain.read().await;

    let bix = query.bix.unwrap_or(blockchain.get_block_count().await?);

    let block_data = if bix > 0 {
        let block = blockchain.get_block(bix).await?;
        let transactions = blockchain.get_transactions_of_block(&block).await?;
        BlockData { bix, block, transactions }
    } else {
        BlockData::genesis()
    };

    Ok(HttpResponse::Ok().json(block_data))
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

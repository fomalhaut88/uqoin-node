# uqoin-node

Node for Uqoin ecosystem.

## Run in docker

Pull the image: `docker pull fomalhaut88/uqoin-node`

Run command:

```
sudo docker run \
    -p 5772:5772 \
    --restart always \
    --volume /var/lib/uqoin-node:/app/data \
    --name uqoin-node-app \
    --env PRIVATE_KEY=054A773A579D5F08817F1EFA5B19837C25DBB4A2D07C67D10D683A9A22B6D96A \
    --env NODES="http://85.99.244.254:5772 http://89.179.245.236:5772 http://89.179.245.236:5773" \
    --env WORKERS=4 \
    --env MINING_THREADS=4 \
    --env MINING_GROUPS_MAX=20 \
    --env NODE_SYNC_TIMEOUT=10000 \
    --env FEE_MIN=D1 \
    -d fomalhaut88/uqoin-node
```

## API description

| Path | Method | Description | Params | Input body example | Output body example |
|---|---|---|---|---|---|
| `/version` | `GET` | Get version of the node. | | | `{"version": "1.0.0"}` |
| `/client/coins` | `GET` | Get coins of the client. | `wallet: str` - wallet address, `order: int` - order of coins (optional; it returns list of coins instead of full map, made to the optimization purposes) | | `{35: [...], ...}` |
| `/client/coins/hash` | `GET` | Get coins hashs for each order of the client. | `wallet: str` - wallet address, `order: int` - order of coins (optional; it returns single hash of coins instead of full map, made to the optimization purposes) | | `{35: ..., ...}` |
| `/client/send` | `POST` | Send transaction to the node. | | `[{"coin": "...", "addr": "...", "sign_r": "...", "sign_s": "..."}, ...]` | |
| `/coin/info` | `GET` | Get creation information about the coin. | `coin: str` - coin number | | `{"order": ..., "tix": ..., "bix": ...}` |
| `/coin/owner` | `GET` | Get current owner wallet of the coin. | `coin: str` - coin number | | `{"wallet": ...}` |
| `/blockchain/block-info` | `GET` | Get short information about the block. | `bix: int` - number of the block (last block if not specified) | | `{"bix": ..., "offset": ..., "hash": ...}` |
| `/blockchain/block-data` | `GET` | Get extended information about the block including transactions. | `bix: int` - number of the block (last block if not specified) | | `{"bix": ..., "block": {...}, "transactions": [...]}` |
| `/blockchain/transaction` | `GET` | Get transaction by the number. | `tix: int` - number of the transaction | | `{"coin": "...", "addr": "...", "sign_r": "...", "sign_s": "..."}` |
| `/node/list` | `GET` | Get list of the nodes to sync. | | | `[...]` |
| `/node/info` | `GET` | Get node information. | | | `{"wallet": "...", "fee": "...", "lite_mode": "..."}` |

## Environment variables

| Variable | Description | Default |
|---|---|---|
| `PRIVATE_KEY` | Private key of the wallet. | - |
| `NODES` | URL list of the nodes to sync. | - |
| `HOST` | Host to deploy. | `localhost` |
| `PORT` | Port to deploy. | `5772` |
| `DATA_PATH` | Path to the directory for the data. | `./tmp` |
| `WORKERS` | Number of workers to process API. | `1` |
| `MINING_THREADS` | Number of threads in mining. | `1` |
| `FEE_MIN` | Minimum fee coin symbol to accept transactions. | - |
| `NODE_SYNC_TIMEOUT` | Timeout between node syncing (in milliseconds). | `5000` |
| `NODE_SYNC_BLOCK_COUNT` | Maximum allowed number of blocks to sync. | `1000` |
| `MINING_TIMEOUT` | Timeout between mining block attempts (in milliseconds). | `20000` |
| `MINING_UPDATE_COUNT` | Number of transactions update while a new block is being mined. | `20` |
| `MINING_NONCE_COUNT_PER_ITERATION` | Number of mining attempts per thread in iteration. | `100000` |
| `MINING_GROUPS_MAX` | Maximum number of groups in mined blocks. | - |
| `FREE_SPLIT` | Allow split transactions without fee. | `true` |

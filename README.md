# uqoin-node

Node for Uqoin ecosystem.

## Run in docker

Build image example:

```
docker build -t uqoin-node .
```

Run two docker containers locally example:

```
docker run \
    -it --rm \
    -p 8081:5772 \
    --volume ./tmp/1:/app/data \
    --name uqoin-node-app-1 \
    --env PRIVATE_KEY=054A773A579D5F08817F1EFA5B19837C25DBB4A2D07C67D10D683A9A22B6D96A \
    --env NODES="http://172.17.0.2:5772 http://172.17.0.3:5772" \
    --env WORKERS=4 \
    --env MINING_THREADS=4 \
    uqoin-node

docker run \
    -it --rm \
    -p 8082:5772 \
    --volume ./tmp/2:/app/data \
    --name uqoin-node-app-2 \
    --env PRIVATE_KEY=0250F121A6DB4C3114B09F7905A274D2DB024A0CA9DDDA5E41EEF97A5E26589B \
    --env NODES="http://172.17.0.2:5772 http://172.17.0.3:5772" \
    --env WORKERS=4 \
    --env MINING_THREADS=4 \
    uqoin-node
```

## API description

| Path | Method | Description | Params | Input body example | Output body example |
|---|---|---|---|---|---|
| `/version` | `GET` | Get version of the node. | | | `{"version": "1.0.0"}` |
| `/client/coins` | `GET` | Get coins of the client. | `wallet: str` - wallet address | | `{35: [...], ...}` |
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
| `PRIVATE_KEY` | Private key of the wallet. | **required** |
| `NODES` | URL list of the nodes to sync. | **required** |
| `HOST` | Host to deploy. | `localhost` |
| `PORT` | Port to deploy. | `5772` |
| `DATA_PATH` | Path to the directory for the data. | `./tmp` |
| `WORKERS` | Number of workers to process API. | `1` |
| `MINING_THREADS` | Number of threads in mining. | `1` |
| `FEE_MIN_ORDER` | Minimum fee coin symbol to accept transactions. | - |
| `LITE_MODE` | Enable lite mode: without mining and accepting `send` transactions. | `false` |
| `NODE_SYNC_TIMEOUT` | Timeout between node syncing (in milliseconds). | `5000` |
| `MINING_TIMEOUT` | Timeout between mining block attempts (in milliseconds). | `20000` |
| `MINING_UPDATE_COUNT` | Number of transactions update while a new block is being mined. | `20` |
| `MINING_NONCE_COUNT_PER_ITERATION` | Number of mining attempts per thread in iteration. | `100000` |
| `MINING_GROUPS_MAX` | Maximum number of groups in mined blocks. | - |

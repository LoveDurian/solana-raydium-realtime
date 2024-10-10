# Raydium price-feeder

## Overview
This project provides real-time price updates for tokens in Raydium CLMM pools on Solana. It listens for changes in the pool state using Solana's PubsubClient and updates token prices based on current pool conditions. Furthermore, it offers basic swap functionality.

## Features
- Realtime price updates through listening account changes of the Raydium pools and ticks
- Basic swap functionality
- Basic account creation and encryption

## How It Works
- Subscribes to the pool's state updates via Solana's RPC.
- Processes changes in the pool's liquidity and price ticks to compute up-to-date price.
- Broadcasts real-time price updates to the users using websockets.

## Build
### Docker
```bash
DOCKER_BUILDKIT=1 docker build -t raydium-price-feeder .
```

## Usage
Application provides a basic CLI for managing account generation and starting the service.

### Generation of a master key
```bash
MASTER_KEY=$(docker run raydium-price-feeder new masterkey)
```

### Creation of a new wallet
```bash
JWT_SECERT=$(opennssl rand -hex 32)
docker run raydium-price-feeder new wallet --duration 5m --masterkey $MASTER_KEY --jwt-secret $JWT_SECERT --database-url <PSQL_DATABASE_URL>
```

### Starting the server
```bash
docker run -p 8080:8080 raydium-price-feeder server --masterkey $MASTER_KEY --jwt-secret $JWT_SECERT --rpc <RPC_NODE> --database-url <PSQL_DATABASE_URL>
```

**Note:** For detailed help refer to the help command.


## API

### GET /api/price-feed
```bash
MINT_0=SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt
MINT_1=So11111111111111111111111111111111111111112
FEE_INDEX=0 # Raydium amm config index of the pool

curl "http://localhost:8080/api/price-feed?mint0=$MINT_0&mint1=$MINT_1&fee_index=$FEE_INDEX" -H "Authorization: Bearer <JWT_TOKEN>"
```

<details>
<summary>More about fee index</summary>
Essentially, the fee index is a unique identifier for a specific pool configuration. It is used to identify the pool's configuration within the Raydium system. For the end user, the fee index is a number that represents the pool's trade fee rate.

```javascript
export const devConfigs = [
  {
    id: 'CQYbhr6amxUER4p5SC44C63R4qw4NFc9Z4Db9vF4tZwG',
    index: 0,
    protocolFeeRate: 120000,
    tradeFeeRate: 100,
    tickSpacing: 10,
    fundFeeRate: 40000,
    description: 'Best for very stable pairs',
    defaultRange: 0.005,
    defaultRangePoint: [0.001, 0.003, 0.005, 0.008, 0.01],
  },
  {
    id: 'B9H7TR8PSjJT7nuW2tuPkFC63z7drtMZ4LoCtD7PrCN1',
    index: 1,
    protocolFeeRate: 120000,
    tradeFeeRate: 2500,
    tickSpacing: 60,
    fundFeeRate: 40000,
    description: 'Best for most pairs',
    defaultRange: 0.1,
    defaultRangePoint: [0.01, 0.05, 0.1, 0.2, 0.5],
  },
  {
    id: 'GjLEiquek1Nc2YjcBhufUGFRkaqW1JhaGjsdFd8mys38',
    index: 3,
    protocolFeeRate: 120000,
    tradeFeeRate: 10000,
    tickSpacing: 120,
    fundFeeRate: 40000,
    description: 'Best for exotic pairs',
    defaultRange: 0.1,
    defaultRangePoint: [0.01, 0.05, 0.1, 0.2, 0.5],
  },
  {
    id: 'GVSwm4smQBYcgAJU7qjFHLQBHTc4AdB3F2HbZp6KqKof',
    index: 2,
    protocolFeeRate: 120000,
    tradeFeeRate: 500,
    tickSpacing: 10,
    fundFeeRate: 40000,
    description: 'Best for tighter ranges',
    defaultRange: 0.1,
    defaultRangePoint: [0.01, 0.05, 0.1, 0.2, 0.5],
  },
]
const mainnetConfigs = [
    {
      id: '9iFER3bpjf1PTTCQCfTRu17EJgvsxo9pVyA9QWwEuX4x',
      index: 4,
      protocolFeeRate: 120000,
      tradeFeeRate: 100,
      tickSpacing: 1,
      fundFeeRate: 40000,
      description: 'Best for very stable pairs',
      defaultRange: 0.005,
      defaultRangePoint: [0.001, 0.003, 0.005, 0.008, 0.01],
    },
    {
      id: '3XCQJQryqpDvvZBfGxR7CLAw5dpGJ9aa7kt1jRLdyxuZ',
      index: 5,
      protocolFeeRate: 120000,
      tradeFeeRate: 500,
      tickSpacing: 1,
      fundFeeRate: 40000,
      description: 'Best for tighter ranges',
      defaultRange: 0.1,
      defaultRangePoint: [0.01, 0.05, 0.1, 0.2, 0.5],
    },
    {
      id: 'E64NGkDLLCdQ2yFNPcavaKptrEgmiQaNykUuLC1Qgwyp',
      index: 1,
      protocolFeeRate: 120000,
      tradeFeeRate: 2500,
      tickSpacing: 60,
      fundFeeRate: 40000,
      description: 'Best for most pairs',
      defaultRange: 0.1,
      defaultRangePoint: [0.01, 0.05, 0.1, 0.2, 0.5],
    },
    {
      id: 'A1BBtTYJd4i3xU8D6Tc2FzU6ZN4oXZWXKZnCxwbHXr8x',
      index: 3,
      protocolFeeRate: 120000,
      tradeFeeRate: 10000,
      tickSpacing: 120,
      fundFeeRate: 40000,
      description: 'Best for exotic pairs',
      defaultRange: 0.1,
      defaultRangePoint: [0.01, 0.05, 0.1, 0.2, 0.5],
    },
]
```
</details>

### GET /api/ws/price-feed
```bash
MINT_0=SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt
MINT_1=So11111111111111111111111111111111111111112
FEE_INDEX=0

websocat "ws://localhost:8080/api/price-feed?mint0=$MINT_0&mint1=$MINT_1&fee_index=$FEE_INDEX" -H "Authorization: Bearer <JWT_TOKEN>"
```

### GET /api/swap
```bash
PAYER=<PUBKEY_OF_PAYER_WALLET_ADDRESS>
MINT_0=SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt
MINT_1=So11111111111111111111111111111111111111112
INPUT_ACCOUNT=$(spl-token accounts $MINT_0 --owner $PAYER --output json-compact | jq .accounts[0].address)
OUTPUT_ACCOUNT=$(spl-token accounts $MINT_1 --owner $PAYER --output json-compact | jq .accounts[0].address)
AMOUNT=0.5
SLIPPAGE=0.01
FEE_INDEX=0

curl -XPOST "http://localhost:8080/api/swap" -H "Content-Type: application/json" -d '{"mint0": "$MINT_0", "mint1": "$MINT_1", "input_account": "$INPUT_ACCOUNT", "output_account": "$OUTPUT_ACCOUNT", "amount": "$AMOUNT", "slippage": "$SLIPPAGE", "fee_index": "$FEE_INDEX", "simulate": true}' -H "Authorization: Bearer <JWT_TOKEN>"
```

<details>
<summary>More about input/output account</summary>
Basically the input/output account is the account that holds the tokens that will be swapped.
Easieast way to create it is by sending a token thorugh wallet to the payer address.
</details>

## Mint addresses on mainnet to play around
- USDC EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
- WBTC 3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh
- WETH 7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs
- SOL 11111111111111111111111111111111
- WSOL So11111111111111111111111111111111111111112
- RAY 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R


## License GPL-3.0

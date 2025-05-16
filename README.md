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





在我们的 swap.rs 中，swapv2的传入账户是通过以下步骤构建的：
1.通过 Pool::load 加载池子信息
2.通过用户输入获取 input_token 和 output_token
3.在 pool.swap() 方法中构建完整的交易指令
4.通过 swap_v2_instr 函数组装所有账户

该项目是纯rust语言编写的后端代码，没有使用任何的solana sdk，而是使用solana-client-rs库。没有任何合约代码，只是调用solana-client-rs库。




让我一个个问题帮你分析：

1. 关于价格获取和交易流程：
是的，通常流程是：
```rust
// 1. 先通过 GET /price-feed 获取价格
#[get("/price-feed")]
async fn price_feed_pooling(
    query: web::Query<PriceFeedQuery>,
    state: web::Data<AppState>,
    _auth: AuthorizationGuard,
) -> impl Responder {
    // ... 获取当前价格
    let amount_out = pool
        .quote(
            state.rpc.clone(),
            ui_amount_to_amount(1f64, pool.state.mint_decimals_0),
            0,
            true,
            true,
        )
        .await
        .unwrap();
}

// 2. 然后通过 POST /swap 执行交易
#[post("/swap")]
async fn swap_route(
    state: web::Data<AppState>,
    AuthorizationGuard(account_id): AuthorizationGuard,
    body: web::Json<SwapBody>,
) -> actix_web::error::Result<Json<TransactionResult>> {
    // ... 执行交易
}
```

2. 关于价格追踪机制：
这里不是追踪成交价格，而是通过监听 Raydium 池子的状态变化来计算当前的价格。具体来说：

```rust
// 在 Pool::quote 方法中
pub async fn quote(
    &self,
    rpc: Arc<RpcClient>,
    amount: u64,
    _sqrt_price_limit_x64: u128,
    zero_for_one: bool,
    is_base_input: bool,
) -> anyhow::Result<u64> {
    // 获取当前池子的状态
    let epoch = rpc.get_epoch_info().await?.epoch;
    
    // 计算价格，这是基于当前池子的流动性状态计算的
    let (other_amount_threshold, _) = utils::get_out_put_amount_and_remaining_accounts(
        amount_specified,
        sqrt_price_limit_x64,
        zero_for_one,
        is_base_input,
        &self.amm_config,
        &self.state,
        &self.tick_array_bitmap_extension,
        &mut self.tick_array.clone(),
    )
    .unwrap();
}
```

3. 关于 WebSocket 实时更新：
WebSocket 实现在 `src/routes/ws/price_feed.rs` 中，它会实时推送价格更新：

```rust
#[get("/ws/price-feed")]
pub async fn stream_price_feed(
    req: HttpRequest,
    query: web::Query<PriceFeedQuery>,
    state: web::Data<AppState>,
    _auth: AuthorizationGuard,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    // 建立 WebSocket 连接
    let (res, mut session, _) = actix_ws::handle(&req, stream)?;

    // 订阅价格更新
    let mut broadcast = state.broadcast.lock().await;
    let mut subscription = broadcast
        .subscribe(&state.raydium_program_id, mint0, mint1, query.fee_index)
        .await
        .unwrap();

    // 在后台任务中持续监听并推送更新
    tokio::spawn(async move {
        while let Ok(pool) = subscription.recv().await {
            let amount_out = pool
                .quote(
                    state.rpc.clone(),
                    ui_amount_to_amount(1f64, pool.state.mint_decimals_0),
                    0,
                    true,
                    true,
                )
                .await
                .unwrap();

            // 推送价格更新给客户端
            session
                .text(
                    serde_json::to_string(&PriceFeedResponse {
                        amount_out: amount_to_ui_amount_string(
                            amount_out,
                            pool.state.mint_decimals_1,
                        ),
                    })
                    .unwrap(),
                )
                .await
                .unwrap();
        }
    });
}
```

价格更新的具体机制是：
1. 通过 Solana 的 PubSub 客户端监听池子状态变化：
```rust
pub async fn listen(
    &self,
    rpc: Arc<RpcClient>,
    raydium_program_id: Pubkey,
    pubsub: Arc<PubsubClient>,
    tx: broadcast::Sender<Pool>,
) -> anyhow::Result<UnsubscribeHandle> {
    // 订阅池子状态更新
    let (state, state_unsub) = pubsub
        .account_subscribe(&pool_id, Some(config.clone()))
        .await?;
    
    // 订阅 tick array bitmap 更新
    let (tick_array_bitmap, tick_array_bitmap_unsub) = pubsub
        .account_subscribe(&tick_array_bitmap_extension_pubkey, Some(config))
        .await?;

    // 当收到更新时，重新计算价格并广播
    while let Some((state, bitmap)) = hose.next().await {
        // ... 处理更新并广播新价格
    }
}
```

所以整个流程是：
1. 客户端可以通过 HTTP API 获取即时价格
2. 也可以通过 WebSocket 订阅价格更新
3. 后端通过监听 Raydium 池子状态来计算实时价格
4. 价格计算基于池子的当前状态（流动性、tick 等），而不是历史成交价格
5. 当池子状态发生变化时（比如有新的交易发生），会触发价格重新计算并推送给订阅的客户端

这种设计允许用户：
1. 随时获取最新价格（HTTP API）
2. 持续接收价格更新（WebSocket）
3. 基于最新价格执行交易（Swap API）
















根据项目的 README.md，我来帮你整理一下使用这个项目进行代币兑换的完整流程：

1. 首先需要启动服务：
```bash
# 1. 生成主密钥
MASTER_KEY=$(docker run raydium-price-feeder new masterkey)

# 2. 生成 JWT 密钥
JWT_SECERT=$(openssl rand -hex 32)

# 3. 创建钱包（有效期5分钟）
docker run raydium-price-feeder new wallet \
  --duration 5m \
  --masterkey $MASTER_KEY \
  --jwt-secret $JWT_SECERT \
  --database-url <PSQL_DATABASE_URL>

# 4. 启动服务器
docker run -p 8080:8080 raydium-price-feeder server \
  --masterkey $MASTER_KEY \
  --jwt-secret $JWT_SECERT \
  --rpc <RPC_NODE> \
  --database-url <PSQL_DATABASE_URL>
```

2. 执行 swap 交易：
```bash
# 设置变量
PAYER=<你的钱包地址>
# 以 WSOL-USDC 交易对为例
MINT_0=So11111111111111111111111111111111111111112  # WSOL
MINT_1=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v  # USDC

# 获取代币账户地址
INPUT_ACCOUNT=$(spl-token accounts $MINT_0 --owner $PAYER --output json-compact | jq .accounts[0].address)
OUTPUT_ACCOUNT=$(spl-token accounts $MINT_1 --owner $PAYER --output json-compact | jq .accounts[0].address)

# 设置交易参数
AMOUNT=0.5  # 想要交换的数量
SLIPPAGE=0.01  # 滑点 1%
FEE_INDEX=1  # 使用费率配置（参考 README 中的 fee index 说明）

# 发送交易请求
curl -XPOST "http://localhost:8080/api/swap" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <JWT_TOKEN>" \
  -d '{
    "mint0": "'$MINT_0'",
    "mint1": "'$MINT_1'",
    "input_account": "'$INPUT_ACCOUNT'",
    "output_account": "'$OUTPUT_ACCOUNT'",
    "amount": '$AMOUNT',
    "slippage": '$SLIPPAGE',
    "fee_index": '$FEE_INDEX',
    "simulate": true  # 先模拟交易
  }'
```

关于你的具体问题：

1. 是否可以直接调用 swap 而不获取价格？
- 是的，可以直接调用 swap 接口
- swap 接口内部会自动计算当前价格和兑换数量
- 但建议先获取价格，这样可以：
  1. 了解预期兑换数量
  2. 避免因价格波动导致的交易失败
  3. 更好地设置滑点参数

2. 重要参数说明：
- `fee_index`: 费率配置，不同的值对应不同的交易费率和流动性范围：
  ```javascript
  // 主网配置示例
  {
    index: 1,  // 适合大多数交易对
    tradeFeeRate: 2500,
    tickSpacing: 60,
    description: 'Best for most pairs'
  }
  ```
- `slippage`: 滑点容忍度，建议根据代币对的波动性设置
- `simulate`: 设置为 true 可以先模拟交易，不会真实执行

3. 执行流程建议：
```bash
# 1. 先用 simulate=true 模拟交易
curl -XPOST "http://localhost:8080/api/swap" ... -d '{"simulate": true, ...}'

# 2. 确认模拟结果后，改为实际交易
curl -XPOST "http://localhost:8080/api/swap" ... -d '{"simulate": false, ...}'
```

4. 常用代币地址（主网）：
```
USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
WSOL: So11111111111111111111111111111111111111112
WBTC: 3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh
WETH: 7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs
RAY:  4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R
```

注意事项：
1. 确保钱包中有足够的代币和 SOL（用于支付交易费）
2. 确保已创建对应代币的账户（ATA）
3. 建议先用小额测试
4. 注意检查 `fee_index` 的选择是否适合你的交易对
5. 交易前确保理解滑点参数的设置


















好的，我帮你整理一下直接使用 swap 方法进行代币兑换的步骤。根据 README.md，以 WSOL 兑换 USDC 为例：

1. 准备工作：
```bash
# 确保你有 Solana CLI 工具和 spl-token 工具
solana --version
spl-token --version

# 确保你的钱包中有足够的 SOL 和要兑换的代币
solana balance <你的钱包地址>
```

2. 检查代币账户：
```bash
# 设置变量
PAYER="你的钱包地址"
WSOL_MINT="So11111111111111111111111111111111111111112"
USDC_MINT="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"

# 检查你是否有这些代币的账户
spl-token accounts $WSOL_MINT --owner $PAYER
spl-token accounts $USDC_MINT --owner $PAYER

# 如果没有账户，需要创建
spl-token create-account $WSOL_MINT
spl-token create-account $USDC_MINT
```

3. 获取代币账户地址：
```bash
# 获取输入和输出代币账户地址
INPUT_ACCOUNT=$(spl-token accounts $WSOL_MINT --owner $PAYER --output json-compact | jq .accounts[0].address)
OUTPUT_ACCOUNT=$(spl-token accounts $USDC_MINT --owner $PAYER --output json-compact | jq .accounts[0].address)
```

4. 发送 swap 请求：
```bash
# 先模拟交易
curl -X POST "http://localhost:8080/api/swap" \
  -H "Content-Type: application/json" \
  -d '{
    "mint0": "'$WSOL_MINT'",
    "mint1": "'$USDC_MINT'",
    "input_account": "'$INPUT_ACCOUNT'",
    "output_account": "'$OUTPUT_ACCOUNT'",
    "amount": 0.1,           # 想要兑换的 WSOL 数量
    "slippage": 0.01,        # 1% 滑点
    "fee_index": 1,          # 使用标准费率配置
    "simulate": true         # 先模拟交易
  }'

# 如果模拟成功，执行实际交易
curl -X POST "http://localhost:8080/api/swap" \
  -H "Content-Type: application/json" \
  -d '{
    "mint0": "'$WSOL_MINT'",
    "mint1": "'$USDC_MINT'",
    "input_account": "'$INPUT_ACCOUNT'",
    "output_account": "'$OUTPUT_ACCOUNT'",
    "amount": 0.1,
    "slippage": 0.01,
    "fee_index": 1,
    "simulate": false        # 实际执行交易
  }'
```

重要参数说明：
1. `fee_index` 选择：
   - 1: 适合大多数交易对（费率 0.25%）
   - 4: 适合稳定币对（费率 0.01%）
   - 3: 适合波动较大的交易对（费率 1%）

2. `slippage`（滑点）建议：
   - 稳定币对：0.001-0.005 (0.1%-0.5%)
   - 普通代币对：0.01-0.02 (1%-2%)
   - 波动大的代币：0.02-0.05 (2%-5%)

3. `amount`：输入代币的数量，注意考虑代币精度

注意事项：
1. 确保钱包中有足够的：
   - 要兑换的代币
   - SOL（用于支付交易费）

2. 建议先：
   - 用小额测试
   - 使用 `simulate: true` 模拟交易
   - 确认模拟结果后再执行实际交易

3. 常见错误处理：
   - 如果账户不存在，需要创建代币账户
   - 如果余额不足，需要确保有足够的代币和 SOL
   - 如果滑点过小导致交易失败，可以适当增加滑点

4. 其他可用的代币对（主网）：
```
WBTC: 3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh
WETH: 7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs
RAY:  4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R
```

这样你就可以直接使用 swap 接口进行代币兑换，而不需要启动 price-feeder 服务。价格会在交易执行时自动从链上获取并计算。



你项目的 swap 相关调用是直接用 Rust crate（raydium-amm-v3）实现的，不需要本地 IDL 文件。
如果要换成自己的合约，通过自己的合约内部cpi调用raydium clmm协议的话，就需要导入自己合约的IDL文件。



在原来的项目基础上，做了如下修改：
1、删除了jwt
2、
3、删除了私钥的创建



**pool.rs代码总结：**
定义了 Pool 结构，封装了 Raydium Pool 的状态数据（amm_config、state、tick_array_bitmap_extension 等）

提供了 load 方法，从链上加载指定 pool 的相关账户和 TickArray

提供了 listen 方法，实时订阅 Pool 状态变更（state + bitmap），并通过 broadcast::Sender 通知外部

提供了 quote 方法，可以根据输入/输出方向和金额，快速估算兑换结果

提供了 swap 方法，真正发起兑换交易（还支持 simulate 预估模式）

包括 TickArray 加载的辅助函数 load_cur_and_next_five_tick_array

还有通用的 deserialize_anchor_account 反序列化 helper








**简单的swap业务流程，目前我正在用的：**
0、cd到项目根目录
1、cargo run -- server --rpc https://api.mainnet-beta.solana.com 执行命令启动swap服务
cargo run -- server --rpc https://virulent-holy-patron.solana-mainnet.quiknode.pro/e824506ae0771bd52773b78fe707f3f997b12148/
2、postman调用http://localhost:8080/api/swap接口，发送请求，json传入
WSOL购买MEME示例：
{
    "mint0": "So11111111111111111111111111111111111111112",
    "mint1": "bioJ9JTqW62MLz7UKHU69gtKhPpGi1BQhccj2kmSvUJ",
    "input_account": "4XBwhbL9dgAnkaMJ6GnQ2CeQnPjc2k5rHXjfiywRfvFr", // 用户WSOL ata
    "output_account": "8tP7Zmup1jxKvvJqZsFsPFx4McdpuF57Nu4aDUW36HgJ",// MEME ata
    "amount": 0.001,
    "slippage": 0.01,
    "fee_index": null,
    "pool_state": "4LuGwek6Jv4xpGvsQwZXonmLuRhrpHtmKVs95bN9EkTm",
    "simulate": true
}
代码内部，zero_for_one = true，is_base_input = true
MEME购买WSOL
{
    "mint0": "So11111111111111111111111111111111111111112",
    "mint1": "bioJ9JTqW62MLz7UKHU69gtKhPpGi1BQhccj2kmSvUJ",
    "input_account": "8tP7Zmup1jxKvvJqZsFsPFx4McdpuF57Nu4aDUW36HgJ", // MEME ata
    "output_account": "4XBwhbL9dgAnkaMJ6GnQ2CeQnPjc2k5rHXjfiywRfvFr",// WSOL ata
    "amount": 0.001,
    "slippage": 0.01,
    "fee_index": null,
    "pool_state": "4LuGwek6Jv4xpGvsQwZXonmLuRhrpHtmKVs95bN9EkTm",
    "simulate": true
}
代码内部，zero_for_one = false，is_base_input = true
mint0和mint1，根据地址大小来判断,不根据交易方向改变,传入的mint0和mint1是固定的。
input_account和output_account，来判断买卖方向。
当zero_for_one = true，is_base_input = true时，mint0兑换mint1，input_account是mint0，output_account是mint1
当zero_for_one = false，is_base_input = true时，mint1兑换mint0，input_account是mint1，output_account是mint0
input_account必须是出售代币的地址，output_account必须是购买代币的地址。




**zero_for_one 和 is_base_input 的区别：**
zero_for_one 决定交易方向（哪种代币是输入/输出）
is_base_input 决定金额的计算方式（输入固定还是输出固定）
is_base_input = true：用户指定输入代币的金额
is_base_input = false：用户指定输出代币的金额

每个 tick 数组有一个起始索引（start_tick_index），表示该数组覆盖的 tick 范围的起点。
current_vaild_tick_array_start_index 是某个已初始化的 tick 数组的 start_tick_index。
current_vaild_tick_array_start_index 是包含当前价格（或交易方向上第一个有效价格）的已初始化 tick 数组的起始 tick 索引，通常与 tick_current 紧密相关。

Pool 里的 tick_array 是“快照”或“缓存”
swap 里的 tick_array 是“最新链上状态”
所以swap.rs里，load方法里调用了一次load_cur_and_next_five_tick_array，swap方法里又调用了一次load_cur_and_next_five_tick_array

实际场景示例
假设一个池子包含：
token_mint_0 = USDC（有 0.1% 转账手续费）
token_mint_1 = SOL（无手续费）
案例 1：用户用 100 USDC 买 SOL
zero_for_one = true（USDC → SOL）（mint0兑换mint1）
is_base_input = true（用户支付 USDC）
​手续费​：从 100 USDC 中扣除 0.1 USDC（剩余 99.9 USDC 用于交易）。
案例 2：用户用 1 SOL 买 USDC
zero_for_one = false（SOL → USDC）（mint1兑换mint0）
is_base_input = true（用户支付 SOL）
​手续费​：SOL 无手续费，所以 transfer_fee = 0。





**特别注意的点：**
#1、fee_index 费率配置，不同的值对应不同的交易费率和流动性范围，这个参数要设置下
#2、zero_for_one: bool：交易方向，true 表示 token0 换 token1（价格下降），false 表示 token1 换 token0（价格上升）。
#3、slippage: f64：滑点，表示允许的价格波动范围，单位为百分比。根据滑点计算出other_amount_threshold
#4、simulate: bool：是否模拟交易，true 表示模拟交易，false 表示实际交易。
#5、is_base_input: bool：是否是基础代币输入，true 表示输入代币是基础代币，false 表示输入代币是报价代币。
#6、加载6个 tick array，针对大额订单。如果小额订单可以只加载3个。
#7、set_compute_unit_limit，swap.rs代码里没有考虑到优先费priority fee设置
#8、设置手续费transfer_fee
#9、交易的4个入参
​**amount**​：明确交易规模
​**other_amount_threshold**​：保护用户免受意外损失（该值由滑点和报价计算而来）
​**sqrt_price_limit_x64**​：防止在极端市场条件下成交 （它阻止你在“某个价格之外”成交，哪怕你 other_amount_threshold 还没被触发，它和other_amount_threshold是互补的，形成了双重保护）
​**is_base_input**​：支持两种交易策略（求购 or 报价）

上面特别要注意的点里，前端传给我们的入参如下：
{
    "mint0": "So11111111111111111111111111111111111111112",
    "mint1": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "input_account": "5XdzbBN65VVkoY7QbfPeM2iyx6rJwg98hQ49YYf7GX7U",
    "output_account": "APrqwKGs54tMf1JwgUEEnBUjxoMdBR8NXtd7hoCZ9kTq",
    "amount": 0.001,
    "slippage": 0.01,
    "fee_index": 4,
    "simulate": true
}
建议：
1、接口应增加 is_base_input 字段，让前端传递，后端直接用。
2、接口应增加 sqrt_price_limit_x64 字段，让前端传递，后端直接用。
目前这两个值，都在swap.rs代码里写死了，is_base_input = true，sqrt_price_limit_x64 = 0
3、
根据poolState也就是根据池子信息，判断哪一个是mint0，哪一个是mint1。mint0和mint1是固定的，根买卖交易方向没有关系。
input_account和output_account，来判断买卖方向。
// 用 WSOL 买 MEME
{
    "mint0": "So11111111111111111111111111111111111111112",
    "mint1": "<MEME地址>",
    "input_account": "<WSOL账户>",  // 输入是 WSOL
    "output_account": "<MEME账户>",  // 输出是 MEME
    "amount": 0.1,
    "slippage": 0.05,
    "fee_index": 3,
}

// 用 MEME 买 WSOL
{
    "mint0": "So11111111111111111111111111111111111111112",
    "mint1": "<MEME地址>",
    "input_account": "<MEME账户>",  // 输入是 MEME
    "output_account": "<WSOL账户>",  // 输出是 WSOL
    "amount": 1000000,
    "slippage": 0.05,
    "fee_index": 3,
}

**如何调用swap接口**
有两种调用swap接口的入参方式：
第一：传入fee_index，通过fee_index计算出ammConfig地址，再通过ammConfig地址计算出pool_state
{
    "mint0": "So11111111111111111111111111111111111111112",
    "mint1": "sZKsQpqaHTdiQMx2JEFCNjoUAQqAqw9g4948ZqWpump",
    "input_account": "4XBwhbL9dgAnkaMJ6GnQ2CeQnPjc2k5rHXjfiywRfvFr", // 用户WSOL ata
    "output_account": "DApf6dASShYeorNAkn82nnf4AZZDxtBxZV8yfndGgXLY",// 用户USDC ata
    "amount": 0.001,
    "slippage": 0.01,
    "fee_index": 4,
    "pool_state": null,
    "simulate": true
}
第二：传入pool_state，表示指定池子，通过pool_state计算出ammConfig地址
{
    "mint0": "So11111111111111111111111111111111111111112",
    "mint1": "sZKsQpqaHTdiQMx2JEFCNjoUAQqAqw9g4948ZqWpump",
    "input_account": "4XBwhbL9dgAnkaMJ6GnQ2CeQnPjc2k5rHXjfiywRfvFr", // 用户WSOL ata
    "output_account": "DApf6dASShYeorNAkn82nnf4AZZDxtBxZV8yfndGgXLY",// 用户USDC ata
    "amount": 0.001,
    "slippage": 0.01,
    "fee_index": null,
    "pool_state": "3RiyGWnn48Zdp3XxqrrPpi44TrkR7JZUE6vt5ozaV37L",
    "simulate": true
}





对上面的解释：
**第一：fee_index**
https://api-v3.raydium.io/main/clmm-config 通过该接口可以获取不同的fee_index对应的费率
fee_index如何选择？？
fee_index 不是你随便选的，而是要根据你实际要用的池子来选。先查池子信息，拿到 fee_index，再传给 swap 接口，这样才能保证交易走的是你想要的那个池子和费率。

| 字段名                   | 说明                                                                                 |
| --------------------- | ---------------------------------------------------------------------------------- |
| **id**                | 流动性池的唯一标识符，通常是池的地址或主键。                                                             |
| **index**             | 该池的排序索引，可能用于前端显示顺序或内部优先级处理。                                                        |
| **protocolFeeRate**   | **协议费率**，单位为 1e6，例如 `120000` 表示 12%。这是平台收取的总费用中归协议（比如开发方或治理）的比例。                   |
| **tradeFeeRate**      | **交易费率**，单位也是 1e6，比如 `100` 表示 0.01%。这是用户进行 swap 时总的交易费用率。                          |
| **fundFeeRate**       | **基金费用率**，可能是奖励池、保险基金、激励基金等收取的额外费率，`40000` 就是 4%。                                  |
| **tickSpacing**       | tick 间距，通常用于 Uniswap v3 类型的精细化流动性池，控制价格刻度的精度，`1` 表示最小间隔。                           |
| **defaultRange**      | 默认的价格区间（百分比），表示建议用户提供流动性的价格范围宽度，例如 0.001 表示 ±0.1%。                                 |
| **defaultRangePoint** | 一个数组，定义了一组常用或推荐的 liquidity range 值（例如 ±0.1%、±0.3%、±0.5%、±0.8%、±1%）。适合做 UI 下拉菜单预设值。 |


Bob 发起交易：用 100 USDC 购买 Token
用到的参数：
tradeFeeRate = 100 → 交易费为 0.01%

总交易费用 = 100 * 0.01% = 0.01 USDC

protocolFeeRate = 120000（12%）和 fundFeeRate = 40000（4%）→ 这两个从上面 0.01 USDC 中划分：

项目	金额（USDC）
总费用	0.01
协议收益	0.01 * 12% = 0.0012
基金收益	0.01 * 4% = 0.0004
剩余给 LP 的收益	0.01 - 上面两项 = 0.0084

所以 大多数费用（84%）还是给 LP 的，其余分给协议和基金。


| 项目         | 金额（USDC）             |
| ---------- | -------------------- |
| 总费用        | 0.01                 |
| 协议收益       | 0.01 \* 12% = 0.0012 |
| 基金收益       | 0.01 \* 4% = 0.0004  |
| 剩余给 LP 的收益 | 0.01 - 上面两项 = 0.0084 |


| `tradeFeeRate` 表示的真实费率 | 备注                          fee_index    |
| ---------------------- | ------------------------------- |
| `100` → 0.01%          | 超低费用，适合稳定币对（如 USDC/USDT）      4  |
| `300` → 0.03%          | 常规费率，适合主流资产对（如 ETH/USDC）      7  |
| `1000` → 0.1%          | 较高费用，适合波动资产（如 MEME/USDC）       10 |
| `10000` → 1.0%         | 非常高费用，适合小币种、流动性低的币对或反狙击池       3 |
| `40000` → 4.0%         | 极高费用，一般用于防止机器人套利的“反 MEV 池”或特殊策略  19 |


**第二：token0、token1、zero_for_one、is_base_input 怎么使用**

Step 1️⃣：比较地址确定 token0 和 token1
if (tokenA < tokenB) {
  token0 = tokenA;
  token1 = tokenB;
} else {
  token0 = tokenB;
  token1 = tokenA;
}
Step 2️⃣：判断交易方向（zero_for_one）
| 你想兑换方向          | zero\_for\_one |
| --------------- | -------------- |
| token0 → token1 | `true`         |
| token1 → token0 | `false`        |
Step 3️⃣：判断输入模式（is_base_input）
| 你控制哪个数量         | is\_base\_input |
| --------------- | --------------- |
| 控制输入的数量（我要花多少）  | `true`          |
| 控制输出的数量（我要拿到多少） | `false`         |

✅ 举个完整例子：我用 SOL 买 MEME，愿意最多出 2 SOL
假设：

SOL = tokenA

MEME = tokenB

SOL 地址 < MEME 地址

判断：

token0 = SOL, token1 = MEME

方向是 SOL → MEME → zero_for_one = true

你控制的是输入数量 2 SOL → is_base_input = true


✅ 再举一个反例：我想卖掉 500 MEME 换成 SOL（你知道你要卖的量）
token0 = SOL, token1 = MEME

方向是 MEME → SOL → zero_for_one = false

你控制的是输入数量（卖的 MEME）→ is_base_input = true

✅ 再再举一例：我想换到 1 SOL，愿意出多少 MEME 都行（控制输出）
token0 = SOL, token1 = MEME

方向是 MEME → SOL → zero_for_one = false

你控制的是输出数量（想拿到 1 SOL）→ is_base_input = false


✅ 如何推导交易方向，没有必要显式指明zero_for_one？
核心推导规则：
交易方向（zero_for_one） = 
    如果 is_base_input 为 true：
        base_token 是 token0 → zero_for_one = true
        base_token 是 token1 → zero_for_one = false
    如果 is_base_input 为 false：
        quote_token 是 token0 → zero_for_one = true
        quote_token 是 token1 → zero_for_one = false



**第九：swap的四个入参**

| 参数                       | 类型     | 说明                                                                                | 如何设置（举例）                                                                                                                                             |
| ------------------------ | ------ | --------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `amount`                 | `u64`  | 表示用户想要交易的数量。是“输入数量”还是“最小接收数量”由 `is_base_input` 决定                                 | - 若买入 MEME（输入 SOL），`amount = 输入的 SOL 数量`（比如 1 SOL = 1\_000\_000\_000 lamports）<br>- 若卖出 MEME（输入 MEME），`amount = 输入的 MEME 数量`                         |
| `other_amount_threshold` | `u64`  | 表示 **你期望的最小输出数量（buy 模式）** 或 **你最多接受的输入成本（sell 模式）**，用于滑点保护                        | - 买 MEME（你提供 SOL），希望至少拿到 500 MEME：`other_amount_threshold = 500_000_000`<br>- 卖 MEME（你提供 MEME），不希望花超过 1 SOL：`other_amount_threshold = 1_000_000_000` |
| `sqrt_price_limit_x64`   | `u128` | 控制价格上下限（x64 格式）。设置后，交易不会跨越这个价格极限。用来限制价格穿透（避免攻击）                                   | - 通常设置为 0 表示无价格限制（⚠️风险）<br>- 设置为报价返回的 sqrt\_price ± 滑点范围后的值，可限制价格冲击                                                                                  |
| `is_base_input`          | `bool` | 控制 `amount` 是输入币数量（true），还是目标币数量（false）<br> true = 市价单（出多少钱），false = 限价单（想要拿到多少钱） | - 用 1 SOL 买 MEME → `is_base_input = true`<br>- 我想要拿到 1000 MEME → `is_base_input = false`（让系统算我最多出多少 SOL）                                             |

✅ 场景一：你用 1 SOL 买 MEME（市价单）
| 参数                       | 值                          |
| ------------------------ | -------------------------- |
| `amount`                 | 1\_000\_000\_000（1 SOL）    |
| `other_amount_threshold` | 500\_000\_000（至少 500 MEME） |
| `sqrt_price_limit_x64`   | 0（或报价返回值 ± 滑点）             |
| `is_base_input`          | `true`                     |


✅ 场景二：你想拿到 1000 MEME，不超过 1.1 SOL 成本（限价）
| 参数                       | 值                                |
| ------------------------ | -------------------------------- |
| `amount`                 | 1\_000\_000\_000（期望拿到 1000 MEME） |
| `other_amount_threshold` | 1\_100\_000\_000（最多花 1.1 SOL）    |
| `sqrt_price_limit_x64`   | 报价结果限制                           |
| `is_base_input`          | `false`                          |


👇 举个例子：
你打算用 1 SOL 买 MEME，当前报价是：

1 SOL = 1000 MEME

你容忍最多 1% 滑点（即最少拿到 990 MEME）

那么你要传的参数是：

| 参数                       | 值                            |
| ------------------------ | ---------------------------- |
| `amount`                 | `1_000_000_000`（输入的 SOL 数量）  |
| `other_amount_threshold` | `990_000_000`（最少收到 990 MEME） |
| `is_base_input`          | `true`                       |

验证是否满足滑点
✅ 滑点计算公式（对于 is_base_input = true）
即：你提供输入币（amount），希望获得输出币（至少多少）
min_output = expected_output * (1 - slippage_percent)
| 变量                 | 含义                                                     |
| ------------------ | ------------------------------------------------------ |
| `expected_output`  | 预期拿到的代币数量（通过 off-chain 报价预估）                           |
| `slippage_percent` | 你容忍的滑点（如 1% = 0.01）                                    |
| `min_output`       | 设置给合约的 `other_amount_threshold`，表示你**最少要拿到的数量**，否则交易回滚 |
✅ 举例：
想用 1 SOL 买 MEME

报价显示 1 SOL ≈ 1000 MEME

滑点设置为 1%
min_output = 1000 * (1 - 0.01) = 990
你要传给合约的：
{
  "amount": 1000000000,                      // 1 SOL（base input）
  "other_amount_threshold": 990000000,       // 最少收到 990 MEME（quote）
  "is_base_input": true,
  ...
}

✅ 反过来，如果你设置的是 is_base_input = false（我要买 1000 MEME，要花最多多少 SOL？）
公式如下：
max_input = expected_input * (1 + slippage_percent)
报价：1000 MEME ≈ 1 SOL

滑点 1%
max_input = 1 * (1 + 0.01) = 1.01 SOL
你要传给合约的：
{
  "amount": 1000000000,                      // 1000 MEME（quote input）
  "other_amount_threshold": 1010000000,      // 最多花费 1.01 SOL（base output）
  "is_base_input": false,
  ...
}
总之：滑点是一个 UI 层/调用层的概念，最终还是会被转换为具体的 amount 和 other_amount_threshold。

**第八：设置手续费transfer_fee**
只有 Token-2022 标准、且 mint 配置了 transfer_fee 的代币，转账时才会自动收手续费。
正是因为这个规定，所有才有了手续费计算的逻辑：

let zero_for_one = user_input_state.base.mint == self.state.token_mint_0
    && user_output_state.base.mint == self.state.token_mint_1;

let transfer_fee = if is_base_input {
    if zero_for_one {
        get_transfer_fee(&mint0_state, epoch, amount)
    } else {
        get_transfer_fee(&mint1_state, epoch, amount)
    }
} else {
    0
};
// 计算 amount_specified，这个值就是用户输入的金额，减去手续费，只有spltoekn 2022代币才有手续费
let amount_specified = amount.checked_sub(transfer_fee).unwrap();


**第七：set_compute_unit_limit**

// 因为只是返回账户给第三方，所有这个计算单元限制可以设置并没有用到。
let request_inits_instr = ComputeBudgetInstruction::set_compute_unit_limit(1_400_000_u32);
instructions.push(request_inits_instr);


**第三：slippage滑点，根据滑点计算出other_amount_threshold**
// 计算基础输出值other_amount_threshold，根据当前池子信息，计算swap最终收到的多少目标代币，还没有考虑滑点
let (mut other_amount_threshold, tick_array_indexs) =
    utils::get_out_put_amount_and_remaining_accounts(
        amount_specified,
        sqrt_price_limit_x64,
        zero_for_one,
        is_base_input,
        &self.amm_config,
        &self.state,
        &self.tick_array_bitmap_extension,
        &mut tick_arrays,
    )
    .unwrap();

// 考虑滑点和spltoken 2022代币的手续费之后，计算最终的输出值other_amount_threshold
if is_base_input {
    // calc mint out amount with slippage
    other_amount_threshold = amount_with_slippage(other_amount_threshold, slippage, false);
} else {
    // calc max in with slippage
    other_amount_threshold = amount_with_slippage(other_amount_threshold, slippage, true);
    // calc max in with transfer_fee
    let transfer_fee = if zero_for_one {
        utils::get_transfer_inverse_fee(&mint0_state, epoch, other_amount_threshold)
    } else {
        utils::get_transfer_inverse_fee(&mint1_state, epoch, other_amount_threshold)
    };
    other_amount_threshold += transfer_fee;
}




**监听池子信息 pubsub.rs**
注意下，监听池子信息，传入的zero_for_one=true，默认是true，如果需要改成false，需要注意下。








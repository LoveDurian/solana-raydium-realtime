use std::str::FromStr;
use std::fs;

use actix_web::{
    post,
    web::{self, Json},
};
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use spl_token::ui_amount_to_amount;
use solana_sdk::signature::EncodableKey;

use crate::{
    constants::RAYDIUM_CLMM_PUBKEY,
    extractors::account::AuthorizationGuard,
    models::account::Account,
    solclient::{
        pool::{LoadPoolAccounts, Pool},
        utils::TransactionResult,
    },
    state::AppState,
};

const DISCRIMINATOR_SIZE: usize = 8;
const BUMP_SIZE: usize = 1;
const PUBKEY_SIZE: usize = 32;

#[derive(Deserialize)]
struct SwapBody {
    mint0: String,
    mint1: String,
    input_account: String,
    output_account: String,
    amount: f64,
    slippage: f64,
    fee_index: Option<u16>,
    pool_state: Option<String>,
    simulate: bool,
    zero_for_one: bool,
    fee_bps: Option<u64>,
}

#[post("/swap")]
async fn swap_route(
    state: web::Data<AppState>,
    AuthorizationGuard(account_id): AuthorizationGuard,
    body: web::Json<SwapBody>,
) -> actix_web::error::Result<Json<TransactionResult>> {
    // let payer = Account::find_one(&state.pool, account_id, &state.masterkey)
    // .await
    // .map_err(actix_web::error::ErrorBadGateway)?
    // .keypair;

    let keypair_file = "/Users/kevin/.config/solana/id_localnet.json";
    let payer = solana_sdk::signature::Keypair::read_from_file(keypair_file)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Failed to parse keypair: {}", e)))?;
    println!("payer: {:?}", payer);

    let mint0 = Pubkey::from_str(&body.mint0).unwrap();
    let mint1 = Pubkey::from_str(&body.mint1).unwrap();

    // 执行了raydium clmm合约地址
    let raydium_amm_v3 = Pubkey::from_str(RAYDIUM_CLMM_PUBKEY).unwrap();

    let (amm_config_key, pool_state) = match (body.fee_index, &body.pool_state) {
        (Some(fee_index), None) => {
            let (amm_config_key, _) = Pubkey::find_program_address(
                &[
                    raydium_amm_v3::states::AMM_CONFIG_SEED.as_bytes(),
                    &fee_index.to_be_bytes(),
                ],
                &raydium_amm_v3,
            );
            
            let (pool_state, _) = Pubkey::find_program_address(
                &[
                    raydium_amm_v3::states::POOL_SEED.as_bytes(),
                    amm_config_key.as_ref(),
                    mint0.as_ref(),
                    mint1.as_ref(),
                ],
                &raydium_amm_v3,
            );
            
            (amm_config_key, pool_state)
        },
        (None, Some(pool_state_str)) => {
            let pool_state = Pubkey::from_str(pool_state_str)
                .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid pool state address: {}", e)))?;
            println!("pool_state: {:?}", pool_state);
            let pool_account = state.rpc.get_account(&pool_state).await
        .map_err(|e| actix_web::error::ErrorBadGateway(format!("Failed to fetch pool account: {}", e)))?;
    
            println!("Data length: {}", pool_account.data.len());
            
            // 1. 计算 ammConfig 的偏移量和数据
            let amm_config_offset = DISCRIMINATOR_SIZE + BUMP_SIZE;
            let amm_config_data = &pool_account.data[amm_config_offset..amm_config_offset + PUBKEY_SIZE];
            let amm_config_key = Pubkey::new(amm_config_data);
            println!("amm_config_key: {:?}", amm_config_key);

            (amm_config_key, pool_state)
        },
        _ => return Err(actix_web::error::ErrorBadRequest("Must provide either fee_index or pool_state")),
    };

    let (tick_array_bitmap_extension, _) = Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
            pool_state.as_ref(),
        ],
        &raydium_amm_v3,
    );
    println!("tick_array_bitmap_extension: {:?}", tick_array_bitmap_extension);
    let pool = Pool::load(
        state.rpc.clone(),
        &state.raydium_program_id,
        LoadPoolAccounts {
            mint0,
            mint1,
            amm_config: amm_config_key,
            pool_id: pool_state,
            tickarray_bitmap_extension: tick_array_bitmap_extension,
        },
        body.zero_for_one,
    )
    .await
    .map_err(actix_web::error::ErrorBadGateway)?;

    // let input_token = get_associated_token_address(&payer.pubkey(), &mint0);
    // let output_token = get_associated_token_address(&payer.pubkey(), &mint1);
    let input_token =
        Pubkey::from_str(&body.input_account).map_err(actix_web::error::ErrorBadRequest)?;
    let output_token =
        Pubkey::from_str(&body.output_account).map_err(actix_web::error::ErrorBadRequest)?;

    let res = pool
        .swap(
            state.rpc.clone(),
            &state.raydium_program_id,
            payer,
            input_token,
            output_token,
            ui_amount_to_amount(body.amount, pool.state.mint_decimals_0),
            body.slippage,
            true, // TODO: Would be nice to implement body.is_base_input,
            body.simulate,
            body.zero_for_one,
            body.fee_bps,
        )
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    Ok(Json(res))
}

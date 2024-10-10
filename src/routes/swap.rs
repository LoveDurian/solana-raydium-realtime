use std::str::FromStr;

use actix_web::{
    post,
    web::{self, Json},
};
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use spl_token::ui_amount_to_amount;

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

#[derive(Deserialize)]
struct SwapBody {
    mint0: String,
    mint1: String,
    input_account: String,
    output_account: String,
    amount: f64,
    slippage: f64,
    fee_index: u16,
    simulate: bool,
}

#[post("/swap")]
async fn swap_route(
    state: web::Data<AppState>,
    AuthorizationGuard(account_id): AuthorizationGuard,
    body: web::Json<SwapBody>,
) -> actix_web::error::Result<Json<TransactionResult>> {
    let payer = Account::find_one(&state.pool, account_id, &state.masterkey)
        .await
        .map_err(actix_web::error::ErrorBadGateway)?
        .keypair;

    let mint0 = Pubkey::from_str(&body.mint0).unwrap();
    let mint1 = Pubkey::from_str(&body.mint1).unwrap();

    let raydium_amm_v3 = Pubkey::from_str(RAYDIUM_CLMM_PUBKEY).unwrap();

    let (amm_config_key, _) = Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::AMM_CONFIG_SEED.as_bytes(),
            &body.fee_index.to_be_bytes(),
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
    let (tick_array_bitmap_extension, _) = Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
            pool_state.as_ref(),
        ],
        &raydium_amm_v3,
    );

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
        )
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    Ok(Json(res))
}

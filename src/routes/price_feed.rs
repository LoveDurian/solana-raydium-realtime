use std::str::FromStr;

use actix_web::{
    get,
    web::{self, Json},
    Responder,
};
use solana_sdk::pubkey::Pubkey;
use spl_token::{amount_to_ui_amount_string, ui_amount_to_amount};

use crate::{
    extractors::account::AuthorizationGuard,
    routes::{PriceFeedQuery, PriceFeedResponse},
    state::AppState,
};

#[get("/price-feed")]
async fn price_feed_pooling(
    query: web::Query<PriceFeedQuery>,
    state: web::Data<AppState>,
    _auth: AuthorizationGuard,
) -> impl Responder {
    let mint0 = Pubkey::from_str(&query.mint0).unwrap();
    let mint1 = Pubkey::from_str(&query.mint1).unwrap();

    // Get current state and wait till update
    let mut broadcast = state.broadcast.lock().await;
    let mut subscription = broadcast
        .subscribe(&state.raydium_program_id, mint0, mint1, query.fee_index)
        .await
        .unwrap();
    drop(broadcast);

    let pool = subscription.recv().await.unwrap();

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

    Json(PriceFeedResponse {
        amount_out: amount_to_ui_amount_string(amount_out, pool.state.mint_decimals_1),
    })
}

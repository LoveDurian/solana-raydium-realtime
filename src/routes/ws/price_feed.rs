use std::str::FromStr;

use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use solana_sdk::pubkey::Pubkey;
use spl_token::{amount_to_ui_amount_string, ui_amount_to_amount};

use crate::{
    extractors::account::AuthorizationGuard,
    routes::{PriceFeedQuery, PriceFeedResponse},
    state::AppState,
};

#[get("/ws/price-feed")]
pub async fn stream_price_feed(
    req: HttpRequest,
    query: web::Query<PriceFeedQuery>,
    state: web::Data<AppState>,
    _auth: AuthorizationGuard,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (res, mut session, _) = actix_ws::handle(&req, stream)?;

    let mint0 = Pubkey::from_str(&query.mint0).unwrap();
    let mint1 = Pubkey::from_str(&query.mint1).unwrap();

    let mut broadcast = state.broadcast.lock().await;
    let mut subscription = broadcast
        .subscribe(&state.raydium_program_id, mint0, mint1, query.fee_index)
        .await
        .unwrap();
    drop(broadcast);

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
    // respond immediately with response connected to WS session
    Ok(res)
}

pub mod price_feed;
pub mod swap;
pub mod ws;

use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PriceFeedQuery {
    mint0: String,
    mint1: String,
    fee_index: u16,
}

#[derive(Serialize)]
struct PriceFeedResponse {
    amount_out: String,
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(price_feed::price_feed_pooling);
    cfg.service(swap::swap_route);
    cfg.service(ws::price_feed::stream_price_feed);
}

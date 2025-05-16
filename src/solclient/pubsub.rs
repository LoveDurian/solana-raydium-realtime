use std::{collections::HashMap, str::FromStr, sync::Arc};

use solana_client::nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient};
use solana_sdk::pubkey::Pubkey;
use tokio::sync::broadcast;

use crate::constants::RAYDIUM_CLMM_PUBKEY;

use super::pool::{LoadPoolAccounts, Pool, UnsubscribeHandle};

#[derive(Clone)]
pub struct ClmmBroadcast {
    pub rpc: Arc<RpcClient>,
    pub pubsub: Arc<PubsubClient>,
    pub subscriptions: HashMap<Pubkey, Subscription>,
}

#[derive(Clone)]
pub struct Subscription(pub broadcast::Sender<Pool>, pub Box<Arc<UnsubscribeHandle>>);

impl ClmmBroadcast {
    pub fn new(rpc: Arc<RpcClient>, pubsub: Arc<PubsubClient>) -> Self {
        Self {
            rpc,
            pubsub,
            subscriptions: HashMap::new(),
        }
    }

    pub async fn subscribe(
        &mut self,
        raydium_program_id: &Pubkey,
        mint0: Pubkey,
        mint1: Pubkey,
        fee_index: u16,
    ) -> anyhow::Result<broadcast::Receiver<Pool>> {
        let raydium_amm_v3 = Pubkey::from_str(RAYDIUM_CLMM_PUBKEY).unwrap();

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
        let (tick_array_bitmap_extension, _) = Pubkey::find_program_address(
            &[
                raydium_amm_v3::states::POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                pool_state.as_ref(),
            ],
            &raydium_amm_v3,
        );

        if self.subscriptions.contains_key(&pool_state) {
            return Ok(self.subscriptions.get(&pool_state).unwrap().0.subscribe());
        }

        let pool = Pool::load(
            self.rpc.clone(),
            raydium_program_id,
            LoadPoolAccounts {
                mint0,
                mint1,
                amm_config: amm_config_key,
                pool_id: pool_state,
                tickarray_bitmap_extension: tick_array_bitmap_extension,
            },
            true,
        )
        .await?;
        let (tx, rx) = broadcast::channel(10);
        // tx.send(pool.clone());

        let unsub = pool
            .listen(
                self.rpc.clone(),
                *raydium_program_id,
                self.pubsub.clone(),
                tx.clone(),
            )
            .await?;
        self.subscriptions
            .insert(pool_state, Subscription(tx, Box::new(Arc::new(unsub))));

        Ok(rx)
    }
}

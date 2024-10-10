use std::sync::Arc;

use solana_client::nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::{error::Result, solclient::pubsub::ClmmBroadcast};

pub type Signer = Arc<Keypair>;

#[derive(Clone)]
pub struct AppState {
    pub rpc: Arc<RpcClient>,
    pub broadcast: Arc<Mutex<ClmmBroadcast>>,
    pub pool: PgPool,
    pub jwt_secret: String,
    pub masterkey: Vec<u8>,
    pub raydium_program_id: Pubkey,
}

impl AppState {
    pub async fn new(
        solana_url: String,
        pool: PgPool,
        jwt_secret: String,
        masterkey: Vec<u8>,
        raydium_program_id: Pubkey,
    ) -> Result<Self> {
        let pubsub =
            Arc::new(PubsubClient::new(&solana_url.as_str().replace("https://", "wss://")).await?);

        let rpc = Arc::new(RpcClient::new_with_commitment(
            solana_url,
            CommitmentConfig {
                commitment: solana_sdk::commitment_config::CommitmentLevel::Finalized,
            },
        ));

        let broadcast = Arc::new(Mutex::new(ClmmBroadcast::new(rpc.clone(), pubsub)));

        Ok(Self {
            rpc,
            broadcast,
            pool,
            jwt_secret,
            masterkey,
            raydium_program_id,
        })
    }
}

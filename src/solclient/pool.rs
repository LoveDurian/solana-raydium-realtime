use std::{collections::VecDeque, future::Future, pin::Pin, str::FromStr, sync::Arc};

use anchor_client::{Client, Cluster};
use anchor_lang::{prelude::AccountMeta, AccountDeserialize};
use arrayref::array_ref;
use futures::{future::Join, StreamExt};
use raydium_amm_v3::states::{
    AmmConfig, PoolState, TickArrayBitmapExtension, TickArrayState, POOL_TICK_ARRAY_BITMAP_SEED,
};
use solana_client::nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient};
use solana_sdk::{
    account::Account,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    compute_budget::ComputeBudgetInstruction,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use spl_token_2022::{
    extension::{
        transfer_fee::TransferFeeConfig, BaseState, BaseStateWithExtensions, StateWithExtensionsMut,
    },
    state::{Account as TokenAccount, Mint},
};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

use crate::solclient::utils::{amount_with_slippage, swap_v2_instr};

use super::utils::{self, TransactionResult};

#[derive(Clone)]
pub struct Pool {
    id: Pubkey,
    mint0: Vec<u8>,
    mint1: Vec<u8>,
    amm_config: AmmConfig,
    pub state: PoolState,
    tick_array: VecDeque<TickArrayState>,
    tick_array_bitmap_extension: TickArrayBitmapExtension,
}

pub struct LoadPoolAccounts {
    pub amm_config: Pubkey,
    pub pool_id: Pubkey,
    pub tickarray_bitmap_extension: Pubkey,
    pub mint0: Pubkey,
    pub mint1: Pubkey,
}

type UnsubscribeFn =
    Join<Pin<Box<dyn Future<Output = ()> + Send>>, Pin<Box<dyn Future<Output = ()> + Send>>>;

pub struct UnsubscribeHandle {
    handle: JoinHandle<std::result::Result<(), anyhow::Error>>,
    rx: mpsc::Receiver<UnsubscribeFn>,
}

impl UnsubscribeHandle {
    pub async fn unsubscribe(mut self) {
        if let Some(unsubscribe) = self.rx.recv().await {
            unsubscribe.await;
        }

        let _ = self.handle.await;
    }
}

impl std::fmt::Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pool")
            .field("id", &self.id)
            .field("mint0", &self.mint0)
            .field("mint1", &self.mint1)
            .field("amm_config", &self.amm_config)
            .field("state", &self.state)
            .field(
                "tick_array_bitmap_extension",
                &self.tick_array_bitmap_extension,
            )
            .finish()
    }
}

impl Pool {
    pub async fn load(
        rpc: Arc<RpcClient>,
        raydium_program_id: &Pubkey,
        accounts: LoadPoolAccounts,
    ) -> anyhow::Result<Self> {
        // load mult account
        let load_accounts = vec![
            accounts.amm_config,
            accounts.pool_id,
            accounts.tickarray_bitmap_extension,
            accounts.mint0,
            accounts.mint1,
        ];
        let rsps = rpc.get_multiple_accounts(&load_accounts).await?;
        dbg!(&rsps);
        let [amm_config_account, pool_account, tickarray_bitmap_extension_account, mint0_account, mint1_account] =
            array_ref![rsps, 0, 5];

        let mint0_data = mint0_account.clone().unwrap().data;
        let mint1_data = mint1_account.clone().unwrap().data;
        let amm_config_state = deserialize_anchor_account::<raydium_amm_v3::states::AmmConfig>(
            amm_config_account.as_ref().unwrap(),
        )?;
        let pool_state = deserialize_anchor_account::<raydium_amm_v3::states::PoolState>(
            pool_account.as_ref().unwrap(),
        )?;
        let tickarray_bitmap_extension =
            deserialize_anchor_account::<raydium_amm_v3::states::TickArrayBitmapExtension>(
                tickarray_bitmap_extension_account.as_ref().unwrap(),
            )?;

        let zero_for_one = true;

        // load tick_arrays
        let tick_arrays = load_cur_and_next_five_tick_array(
            &rpc,
            raydium_program_id,
            &accounts.pool_id,
            &pool_state,
            &tickarray_bitmap_extension,
            zero_for_one,
        )
        .await?;

        Ok(Self {
            id: accounts.pool_id,
            mint0: mint0_data,
            mint1: mint1_data,
            amm_config: amm_config_state,
            state: pool_state,
            tick_array: tick_arrays,
            tick_array_bitmap_extension: tickarray_bitmap_extension,
        })
    }

    pub async fn listen(
        &self,
        rpc: Arc<RpcClient>,
        raydium_program_id: Pubkey,
        pubsub: Arc<PubsubClient>,
        tx: broadcast::Sender<Pool>,
    ) -> anyhow::Result<UnsubscribeHandle> {
        let pool_id = self.id;
        let mint0 = self.mint0.clone();
        let mint1 = self.mint1.clone();
        let amm_config = self.amm_config.clone();

        let (shutdown_tx, rx) = mpsc::channel::<_>(2);

        let handle = tokio::spawn(async move {
            let config = solana_client::rpc_config::RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64Zstd),
                data_slice: None,
                commitment: Some(solana_sdk::commitment_config::CommitmentConfig {
                    commitment: CommitmentLevel::Processed,
                }),
                min_context_slot: None,
            };
            let (state, state_unsub) = pubsub
                .account_subscribe(&pool_id, Some(config.clone()))
                .await?;
            let (tick_array_bitmap_extension_pubkey, _) = Pubkey::find_program_address(
                &[POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(), pool_id.as_ref()],
                &raydium_program_id,
            );
            let (tick_array_bitmap, tick_array_bitmap_unsub) = pubsub
                .account_subscribe(&tick_array_bitmap_extension_pubkey, Some(config))
                .await?;
            let unsub = futures::future::join(state_unsub(), tick_array_bitmap_unsub());
            shutdown_tx
                .send(unsub)
                .await
                .expect("Cannot send unsub handler to shutdown_tx, this should never happen");
            let mut hose = state.zip(tick_array_bitmap);

            while let Some((state, bitmap)) = hose.next().await {
                let pool_state = deserialize_anchor_account::<raydium_amm_v3::states::PoolState>(
                    state.value.decode().as_ref().unwrap(),
                )?;
                let tickarray_bitmap_extension =
                    deserialize_anchor_account::<raydium_amm_v3::states::TickArrayBitmapExtension>(
                        bitmap.value.decode().as_ref().unwrap(),
                    )?;
                let tick_arrays = load_cur_and_next_five_tick_array(
                    &rpc,
                    &raydium_program_id,
                    &tickarray_bitmap_extension.pool_id,
                    &pool_state,
                    &tickarray_bitmap_extension,
                    true,
                )
                .await?;

                tx.send(Pool {
                    id: tickarray_bitmap_extension.pool_id,
                    mint0: mint0.clone(),
                    mint1: mint1.clone(),
                    amm_config: amm_config.clone(),
                    state: pool_state,
                    tick_array: tick_arrays,
                    tick_array_bitmap_extension: tickarray_bitmap_extension,
                })?;
            }

            Ok::<_, anyhow::Error>(())
        });

        Ok(UnsubscribeHandle { handle, rx })
    }

    pub async fn quote(
        &self,
        rpc: Arc<RpcClient>,
        amount: u64,
        _sqrt_price_limit_x64: u128, // TODO: Would be nice to implement
        zero_for_one: bool,
        is_base_input: bool,
    ) -> anyhow::Result<u64> {
        let epoch = rpc.get_epoch_info().await?.epoch;
        let mut mint0 = self.mint0.clone();
        let mut mint1 = self.mint1.clone();
        let mint0_state = StateWithExtensionsMut::<Mint>::unpack(&mut mint0)?;
        let mint1_state = StateWithExtensionsMut::<Mint>::unpack(&mut mint1)?;
        let transfer_fee = if is_base_input {
            if zero_for_one {
                get_transfer_fee(&mint0_state, epoch, amount)
            } else {
                get_transfer_fee(&mint1_state, epoch, amount)
            }
        } else {
            0
        };
        let amount_specified = amount.checked_sub(transfer_fee).unwrap();

        let sqrt_price_limit_x64 = None;

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

        Ok(other_amount_threshold)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn swap(
        &self,
        rpc: Arc<RpcClient>,
        raydium_program_id: &Pubkey,
        payer: Keypair,
        input_token: Pubkey,
        output_token: Pubkey,
        amount: u64,
        slippage: f64,
        is_base_input: bool,
        simulate: bool,
    ) -> anyhow::Result<TransactionResult> {
        let load_accounts = vec![input_token, output_token];
        let rsps = rpc.get_multiple_accounts(&load_accounts).await?;
        let epoch = rpc.get_epoch_info().await?.epoch;
        let [user_input_account, user_output_account] = array_ref![rsps, 0, 2];

        let mut user_input_token_data = user_input_account.clone().unwrap().data;
        let user_input_state =
            StateWithExtensionsMut::<TokenAccount>::unpack(&mut user_input_token_data)?;
        let mut user_output_token_data = user_output_account.clone().unwrap().data;
        let user_output_state =
            StateWithExtensionsMut::<TokenAccount>::unpack(&mut user_output_token_data)?;

        let mut mint0 = self.mint0.clone();
        let mut mint1 = self.mint1.clone();
        let mint0_state = StateWithExtensionsMut::<Mint>::unpack(&mut mint0)?;
        let mint1_state = StateWithExtensionsMut::<Mint>::unpack(&mut mint1)?;

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
        let amount_specified = amount.checked_sub(transfer_fee).unwrap();
        // load tick_arrays
        let mut tick_arrays = load_cur_and_next_five_tick_array(
            &rpc,
            raydium_program_id,
            &self.id,
            &self.state,
            &self.tick_array_bitmap_extension,
            zero_for_one,
        )
        .await?;

        let mut sqrt_price_limit_x64 = None;

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

        let mut remaining_accounts = Vec::new();
        remaining_accounts.push(AccountMeta::new_readonly(
            Pubkey::find_program_address(
                &[
                    POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                    self.id.to_bytes().as_ref(),
                ],
                raydium_program_id,
            )
            .0,
            false,
        ));
        let mut accounts = tick_array_indexs
            .into_iter()
            .map(|index| {
                AccountMeta::new(
                    Pubkey::find_program_address(
                        &[
                            raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
                            self.id.to_bytes().as_ref(),
                            &index.to_be_bytes(),
                        ],
                        raydium_program_id,
                    )
                    .0,
                    false,
                )
            })
            .collect();
        remaining_accounts.append(&mut accounts);
        let mut instructions = Vec::new();
        let request_inits_instr = ComputeBudgetInstruction::set_compute_unit_limit(1_400_000_u32);
        instructions.push(request_inits_instr);
        let cluster = Cluster::Custom("".to_string(), "".to_string());
        let client = Client::new(cluster, &payer);
        let clmm = client.program(*raydium_program_id)?;

        let swap_instr = swap_v2_instr(
            clmm,
            self.state.amm_config,
            self.id,
            if zero_for_one {
                self.state.token_vault_0
            } else {
                self.state.token_vault_1
            },
            if zero_for_one {
                self.state.token_vault_1
            } else {
                self.state.token_vault_0
            },
            self.state.observation_key,
            input_token,
            output_token,
            if zero_for_one {
                self.state.token_mint_0
            } else {
                self.state.token_mint_1
            },
            if zero_for_one {
                self.state.token_mint_1
            } else {
                self.state.token_mint_0
            },
            remaining_accounts,
            amount,
            other_amount_threshold,
            sqrt_price_limit_x64,
            is_base_input,
        )
        .unwrap();
        instructions.extend(swap_instr);
        // send
        let signers = vec![&payer];
        let recent_hash = rpc.get_latest_blockhash().await?;
        let txn = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &signers,
            recent_hash,
        );
        if simulate {
            let ret = utils::simulate_transaction(&rpc, &txn, true, CommitmentConfig::confirmed())
                .await?;
            return Ok(TransactionResult::Simulate(ret.value));
        }

        let signature = utils::send_txn(&rpc, &txn, true).await?;
        Ok(TransactionResult::Send(signature.to_string()))
    }
}

pub fn deserialize_anchor_account<T: AccountDeserialize>(account: &Account) -> anyhow::Result<T> {
    let mut data: &[u8] = &account.data;
    T::try_deserialize(&mut data).map_err(Into::into)
}

async fn load_cur_and_next_five_tick_array(
    rpc_client: &RpcClient,
    raydium_program_id: &Pubkey,
    pool_id: &Pubkey,
    pool_state: &PoolState,
    tickarray_bitmap_extension: &TickArrayBitmapExtension,
    zero_for_one: bool,
) -> anyhow::Result<VecDeque<TickArrayState>> {
    let (_, mut current_vaild_tick_array_start_index) = pool_state
        .get_first_initialized_tick_array(&Some(*tickarray_bitmap_extension), zero_for_one)
        .unwrap();
    let mut tick_array_keys = Vec::new();
    tick_array_keys.push(
        Pubkey::find_program_address(
            &[
                raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
                pool_id.to_bytes().as_ref(),
                &current_vaild_tick_array_start_index.to_be_bytes(),
            ],
            raydium_program_id,
        )
        .0,
    );
    let mut max_array_size = 5;
    while max_array_size != 0 {
        let next_tick_array_index = pool_state
            .next_initialized_tick_array_start_index(
                &Some(*tickarray_bitmap_extension),
                current_vaild_tick_array_start_index,
                zero_for_one,
            )
            .unwrap();
        if next_tick_array_index.is_none() {
            break;
        }
        current_vaild_tick_array_start_index = next_tick_array_index.unwrap();
        tick_array_keys.push(
            Pubkey::find_program_address(
                &[
                    raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
                    pool_id.to_bytes().as_ref(),
                    &current_vaild_tick_array_start_index.to_be_bytes(),
                ],
                raydium_program_id,
            )
            .0,
        );
        max_array_size -= 1;
    }
    let tick_array_rsps = rpc_client.get_multiple_accounts(&tick_array_keys).await?;
    let mut tick_arrays = VecDeque::new();
    for tick_array in tick_array_rsps {
        let tick_array_state =
            deserialize_anchor_account::<raydium_amm_v3::states::TickArrayState>(
                &tick_array.unwrap(),
            )
            .unwrap();
        tick_arrays.push_back(tick_array_state);
    }
    Ok(tick_arrays)
}

pub fn get_transfer_fee<S: BaseState>(
    account_state: &StateWithExtensionsMut<'_, S>,
    epoch: u64,
    pre_fee_amount: u64,
) -> u64 {
    let fee = if let Ok(transfer_fee_config) = account_state.get_extension::<TransferFeeConfig>() {
        transfer_fee_config
            .calculate_epoch_fee(epoch, pre_fee_amount)
            .unwrap()
    } else {
        0
    };
    fee
}

// {
//     "jsonrpc": "2.0",
//     "id": 1,
//     "method": "programSubscribe",
//     "params": [
//       "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK",
//       {
//         "encoding": "base64",
//         "commitment": "finalized"
//       }
//     ]
//   }
//   {"jsonrpc":"2.0","id":1,"method":"programSubscribe","params":["CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK",{"encoding":"jsonParsed","commitment":"finalized"}]}

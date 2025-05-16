use std::str::FromStr;

use actix_web::{middleware::Logger, web, App, HttpServer};
use aes_gcm::{aead::OsRng, Aes256Gcm, KeyInit};
use clap::{Parser, Subcommand};
use solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;

use crate::{models::account::Account, routes::routes, state::AppState};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cmd {
    /// PSQL database URL
    #[arg(
        long,
        default_value = "postgresql://postgres:postgres@localhost:5432/raydium"
    )]
    database_url: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    New(NewCommands),
    Server {
        /// Solana RPC URL, can be anythin as long as it's a valid RPC URL and https://
        #[arg(short, long)]
        rpc: String,
        /// Raydium CLMM program ID, useful for testing on devnet
        #[arg(long, default_value = crate::constants::RAYDIUM_CLMM_PUBKEY)]
        raydium_clmm: String,
        // #[arg(long, value_parser = parse_masterkey)]
        // masterkey: std::vec::Vec<u8>,
        // #[arg(long)]
        // jwt_secret: String,
    },
}

#[derive(Subcommand)]
enum NewCommands {
    /// Can create a new masterkey for you
    Masterkey,
    /// Can create a new wallet in datbase and encrypt it
    Wallet {
        /// How long this account will be valid for
        #[arg(long, value_parser = parse_duration)]
        duration: chrono::Duration,
        // #[arg(long, value_parser = parse_masterkey)]
        // masterkey: std::vec::Vec<u8>,
        // #[arg(long)]
        // jwt_secret: String,
    },
}

fn parse_duration(s: &str) -> anyhow::Result<chrono::Duration> {
    Ok(chrono::Duration::from_std(humantime::parse_duration(s)?)?)
}
fn parse_masterkey(s: &str) -> anyhow::Result<Vec<u8>> {
    Ok(bs58::decode(s).into_vec()?)
}

impl Cmd {
    pub async fn process(&self) -> anyhow::Result<()> {
        match &self.command {
            Some(Commands::Server {
                rpc,
                raydium_clmm,
            }) => {
                let pool = create_pool(&self.database_url).await?;
                let state = AppState::new(
                    rpc.to_string(),
                    pool,
                    Pubkey::from_str(raydium_clmm).expect("RAYDIUM_CLMM_PUBKEY invalid"),
                )
                .await
                .expect("Unable to create state");
                HttpServer::new(move || {
                    App::new()
                        .app_data(web::Data::new(state.clone()))
                        .wrap(Logger::default())
                        .service(web::scope("/api").configure(routes))
                })
                .bind(("127.0.0.1", 8080))?
                .run()
                .await?;
            }
            Some(Commands::New(cmd)) => match cmd {
                NewCommands::Masterkey => {
                    let masterkey = Aes256Gcm::generate_key(OsRng);
                    println!("{}", bs58::encode(masterkey.as_slice()).into_string());
                }
                NewCommands::Wallet {
                    duration,
                } => {
                    let pool = create_pool(&self.database_url).await?;
                    let mut account = Account::new_unique(&[], *duration).unwrap();
                    let id = account.save(&pool).await?;
                    println!("Account id: {}", id);
                    println!("Public key: {}", account.pubkey());
                }
            },
            None => todo!(),
        }
        Ok(())
    }
}

async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    sqlx::query!(
        "create table if not exists accounts (
                id serial primary key,
                ciphertext bytea not null,
                nonce bytea not null,
                die_at bigint not null
            );"
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

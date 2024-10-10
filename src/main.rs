use clap::Parser;
use pricefeeder::cmd::Cmd;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    Cmd::parse().process().await?;
    Ok(())
}

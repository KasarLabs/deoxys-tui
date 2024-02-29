mod deoxys_ui;
mod ui;
mod app;

use anyhow::Result;
use deoxys_ui as tui;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_endpoint = "https://starknet-testnet.public.blastapi.io/rpc/v0_6//";//"http://localhost:9944";
    tui::run(&rpc_endpoint).await?;
    Ok(())
}
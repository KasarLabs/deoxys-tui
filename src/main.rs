mod app;
mod deoxys_ui;
mod ui;

use anyhow::Result;
use deoxys_ui as tui;

#[tokio::main]
async fn main() -> Result<()> {
    // TEMPORAIRE
    tui::run("http://querypointer.com:9545", "usecpu", "/home/noa/Desktop").await?;
    Ok(())
}

mod app;
mod deoxys_ui;
mod ui;

use anyhow::Result;
use deoxys_ui as tui;

#[tokio::main]
async fn main() -> Result<()> {
    // TEMPORAIRE
    tui::run("http://localhost:9944").await?;
    Ok(())
}

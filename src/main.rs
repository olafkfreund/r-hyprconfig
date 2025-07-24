use anyhow::Result;
use clap::Parser;

mod app;
mod hyprctl;
mod ui;
mod config;

use app::App;

#[derive(Parser)]
#[command(name = "r-hyprconfig")]
#[command(about = "A modern TUI for managing Hyprland configuration")]
struct Cli {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let mut app = App::new(cli.debug).await?;
    app.run().await?;
    
    Ok(())
}
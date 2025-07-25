use anyhow::Result;
use clap::Parser;

mod app;
mod config;
mod hyprctl;
mod theme;
mod ui;
mod nixos;

use app::App;

#[derive(Parser)]
#[command(name = "r-hyprconfig")]
#[command(about = "A modern TUI for managing Hyprland configuration")]
struct Cli {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// Test save functionality without running TUI
    #[arg(long)]
    test_save: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut app = App::new(cli.debug).await?;

    if cli.test_save {
        app.test_save_functionality().await?;
    } else {
        app.run().await?;
    }

    Ok(())
}

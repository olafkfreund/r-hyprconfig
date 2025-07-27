#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::single_char_add_str)]
#![allow(clippy::double_ended_iterator_last)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::derivable_impls)]

use anyhow::Result;
use clap::Parser;

mod app;
mod batch;
mod config;
mod hyprctl;
mod import_export;
mod nixos;
mod platform;
mod theme;
mod ui;

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

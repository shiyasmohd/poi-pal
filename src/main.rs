use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use poi_agent::commands::{CheckDivergenceCommand, PoiCommand};

#[derive(Debug, Parser)]
#[command(
    name = "poi-agent",
    author = "POI Agent",
    version = "0.1.0",
    about = "A CLI tool for managing Proof of Indexing (POI) operations on The Graph",
    long_about = "A command-line interface for fetching and comparing Proof of Indexing data \
                  from The Graph indexers. Supports POI fetching and divergence checking."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(
        name = "poi",
        about = "Fetch POIs for a specific deployment and block",
        long_about = "Fetches Proof of Indexing data from all active indexers for a given \
                      deployment ID and block number. Displays the POI hash for each indexer."
    )]
    Poi(PoiCommand),

    #[command(
        name = "check-divergence",
        about = "Find diverged blocks using binary search",
        long_about = "Performs a binary search between a start and end block to find the first \
                      block where POIs diverge from a trusted indexer. This helps identify \
                      where indexing discrepancies begin."
    )]
    CheckDivergence(CheckDivergenceCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Poi(cmd) => {
            if let Err(e) = cmd.execute().await {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::CheckDivergence(cmd) => {
            if let Err(e) = cmd.execute().await {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

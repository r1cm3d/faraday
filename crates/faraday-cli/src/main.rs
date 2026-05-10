/*!
Faraday CLI - OBD-II diagnostics for Ford Fusion 2017 SEL.

A command-line tool for automotive diagnostics and configuration
through FORScan-compatible OBD-II adapters.
*/

mod cli;
mod commands;
mod error;
mod output;

use anyhow::Result;
use clap::Parser;
use cli::{Args, Commands};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup_logging(&args)?;

    info!("Faraday CLI v{} starting", env!("CARGO_PKG_VERSION"));

    if let Err(err) = run_command(args).await {
        error!("Command failed: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_command(args: Args) -> Result<()> {
    match args.command {
        Commands::ReadDtc { module, stored, pending, permanent } => {
            commands::read_dtc::execute(args.adapter, module, stored, pending, permanent).await
        }
        Commands::ClearDtc { module } => {
            commands::clear_dtc::execute(args.adapter, module).await
        }
        Commands::Live { pids, module, interval } => {
            commands::live::execute(args.adapter, module, pids, interval).await
        }
        Commands::Vin { method } => {
            commands::vin::execute(args.adapter, method).await
        }
        Commands::ReadDid { module, did } => {
            commands::read_did::execute(args.adapter, module, did).await
        }
        Commands::Session { module, session } => {
            commands::session::execute(args.adapter, module, session).await
        }
    }
}

fn setup_logging(args: &Args) -> Result<()> {
    let level = match args.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    if args.verbose > 0 {
        info!("Logging level set to {:?}", level);
    }

    Ok(())
}
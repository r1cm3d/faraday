/*!
Faraday CLI - OBD-II diagnostics for Ford Fusion 2017 SEL.

A command-line tool for automotive diagnostics and configuration
through FORScan-compatible OBD-II adapters.
*/

mod audit;
mod cli;
mod commands;
mod error;
mod output;
mod profile;
mod session_log;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use cli::{Args, AsBuiltAction, Commands, ProfileAction};
use std::time::Instant;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup_logging(&args)?;

    info!("Faraday CLI v{} starting", env!("CARGO_PKG_VERSION"));

    let adapter = args.adapter.clone();
    let command_label = command_label(&args.command);
    let start = Instant::now();

    let result = run_command(args).await;

    let duration_ms = start.elapsed().as_millis() as u64;
    let result_str = match &result {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    log_session(adapter, command_label, duration_ms, result_str);

    if let Err(err) = result {
        error!("Command failed: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_command(args: Args) -> Result<()> {
    match args.command {
        Commands::ReadDtc {
            module,
            stored,
            pending,
            permanent,
        } => commands::read_dtc::execute(args.adapter, module, stored, pending, permanent).await,
        Commands::ClearDtc { module } => commands::clear_dtc::execute(args.adapter, module).await,
        Commands::Live {
            pids,
            module,
            interval,
        } => commands::live::execute(args.adapter, module, pids, interval).await,
        Commands::Vin { method } => commands::vin::execute(args.adapter, method).await,
        Commands::ReadDid { module, did } => {
            commands::read_did::execute(args.adapter, module, did).await
        }
        Commands::Session { module, session } => {
            commands::session::execute(args.adapter, module, session).await
        }
        Commands::Asbuilt { action } => commands::asbuilt::execute(args.adapter, action).await,
        Commands::Profile { action } => commands::profile::execute(args.adapter, action).await,
    }
}

fn command_label(cmd: &Commands) -> String {
    match cmd {
        Commands::ReadDtc { .. } => "read-dtc".to_string(),
        Commands::ClearDtc { .. } => "clear-dtc".to_string(),
        Commands::Live { .. } => "live".to_string(),
        Commands::Vin { .. } => "vin".to_string(),
        Commands::ReadDid { did, .. } => format!("read-did {}", did),
        Commands::Session { .. } => "session".to_string(),
        Commands::Asbuilt { action } => match action {
            AsBuiltAction::Dump { .. } => "asbuilt dump".to_string(),
            AsBuiltAction::Show { feature, .. } => format!("asbuilt show {}", feature),
            AsBuiltAction::Write { feature, .. } => format!("asbuilt write {}", feature),
            AsBuiltAction::Snapshot { .. } => "asbuilt snapshot".to_string(),
            AsBuiltAction::Restore { .. } => "asbuilt restore".to_string(),
        },
        Commands::Profile { action } => match action {
            ProfileAction::Apply { file, .. } => {
                format!("profile apply {}", file.display())
            }
            ProfileAction::Validate { file } => {
                format!("profile validate {}", file.display())
            }
        },
    }
}

fn log_session(adapter: String, command: String, duration_ms: u64, result: String) {
    let entry = session_log::SessionEntry {
        timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        command,
        adapter,
        duration_ms,
        result,
    };
    if let Ok(logger) = session_log::SessionLogger::new() {
        if let Err(e) = logger.log(&entry) {
            tracing::warn!("session log failed: {}", e);
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

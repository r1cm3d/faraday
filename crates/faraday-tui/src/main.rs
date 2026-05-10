/*!
Faraday TUI - Real-time OBD-II data visualization.

A terminal user interface for monitoring live vehicle data with
gauges, graphs, and real-time updates.
*/

mod app;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};
use tracing::{error, info};

#[derive(Parser)]
#[command(
    name = "faraday-tui",
    version,
    about = "Terminal UI for live OBD-II data visualization"
)]
struct Args {
    #[arg(
        long,
        env = "FARADAY_ADAPTER",
        default_value = "/dev/ttyUSB0",
        help = "OBD-II adapter device path or port"
    )]
    adapter: String,

    #[arg(
        long,
        short,
        default_value = "500",
        help = "Update interval in milliseconds"
    )]
    interval: u64,

    #[arg(
        short,
        long,
        help = "Increase verbosity (-v, -vv, -vvv)",
        action = clap::ArgAction::Count
    )]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup_logging(args.verbose)?;

    info!("Faraday TUI v{} starting", env!("CARGO_PKG_VERSION"));

    run_tui(args).await
}

async fn run_tui(args: Args) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = app::App::new(args.adapter, Duration::from_millis(args.interval)).await?;
    let res = run_app(&mut terminal, app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        error!("TUI error: {}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: app::App) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => app.reset_data(),
                    KeyCode::Char('p') => app.toggle_pause(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await?;
            last_tick = Instant::now();
        }
    }
}

fn setup_logging(verbose: u8) -> Result<()> {
    let level = match verbose {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        _ => tracing::Level::DEBUG,
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

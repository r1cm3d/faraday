mod app;
mod panels;
mod ui;
mod widgets;

use anyhow::Result;
use app::ActiveTab;
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
    about = "Comprehensive Ford diagnostic terminal UI"
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

    let app = app::App::new(args.adapter).await?;
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
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => app.reset_data(),
                    KeyCode::Char('p') => app.toggle_pause(),
                    KeyCode::Left => app.prev_tab(),
                    KeyCode::Right => app.next_tab(),
                    KeyCode::Char('1') => app.goto_tab(ActiveTab::Engine),
                    KeyCode::Char('2') => app.goto_tab(ActiveTab::Transmission),
                    KeyCode::Char('3') => app.goto_tab(ActiveTab::Body),
                    KeyCode::Char('4') => app.goto_tab(ActiveTab::Safety),
                    KeyCode::Char('5') => app.goto_tab(ActiveTab::Adas),
                    KeyCode::Char('6') => app.goto_tab(ActiveTab::Climate),
                    KeyCode::Char('7') => app.goto_tab(ActiveTab::Infotainment),
                    KeyCode::Char('8') => app.goto_tab(ActiveTab::Analytics),
                    KeyCode::Char('9') => app.goto_tab(ActiveTab::Health),
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

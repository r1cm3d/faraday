//! Top-level frame layout: tab bar, status bar, panel area, and help bar.
use crate::app::{ActiveTab, App, ConnectionStatus};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    draw_tab_bar(f, chunks[0], app);
    draw_status_bar(f, chunks[1], app);
    draw_panel(f, chunks[2], app);
    draw_help_bar(f, chunks[3], app);
}

fn draw_tab_bar<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let titles: Vec<Spans> = (0..crate::app::N_TABS)
        .map(|i| Spans::from(ActiveTab::from_index(i).title()))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Faraday Diagnostics"),
        )
        .select(app.active_tab.index())
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, area);
}

fn draw_status_bar<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let (conn_label, conn_color) = match app.connection_status {
        ConnectionStatus::Connected => ("Connected", Color::Green),
        ConnectionStatus::Connecting => ("Connecting…", Color::Yellow),
        ConnectionStatus::Error => ("Error", Color::Red),
        ConnectionStatus::Disconnected => ("Disconnected", Color::Gray),
    };

    let pause_str = if app.paused { " [PAUSED]" } else { "" };

    let battery_str = app
        .battery_voltage()
        .map(|v| format!(" | Batt: {:.2}V", v))
        .unwrap_or_default();

    let mut spans = vec![
        Span::styled(conn_label, Style::default().fg(conn_color)),
        Span::raw(format!("{}{}", pause_str, battery_str)),
    ];

    if let Some(err) = &app.error_message {
        spans.push(Span::styled(
            format!(" | {}", err),
            Style::default().fg(Color::Red),
        ));
    }

    let status_para = Paragraph::new(Spans::from(spans))
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .alignment(Alignment::Left);
    f.render_widget(status_para, area);
}

fn draw_panel<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    match app.active_tab {
        ActiveTab::Engine => app.engine.render(f, area),
        ActiveTab::Transmission => app.transmission.render(f, area),
        ActiveTab::Body => app.body.render(f, area),
        ActiveTab::Safety => app.safety.render(f, area),
        ActiveTab::Adas => app.adas.render(f, area),
        ActiveTab::Climate => app.climate.render(f, area),
        ActiveTab::Infotainment => app.infotainment.render(f, area),
        ActiveTab::Analytics => app.analytics.render(f, area),
        ActiveTab::Health => app.health.render(f, area),
        ActiveTab::Glossary => app.glossary.render(f, area),
    }
}

fn draw_help_bar<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let nav = "←/→: tabs | 1-9/0: jump | p: pause | r: reset | q: quit";
    let panel_help = app.active_help_text();
    let text = format!("{}  |  {}", nav, panel_help);
    let help_para = Paragraph::new(text.as_str())
        .block(Block::default().borders(Borders::ALL).title("Keys"))
        .alignment(Alignment::Left);
    f.render_widget(help_para, area);
}

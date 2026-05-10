use crate::app::{App, ConnectionStatus};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, Paragraph},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Status bar
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Help text
        ])
        .split(f.size());

    draw_status_bar(f, chunks[0], app);
    draw_main_content(f, chunks[1], app);
    draw_help_text(f, chunks[2]);
}

fn draw_status_bar<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let status_text = match app.connection_status() {
        ConnectionStatus::Disconnected => "Disconnected",
        ConnectionStatus::Connecting => "Connecting...",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Error => "Error",
    };

    let _status_color = match app.connection_status() {
        ConnectionStatus::Connected => Color::Green,
        ConnectionStatus::Connecting => Color::Yellow,
        ConnectionStatus::Error => Color::Red,
        ConnectionStatus::Disconnected => Color::Gray,
    };

    let pause_text = if app.is_paused() { " [PAUSED]" } else { "" };

    let status_text_full = format!(
        "Status: {} {} | Data Points: {}",
        status_text,
        pause_text,
        app.data_points_count()
    );

    let mut status_content = status_text_full;
    if let Some(error) = app.error_message() {
        status_content = format!("{}\nError: {}", status_content, error);
    }

    let status_paragraph = Paragraph::new(status_content)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .alignment(Alignment::Left);

    f.render_widget(status_paragraph, area);
}

fn draw_main_content<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_gauges(f, main_chunks[0], app);
    draw_charts(f, main_chunks[1], app);
}

fn draw_gauges<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25), // RPM
            Constraint::Percentage(25), // Speed
            Constraint::Percentage(25), // Coolant Temp
            Constraint::Percentage(25), // Engine Load
        ])
        .split(area);

    if let Some(snapshot) = app.latest_snapshot() {
        // RPM Gauge
        let rpm = snapshot.rpm.unwrap_or(0.0) as u16;
        let rpm_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("RPM"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(((rpm as f64 / 7000.0) * 100.0) as u16)
            .label(format!("{} rpm", rpm));
        f.render_widget(rpm_gauge, gauge_chunks[0]);

        // Speed Gauge
        let speed = snapshot.speed.unwrap_or(0.0) as u16;
        let speed_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Speed"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(((speed as f64 / 200.0) * 100.0) as u16)
            .label(format!("{} km/h", speed));
        f.render_widget(speed_gauge, gauge_chunks[1]);

        // Coolant Temperature
        let temp = snapshot.coolant_temp.unwrap_or(-40.0);
        let temp_percent = ((temp + 40.0) / 150.0 * 100.0) as u16;
        let temp_color = if temp > 100.0 {
            Color::Red
        } else {
            Color::Blue
        };
        let temp_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Coolant Temp"))
            .gauge_style(Style::default().fg(temp_color))
            .percent(temp_percent)
            .label(format!("{:.1}°C", temp));
        f.render_widget(temp_gauge, gauge_chunks[2]);

        // Engine Load
        let load = snapshot.engine_load.unwrap_or(0.0) as u16;
        let load_color = if load > 80 { Color::Red } else { Color::Yellow };
        let load_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Engine Load"))
            .gauge_style(Style::default().fg(load_color))
            .percent(load)
            .label(format!("{}%", load));
        f.render_widget(load_gauge, gauge_chunks[3]);
    } else {
        // No data available
        for chunk in gauge_chunks.iter() {
            let no_data = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("No Data"))
                .gauge_style(Style::default().fg(Color::Gray))
                .percent(0)
                .label("--");
            f.render_widget(no_data, *chunk);
        }
    }
}

fn draw_charts<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let chart_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_rpm_chart(f, chart_chunks[0], app);
    draw_speed_chart(f, chart_chunks[1], app);
}

fn draw_rpm_chart<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let data: Vec<(f64, f64)> = app
        .engine_data()
        .iter()
        .enumerate()
        .filter_map(|(i, snapshot)| snapshot.rpm.map(|rpm| (i as f64, rpm)))
        .collect();

    if !data.is_empty() {
        let dataset = Dataset::default()
            .name("RPM")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(&data);

        let chart = Chart::new(vec![dataset])
            .block(Block::default().borders(Borders::ALL).title("RPM History"))
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, data.len() as f64]),
            )
            .y_axis(
                Axis::default()
                    .title("RPM")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 7000.0]),
            );

        f.render_widget(chart, area);
    } else {
        let no_data = Paragraph::new("No RPM data available")
            .block(Block::default().borders(Borders::ALL).title("RPM History"))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
    }
}

fn draw_speed_chart<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let data: Vec<(f64, f64)> = app
        .engine_data()
        .iter()
        .enumerate()
        .filter_map(|(i, snapshot)| snapshot.speed.map(|speed| (i as f64, speed)))
        .collect();

    if !data.is_empty() {
        let dataset = Dataset::default()
            .name("Speed")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Green))
            .data(&data);

        let chart = Chart::new(vec![dataset])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Speed History"),
            )
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, data.len() as f64]),
            )
            .y_axis(
                Axis::default()
                    .title("km/h")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 200.0]),
            );

        f.render_widget(chart, area);
    } else {
        let no_data = Paragraph::new("No speed data available")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Speed History"),
            )
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
    }
}

fn draw_help_text<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = "q/ESC: Quit  r: Reset Data  p: Pause/Resume";

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Center);

    f.render_widget(help_paragraph, area);
}

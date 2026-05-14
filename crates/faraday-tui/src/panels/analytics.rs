use super::{engine::EngineSnapshot, HISTORY_LEN};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};
use std::time::Instant;

pub struct AnalyticsPanel {
    pub rpm_histogram: [u64; 4],
    pub speed_distribution: [u64; 4],
    pub total_fuel_liters: f64,
    pub brake_events: u64,
    pub accel_events: u64,
    pub data_points: u64,
    pub fuel_rate_history: Vec<u64>,
    pub last_updated: Option<Instant>,
    prev_speed: Option<f64>,
}

impl AnalyticsPanel {
    pub fn new() -> Self {
        Self {
            rpm_histogram: [0; 4],
            speed_distribution: [0; 4],
            total_fuel_liters: 0.0,
            brake_events: 0,
            accel_events: 0,
            data_points: 0,
            fuel_rate_history: Vec::new(),
            last_updated: None,
            prev_speed: None,
        }
    }

    pub fn ingest(&mut self, snap: &EngineSnapshot, interval_secs: f64) {
        self.data_points += 1;

        if let Some(rpm) = snap.rpm {
            let bucket = match rpm as u32 {
                0..=999 => 0,
                1000..=2499 => 1,
                2500..=4499 => 2,
                _ => 3,
            };
            self.rpm_histogram[bucket] += 1;
        }

        if let Some(speed) = snap.speed {
            let bucket = match speed as u32 {
                0..=29 => 0,
                30..=79 => 1,
                80..=119 => 2,
                _ => 3,
            };
            self.speed_distribution[bucket] += 1;

            if let Some(prev) = self.prev_speed {
                let delta = speed - prev;
                let rate = delta / interval_secs;
                if rate < -8.0 {
                    self.brake_events += 1;
                }
                if rate > 2.7 {
                    self.accel_events += 1;
                }
            }
            self.prev_speed = Some(speed);
        }

        if let Some(fuel_rate) = snap.fuel_rate {
            self.total_fuel_liters += fuel_rate * interval_secs / 3600.0;
            let scaled = (fuel_rate * 10.0).clamp(0.0, u64::MAX as f64) as u64;
            self.fuel_rate_history.push(scaled);
            if self.fuel_rate_history.len() > HISTORY_LEN {
                self.fuel_rate_history.remove(0);
            }
        }

        self.last_updated = Some(Instant::now());
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let hist_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(rows[0]);

        let rpm_labels = [
            "Idle\n<1k",
            "Low\n1-2.5k",
            "Cruise\n2.5-4.5k",
            "High\n>4.5k",
        ];
        let total_rpm: u64 = self.rpm_histogram.iter().sum::<u64>().max(1);
        for (i, (label, &col)) in rpm_labels.iter().zip(hist_cols.iter()).enumerate() {
            let pct = (self.rpm_histogram[i] * 100 / total_rpm) as u16;
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(*label))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(pct)
                .label(format!("{}%", pct));
            f.render_widget(gauge, col);
        }

        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        let fuel_data = if self.fuel_rate_history.is_empty() {
            vec![0u64]
        } else {
            self.fuel_rate_history.clone()
        };
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Fuel Rate History (×0.1 L/h)"),
            )
            .style(Style::default().fg(Color::Yellow))
            .data(&fuel_data);
        f.render_widget(sparkline, bottom_cols[0]);

        let _total_rpm_pts: u64 = self.rpm_histogram.iter().sum();
        let total_spd_pts: u64 = self.speed_distribution.iter().sum::<u64>().max(1);
        let stats_lines = vec![
            Spans::from(format!("Total data points: {}", self.data_points)),
            Spans::from(format!(
                "Total fuel consumed: {:.2} L / {:.2} gal",
                self.total_fuel_liters,
                self.total_fuel_liters * 0.264172,
            )),
            Spans::from(format!("Brake events (est.): {}", self.brake_events)),
            Spans::from(format!("Hard accel events:   {}", self.accel_events)),
            Spans::from(""),
            Spans::from("Speed dist: City/Highway/Fast/Very fast"),
            Spans::from(format!(
                "  {}% / {}% / {}% / {}%",
                self.speed_distribution[0] * 100 / total_spd_pts,
                self.speed_distribution[1] * 100 / total_spd_pts,
                self.speed_distribution[2] * 100 / total_spd_pts,
                self.speed_distribution[3] * 100 / total_spd_pts,
            )),
        ];
        let stats_para = Paragraph::new(stats_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Session Statistics"),
        );
        f.render_widget(stats_para, bottom_cols[1]);
    }

    pub fn help_text(&self) -> &str {
        "Analytics: RPM histogram · Speed distribution · Fuel consumed · Brake/Accel events"
    }
}

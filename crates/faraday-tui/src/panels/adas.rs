//! Parking aid module ultrasonic sensor panel.
use super::{fmt_dist_cm, sensor_opt, u16_be};
use faraday_core::{commands::CommandExecutor, transport::IsoTpTransport, Module};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};
use std::time::Instant;

const MAX_SENSOR_CM: u16 = 250;

#[derive(Default)]
pub struct AdasData {
    pub sensors_front: [Option<u16>; 4],
    pub sensors_rear: [Option<u16>; 4],
}

pub struct AdasPanel {
    pub data: AdasData,
    pub last_updated: Option<Instant>,
    pub error: Option<String>,
}

impl AdasPanel {
    pub fn new() -> Self {
        Self {
            data: AdasData::default(),
            last_updated: None,
            error: None,
        }
    }

    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        self.error = None;
        let d = &mut self.data;

        if let Ok(raw) = executor.read_asbuilt_block(Module::Pam, 0xE001).await {
            if raw.len() >= 16 {
                for i in 0..4 {
                    d.sensors_front[i] = sensor_opt(u16_be(raw[i * 2], raw[i * 2 + 1]));
                    d.sensors_rear[i] = sensor_opt(u16_be(raw[8 + i * 2], raw[8 + i * 2 + 1]));
                }
            } else if raw.len() >= 8 {
                for i in 0..4 {
                    d.sensors_front[i] = sensor_opt(u16_be(raw[i * 2], raw[i * 2 + 1]));
                }
            }
        } else {
            d.sensors_front = [None; 4];
            d.sensors_rear = [None; 4];
        }

        self.last_updated = Some(Instant::now());
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let d = &self.data;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let front_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(rows[0]);

        let labels = ["FL", "FC-L", "FC-R", "FR"];
        for (i, (label, &col)) in labels.iter().zip(front_cols.iter()).enumerate() {
            let sensor = d.sensors_front[i];
            let percent = sensor
                .map(|v| 100u16.saturating_sub(v * 100 / MAX_SENSOR_CM))
                .unwrap_or(0);
            let color = match sensor {
                Some(v) if v < 50 => Color::Red,
                Some(v) if v < 100 => Color::Yellow,
                Some(_) => Color::Green,
                None => Color::DarkGray,
            };
            let label_str = format!("{}: {}", label, fmt_dist_cm(sensor));
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(Style::default().fg(color))
                .percent(percent)
                .label(label_str);
            f.render_widget(gauge, col);
        }

        let rear_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(rows[1]);

        let rear_labels = ["RL", "RC-L", "RC-R", "RR"];
        for (i, (label, &col)) in rear_labels.iter().zip(rear_cols.iter()).enumerate() {
            let sensor = d.sensors_rear[i];
            let percent = sensor
                .map(|v| 100u16.saturating_sub(v * 100 / MAX_SENSOR_CM))
                .unwrap_or(0);
            let color = match sensor {
                Some(v) if v < 50 => Color::Red,
                Some(v) if v < 100 => Color::Yellow,
                Some(_) => Color::Green,
                None => Color::DarkGray,
            };
            let label_str = format!("{}: {}", label, fmt_dist_cm(sensor));
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(Style::default().fg(color))
                .percent(percent)
                .label(label_str);
            f.render_widget(gauge, col);
        }
    }

    pub fn help_text(&self) -> &str {
        "ADAS: 8 ultrasonic sensors (front/rear), backup camera status"
    }
}

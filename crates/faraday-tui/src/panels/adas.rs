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

/// Maximum measurable sensor distance in cm; used to scale the proximity gauges.
const MAX_SENSOR_CM: u16 = 250;

/// Latest PAM ultrasonic sensor readings.
#[derive(Default)]
pub struct AdasData {
    /// Front sensor distances in cm (FL, FC-L, FC-R, FR); `None` = absent/out-of-range.
    pub sensors_front: [Option<u16>; 4],
    /// Rear sensor distances in cm (RL, RC-L, RC-R, RR); `None` = absent/out-of-range.
    pub sensors_rear: [Option<u16>; 4],
}

/// Live parking aid module (PAM) ultrasonic sensor panel.
pub struct AdasPanel {
    /// Most recent PAM sensor readings.
    pub data: AdasData,
    /// Timestamp of the last successful update.
    pub last_updated: Option<Instant>,
    /// Error message from the most recent failed update, if any.
    pub error: Option<String>,
}

impl AdasPanel {
    /// Create a new `AdasPanel` with empty state.
    pub fn new() -> Self {
        Self {
            data: AdasData::default(),
            last_updated: None,
            error: None,
        }
    }

    /// Poll PAM DID 0xE001 and decode up to 8 ultrasonic sensor distances.
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

    /// Render the ADAS sensor panel into `area`.
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let d = &self.data;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.render_sensors(f, rows[0], &["FL", "FC-L", "FC-R", "FR"], &d.sensors_front);
        self.render_sensors(f, rows[1], &["RL", "RC-L", "RC-R", "RR"], &d.sensors_rear);
    }

    fn render_sensors<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        labels: &[&str],
        sensors: &[Option<u16>; 4],
    ) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        for (i, (label, &col)) in labels.iter().zip(cols.iter()).enumerate() {
            let sensor = sensors[i];
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

    /// Return the context-sensitive help string for this panel.
    pub fn help_text(&self) -> &str {
        "ADAS: 8 ultrasonic sensors (front/rear), backup camera status"
    }
}

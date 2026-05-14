//! Body control module live data panel.
use super::{fmt_opt_f, pwm_to_percent, u16_be};
use faraday_core::{commands::CommandExecutor, transport::IsoTpTransport, Module};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::time::Instant;

/// Latest BCM data values read from the CAN bus.
#[derive(Default)]
pub struct BodyData {
    /// Battery voltage under load in volts (DID 0x4001, scaled ÷1000).
    pub battery_voltage: Option<f64>,
    /// Alternator duty cycle as a percentage (DID 0x4002, PWM-scaled).
    pub alternator_duty: Option<f64>,
    /// Door-ajar bitmask: bit N = door N open (DID 0x4010).
    pub door_ajar: Option<u8>,
}

/// Live body control module diagnostic panel.
pub struct BodyPanel {
    /// Most recent BCM data values.
    pub data: BodyData,
    /// Timestamp of the last successful update.
    pub last_updated: Option<Instant>,
    /// Error message from the most recent failed update, if any.
    pub error: Option<String>,
}

impl BodyPanel {
    /// Create a new `BodyPanel` with empty state.
    pub fn new() -> Self {
        Self {
            data: BodyData::default(),
            last_updated: None,
            error: None,
        }
    }

    /// Poll BCM DIDs and update battery voltage, alternator duty, and door status.
    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        self.error = None;
        let d = &mut self.data;

        if let Ok(raw) = executor.read_asbuilt_block(Module::Bcm, 0x4001).await {
            if raw.len() >= 2 {
                d.battery_voltage = Some(u16_be(raw[0], raw[1]) as f64 / 1000.0);
            }
        } else {
            d.battery_voltage = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Bcm, 0x4002).await {
            if !raw.is_empty() {
                d.alternator_duty = Some(pwm_to_percent(raw[0]));
            }
        } else {
            d.alternator_duty = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Bcm, 0x4010).await {
            if !raw.is_empty() {
                d.door_ajar = Some(raw[0]);
            }
        } else {
            d.door_ajar = None;
        }

        self.last_updated = Some(Instant::now());
    }

    /// Render the body panel into `area`.
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let d = &self.data;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(4)])
            .split(area);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let batt = d.battery_voltage.unwrap_or(0.0);
        let batt_color = if batt < 12.0 {
            Color::Red
        } else if batt < 12.4 {
            Color::Yellow
        } else {
            Color::Green
        };
        let batt_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Battery"))
            .gauge_style(Style::default().fg(batt_color))
            .percent(((batt - 10.0) / 6.0 * 100.0).clamp(0.0, 100.0) as u16)
            .label(fmt_opt_f(d.battery_voltage, 2, " V"));
        f.render_widget(batt_gauge, cols[0]);

        let alt = d.alternator_duty.unwrap_or(0.0);
        let alt_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Alternator"))
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(alt as u16)
            .label(fmt_opt_f(d.alternator_duty, 1, "%"));
        f.render_widget(alt_gauge, cols[1]);

        let door_bits = d.door_ajar.unwrap_or(0);
        let door_names = ["Driver", "Pass.", "RL", "RR", "Trunk", "Hood"];
        let door_lines: Vec<Spans> = door_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if d.door_ajar.is_none() {
                    Spans::from(format!("{}: --", name))
                } else {
                    let open = (door_bits >> i) & 1 == 1;
                    let (status, color) = if open {
                        ("OPEN  ", Color::Yellow)
                    } else {
                        ("closed", Color::Green)
                    };
                    Spans::from(vec![
                        Span::raw(format!("{}: ", name)),
                        Span::styled(status, Style::default().fg(color)),
                    ])
                }
            })
            .collect();

        let door_para = Paragraph::new(door_lines)
            .block(Block::default().borders(Borders::ALL).title("Door Status"));
        f.render_widget(door_para, rows[1]);
    }

    /// Return the context-sensitive help string for this panel.
    pub fn help_text(&self) -> &str {
        "Body: Battery Voltage · Alternator · Door Ajar · Window Motors · Lighting"
    }
}

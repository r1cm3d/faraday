use super::{celsius_from_raw, fmt_opt_f, fmt_pressure_kpa, fmt_temp, u16_be, HISTORY_LEN};
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

#[derive(Default)]
pub struct TransmissionData {
    pub fluid_temp: Option<f64>,
    pub current_gear: Option<u8>,
    pub commanded_gear: Option<u8>,
    pub tc_slip: Option<f64>,
    pub line_pressure: Option<f64>,
    pub solenoids: [Option<bool>; 6],
}

pub struct TransmissionPanel {
    pub data: TransmissionData,
    pub fluid_history: Vec<f64>,
    pub last_updated: Option<Instant>,
    pub error: Option<String>,
}

impl TransmissionPanel {
    pub fn new() -> Self {
        Self {
            data: TransmissionData::default(),
            fluid_history: Vec::new(),
            last_updated: None,
            error: None,
        }
    }

    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        self.error = None;
        let d = &mut self.data;

        if let Ok(raw) = executor.read_asbuilt_block(Module::Tcm, 0xDD01).await {
            if !raw.is_empty() {
                d.fluid_temp = Some(celsius_from_raw(raw[0]));
                if let Some(t) = d.fluid_temp {
                    self.fluid_history.push(t);
                    if self.fluid_history.len() > HISTORY_LEN {
                        self.fluid_history.remove(0);
                    }
                }
            }
        } else {
            d.fluid_temp = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Tcm, 0xDD02).await {
            if raw.len() >= 2 {
                d.current_gear = Some(raw[0]);
                d.commanded_gear = Some(raw[1]);
            }
        } else {
            d.current_gear = None;
            d.commanded_gear = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Tcm, 0xDD03).await {
            if raw.len() >= 2 {
                d.tc_slip = Some(u16_be(raw[0], raw[1]) as f64 * 0.1);
            }
        } else {
            d.tc_slip = None;
        }

        self.last_updated = Some(Instant::now());
    }

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

        let temp = d.fluid_temp.unwrap_or(0.0);
        let temp_color = if temp > 110.0 {
            Color::Red
        } else if temp > 90.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        let temp_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Fluid Temp"))
            .gauge_style(Style::default().fg(temp_color))
            .percent(((temp + 40.0) / 150.0 * 100.0).clamp(0.0, 100.0) as u16)
            .label(fmt_temp(d.fluid_temp, 1));
        f.render_widget(temp_gauge, cols[0]);

        let tc_slip = d.tc_slip.unwrap_or(0.0);
        let tc_color = if tc_slip > 50.0 {
            Color::Red
        } else {
            Color::Cyan
        };
        let tc_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("TC Slip"))
            .gauge_style(Style::default().fg(tc_color))
            .percent((tc_slip).clamp(0.0, 100.0) as u16)
            .label(fmt_opt_f(d.tc_slip, 1, "%"));
        f.render_widget(tc_gauge, cols[1]);

        let lines = vec![
            Spans::from(format!(
                "Current Gear: {}   Commanded: {}",
                d.current_gear
                    .map(|g| g.to_string())
                    .unwrap_or_else(|| "--".to_string()),
                d.commanded_gear
                    .map(|g| g.to_string())
                    .unwrap_or_else(|| "--".to_string()),
            )),
            Spans::from(format!(
                "Line Pressure: {}",
                fmt_pressure_kpa(d.line_pressure)
            )),
            Spans::from(Span::raw("")),
            Spans::from(Span::styled(
                "Shift Solenoids",
                Style::default().fg(Color::Gray),
            )),
        ];

        let solenoid_labels = ["SS1", "SS2", "SS3", "SS4", "TCC", "LPC"];
        let solenoid_line = solenoid_labels
            .iter()
            .enumerate()
            .map(|(i, name)| match d.solenoids[i] {
                Some(true) => format!("{}:ON  ", name),
                Some(false) => format!("{}:OFF ", name),
                None => format!("{}:--  ", name),
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut lines = lines;
        lines.push(Spans::from(solenoid_line));

        let info_para = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Transmission"));
        f.render_widget(info_para, rows[1]);
    }

    pub fn help_text(&self) -> &str {
        "Transmission: Fluid Temp · Gear · TC Slip · Line Pressure · Solenoids"
    }
}

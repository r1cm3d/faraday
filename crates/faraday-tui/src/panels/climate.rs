use super::{fmt_opt_f, fmt_pressure_kpa, fmt_temp};
use faraday_core::{commands::CommandExecutor, transport::IsoTpTransport, Module};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::time::Instant;

#[derive(Default)]
pub struct ClimateData {
    pub cabin_temp_driver: Option<f64>,
    pub cabin_temp_pass: Option<f64>,
    pub blend_driver: Option<f64>,
    pub blend_pass: Option<f64>,
    pub evap_temp: Option<f64>,
    pub refrigerant_pressure: Option<f64>,
    pub compressor_load: Option<f64>,
    pub blower_speed: Option<f64>,
}

pub struct ClimatePanel {
    pub data: ClimateData,
    pub last_updated: Option<Instant>,
    pub error: Option<String>,
}

impl ClimatePanel {
    pub fn new() -> Self {
        Self {
            data: ClimateData::default(),
            last_updated: None,
            error: None,
        }
    }

    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        self.error = None;
        let d = &mut self.data;

        if let Ok(raw) = executor.read_asbuilt_block(Module::Hvac, 0xB001).await {
            if raw.len() >= 4 {
                d.blend_driver = Some(raw[0] as f64 * 100.0 / 255.0);
                d.blend_pass = Some(raw[1] as f64 * 100.0 / 255.0);
                d.cabin_temp_driver = Some(raw[2] as f64 - 40.0);
                d.cabin_temp_pass = Some(raw[3] as f64 - 40.0);
            }
        } else {
            d.blend_driver = None;
            d.blend_pass = None;
            d.cabin_temp_driver = None;
            d.cabin_temp_pass = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Hvac, 0xB002).await {
            if raw.len() >= 4 {
                d.evap_temp = Some(raw[0] as f64 - 40.0);
                d.refrigerant_pressure = Some(((raw[1] as u16) << 8 | raw[2] as u16) as f64 * 0.1);
                d.compressor_load = Some(raw[3] as f64 * 100.0 / 255.0);
            }
        } else {
            d.evap_temp = None;
            d.refrigerant_pressure = None;
            d.compressor_load = None;
        }

        self.last_updated = Some(Instant::now());
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let d = &self.data;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(2),
            ])
            .split(area);

        let temp_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let driver_temp = d.cabin_temp_driver.unwrap_or(20.0);
        let driver_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Driver Zone"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(((driver_temp + 20.0) / 60.0 * 100.0).clamp(0.0, 100.0) as u16)
            .label(fmt_temp(d.cabin_temp_driver, 1));
        f.render_widget(driver_gauge, temp_cols[0]);

        let pass_temp = d.cabin_temp_pass.unwrap_or(20.0);
        let pass_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Passenger Zone"),
            )
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(((pass_temp + 20.0) / 60.0 * 100.0).clamp(0.0, 100.0) as u16)
            .label(fmt_temp(d.cabin_temp_pass, 1));
        f.render_widget(pass_gauge, temp_cols[1]);

        let blend_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        let blend_d = d.blend_driver.unwrap_or(0.0);
        let blend_d_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Blend Door Driver"),
            )
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(blend_d as u16)
            .label(fmt_opt_f(d.blend_driver, 1, "%"));
        f.render_widget(blend_d_gauge, blend_cols[0]);

        let blend_p = d.blend_pass.unwrap_or(0.0);
        let blend_p_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Blend Door Pass"),
            )
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(blend_p as u16)
            .label(fmt_opt_f(d.blend_pass, 1, "%"));
        f.render_widget(blend_p_gauge, blend_cols[1]);

        let info_lines = vec![
            Spans::from(format!(
                "Evap Temp: {}  Refrigerant: {}",
                fmt_temp(d.evap_temp, 1),
                fmt_pressure_kpa(d.refrigerant_pressure),
            )),
            Spans::from(format!(
                "Compressor Load: {}  Blower: {}",
                fmt_opt_f(d.compressor_load, 1, "%"),
                fmt_opt_f(d.blower_speed, 0, " rpm")
            )),
        ];
        let info_para = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title("HVAC Status"));
        f.render_widget(info_para, rows[2]);
    }

    pub fn help_text(&self) -> &str {
        "Climate: Cabin Temps · Blend Doors · Evap Temp · Refrigerant Pressure"
    }
}

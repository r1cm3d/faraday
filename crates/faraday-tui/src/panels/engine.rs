use super::{clamp_percent, fmt_fuel_economy, fmt_fuel_rate, fmt_opt_f, fmt_speed_kmh, fmt_temp};
use faraday_core::{
    commands::CommandExecutor, protocol::j1979::Pid, transport::IsoTpTransport, Module,
};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::collections::VecDeque;
use std::time::Instant;
pub const MAX_HISTORY: usize = 200;

#[derive(Clone, Default)]
pub struct EngineSnapshot {
    pub rpm: Option<f64>,
    pub speed: Option<f64>,
    pub coolant_temp: Option<f64>,
    pub engine_load: Option<f64>,
    pub throttle_pos: Option<f64>,
    pub rel_throttle_pos: Option<f64>,
    pub maf_rate: Option<f64>,
    pub intake_temp: Option<f64>,
    pub oil_temp: Option<f64>,
    pub stft_b1: Option<f64>,
    pub ltft_b1: Option<f64>,
    pub stft_b2: Option<f64>,
    pub ltft_b2: Option<f64>,
    pub timing_advance: Option<f64>,
    pub o2_b1s1: Option<f64>,
    pub o2_b1s2: Option<f64>,
    pub egr_commanded: Option<f64>,
    pub egr_error: Option<f64>,
    pub fuel_rate: Option<f64>,
    pub driver_torque: Option<f64>,
    pub engine_torque: Option<f64>,
    pub battery_voltage: Option<f64>,
    pub oil_life: Option<f64>,
    pub misfire_counts: [Option<u32>; 4],
}

pub struct EnginePanel {
    pub history: VecDeque<EngineSnapshot>,
    pub latest: EngineSnapshot,
    pub last_updated: Option<Instant>,
    pub error: Option<String>,
}

impl EnginePanel {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            latest: EngineSnapshot::default(),
            last_updated: None,
            error: None,
        }
    }

    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        self.error = None;
        let mut snap = EngineSnapshot::default();

        if let Ok(values) = executor.read_engine_data().await {
            for v in &values {
                match v.pid {
                    Pid::ENGINE_RPM => snap.rpm = v.interpreted_value,
                    Pid::VEHICLE_SPEED => snap.speed = v.interpreted_value,
                    Pid::COOLANT_TEMP => snap.coolant_temp = v.interpreted_value,
                    Pid::ENGINE_LOAD => snap.engine_load = v.interpreted_value,
                    Pid::THROTTLE_POS => snap.throttle_pos = v.interpreted_value,
                    _ => {}
                }
            }
        } else {
            self.error = Some("Engine data read failed".to_string());
        }

        if let Ok(values) = executor.read_temperature_data().await {
            for v in &values {
                match v.pid {
                    Pid::INTAKE_TEMP => snap.intake_temp = v.interpreted_value,
                    Pid::ENGINE_OIL_TEMP => snap.oil_temp = v.interpreted_value,
                    _ => {}
                }
            }
        }

        if let Ok(values) = executor.read_fuel_trim_data().await {
            for v in &values {
                match v.pid {
                    Pid::SHORT_FUEL_TRIM_B1 => snap.stft_b1 = v.interpreted_value,
                    Pid::LONG_FUEL_TRIM_B1 => snap.ltft_b1 = v.interpreted_value,
                    Pid::SHORT_FUEL_TRIM_B2 => snap.stft_b2 = v.interpreted_value,
                    Pid::LONG_FUEL_TRIM_B2 => snap.ltft_b2 = v.interpreted_value,
                    _ => {}
                }
            }
        }

        if let Ok(values) = executor.read_emissions_data().await {
            for v in &values {
                match v.pid {
                    Pid::TIMING_ADVANCE => snap.timing_advance = v.interpreted_value,
                    Pid::O2_B1S1_VOLTAGE => snap.o2_b1s1 = v.interpreted_value,
                    Pid::O2_B1S2_VOLTAGE => snap.o2_b1s2 = v.interpreted_value,
                    Pid::EGR_COMMANDED => snap.egr_commanded = v.interpreted_value,
                    Pid::EGR_ERROR => snap.egr_error = v.interpreted_value,
                    _ => {}
                }
            }
        }

        if let Ok(values) = executor.read_powertrain_data().await {
            for v in &values {
                match v.pid {
                    Pid::ENGINE_FUEL_RATE => snap.fuel_rate = v.interpreted_value,
                    Pid::DRIVER_DEMAND_TORQUE => snap.driver_torque = v.interpreted_value,
                    Pid::ACTUAL_ENGINE_TORQUE => snap.engine_torque = v.interpreted_value,
                    Pid::REL_THROTTLE_POS => snap.rel_throttle_pos = v.interpreted_value,
                    Pid::CONTROL_MODULE_VOLTAGE => snap.battery_voltage = v.interpreted_value,
                    _ => {}
                }
            }
        }

        if let Ok(data) = executor.read_asbuilt_block(Module::Pcm, 0x1900).await {
            if !data.is_empty() {
                snap.oil_life = Some(data[0] as f64);
            }
        }

        if let Ok(data) = executor.read_asbuilt_block(Module::Pcm, 0x1901).await {
            if data.len() >= 4 {
                snap.misfire_counts[0] = Some(((data[0] as u32) << 8) | data[1] as u32);
                snap.misfire_counts[1] = Some(((data[2] as u32) << 8) | data[3] as u32);
            }
            if data.len() >= 8 {
                snap.misfire_counts[2] = Some(((data[4] as u32) << 8) | data[5] as u32);
                snap.misfire_counts[3] = Some(((data[6] as u32) << 8) | data[7] as u32);
            }
        }

        self.latest = snap.clone();
        self.history.push_back(snap);
        if self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
        self.last_updated = Some(Instant::now());
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let s = &self.latest;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(4),
                Constraint::Length(3),
            ])
            .split(area);

        let gauge_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(rows[0]);

        let rpm = s.rpm.unwrap_or(0.0);
        let rpm_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("RPM"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(clamp_percent(rpm, 7000.0))
            .label(format!("{:.0} rpm", rpm));
        f.render_widget(rpm_gauge, gauge_row[0]);

        let speed = s.speed.unwrap_or(0.0);
        let speed_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Speed"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(clamp_percent(speed, 220.0))
            .label(fmt_speed_kmh(s.speed));
        f.render_widget(speed_gauge, gauge_row[1]);

        let load = s.engine_load.unwrap_or(0.0);
        let load_color = if load > 80.0 {
            Color::Red
        } else {
            Color::Yellow
        };
        let load_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Load"))
            .gauge_style(Style::default().fg(load_color))
            .percent(clamp_percent(load, 100.0))
            .label(format!("{:.1}%", load));
        f.render_widget(load_gauge, gauge_row[2]);

        let info_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        let temps = vec![
            Spans::from(format!(
                "Coolant: {}",
                fmt_temp(s.coolant_temp, 1),
            )),
            Spans::from(format!(
                "Oil:     {}",
                fmt_temp(s.oil_temp, 1),
            )),
            Spans::from(format!(
                "Intake:  {}",
                fmt_temp(s.intake_temp, 1),
            )),
            Spans::from(format!(
                "Throttle: {}  Rel: {}",
                fmt_opt_f(s.throttle_pos, 1, "%"),
                fmt_opt_f(s.rel_throttle_pos, 1, "%"),
            )),
            Spans::from(format!(
                "MAF: {}  Battery: {}",
                fmt_opt_f(s.maf_rate, 2, " g/s"),
                fmt_opt_f(s.battery_voltage, 2, " V")
            )),
            Spans::from(format!(
                "Oil Life: {}  Timing: {}",
                fmt_opt_f(s.oil_life, 0, "%"),
                fmt_opt_f(s.timing_advance, 1, "°")
            )),
        ];
        let temps_para =
            Paragraph::new(temps).block(Block::default().borders(Borders::ALL).title("Sensors"));
        f.render_widget(temps_para, info_cols[0]);

        let emissions = vec![
            Spans::from(format!(
                "STFT B1: {}  LTFT B1: {}",
                fmt_opt_f(s.stft_b1, 1, "%"),
                fmt_opt_f(s.ltft_b1, 1, "%"),
            )),
            Spans::from(format!(
                "STFT B2: {}  LTFT B2: {}",
                fmt_opt_f(s.stft_b2, 1, "%"),
                fmt_opt_f(s.ltft_b2, 1, "%"),
            )),
            Spans::from(format!(
                "O2 B1S1: {}  O2 B1S2: {}",
                fmt_opt_f(s.o2_b1s1, 3, " V"),
                fmt_opt_f(s.o2_b1s2, 3, " V"),
            )),
            Spans::from(format!(
                "EGR Cmd: {}  EGR Err: {}",
                fmt_opt_f(s.egr_commanded, 1, "%"),
                fmt_opt_f(s.egr_error, 1, "%"),
            )),
            Spans::from(format!(
                "Fuel Rate: {}  Torque: {}",
                fmt_fuel_rate(s.fuel_rate),
                fmt_opt_f(s.engine_torque, 0, "%"),
            )),
            Spans::from(format!(
                "Fuel Econ: {}",
                fmt_fuel_economy(s.speed, s.fuel_rate),
            )),
        ];
        let emissions_para = Paragraph::new(emissions).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Emissions & Fuel"),
        );
        f.render_widget(emissions_para, info_cols[1]);

        let mf = &s.misfire_counts;
        let misfire_text = format!(
            "Cyl 1: {}  Cyl 2: {}  Cyl 3: {}  Cyl 4: {}",
            mf[0]
                .map(|v| v.to_string())
                .unwrap_or_else(|| "--".to_string()),
            mf[1]
                .map(|v| v.to_string())
                .unwrap_or_else(|| "--".to_string()),
            mf[2]
                .map(|v| v.to_string())
                .unwrap_or_else(|| "--".to_string()),
            mf[3]
                .map(|v| v.to_string())
                .unwrap_or_else(|| "--".to_string()),
        );
        let misfire_para = Paragraph::new(misfire_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Cylinder Misfires"),
        );
        f.render_widget(misfire_para, rows[2]);
    }

    pub fn help_text(&self) -> &str {
        "Engine: RPM · Speed · Load · Fuel Trim · O2 · EGR · Timing · Misfires"
    }

    pub fn battery_voltage(&self) -> Option<f64> {
        self.latest.battery_voltage
    }
}

//! ABS, ESC, RCM, and TPMS safety diagnostic panel.
use super::{fmt_opt_f, fmt_speed_kmh, fmt_tpms_kpa, u16_be, KPA_PER_PSI};
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

/// Latest ABS, RCM, and TPMS data values read from the CAN bus.
#[derive(Default)]
pub struct SafetyData {
    /// Front-left wheel speed in km/h (ABS DID 0xC001, scaled ×0.1).
    pub wheel_speed_fl: Option<f64>,
    /// Front-right wheel speed in km/h (ABS DID 0xC001, scaled ×0.1).
    pub wheel_speed_fr: Option<f64>,
    /// Rear-left wheel speed in km/h (ABS DID 0xC001, scaled ×0.1).
    pub wheel_speed_rl: Option<f64>,
    /// Rear-right wheel speed in km/h (ABS DID 0xC001, scaled ×0.1).
    pub wheel_speed_rr: Option<f64>,
    /// Yaw rate in °/s (ABS DID 0xC002, signed, scaled ×0.1).
    pub yaw_rate: Option<f64>,
    /// Lateral acceleration in g (ABS DID 0xC002, signed, scaled ×0.01).
    pub lateral_accel: Option<f64>,
    /// Airbag squib continuity bitmask (RCM DID 0xD001; 0xFF = all OK).
    pub airbag_squib: Option<u8>,
    /// Seatbelt-buckled bitmask: bit N = occupant N buckled (RCM DID 0xD002).
    pub seatbelt_status: Option<u8>,
    /// Front axle TPMS pressure in kPa (BCM DID 0x0201, scaled by KPA_PER_PSI).
    pub tpms_front_kpa: Option<f64>,
    /// Rear axle TPMS pressure in kPa (BCM DID 0x0201, scaled by KPA_PER_PSI).
    pub tpms_rear_kpa: Option<f64>,
}

/// Live safety systems diagnostic panel (ABS, ESC, RCM, TPMS).
pub struct SafetyPanel {
    /// Most recent safety data values.
    pub data: SafetyData,
    /// Timestamp of the last successful update.
    pub last_updated: Option<Instant>,
    /// Error message from the most recent failed update, if any.
    pub error: Option<String>,
}

impl SafetyPanel {
    /// Create a new `SafetyPanel` with empty state.
    pub fn new() -> Self {
        Self {
            data: SafetyData::default(),
            last_updated: None,
            error: None,
        }
    }

    /// Poll ABS, RCM, and BCM DIDs and update all safety data fields.
    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        self.error = None;
        let d = &mut self.data;

        if let Ok(raw) = executor.read_asbuilt_block(Module::Abs, 0xC001).await {
            if raw.len() >= 8 {
                d.wheel_speed_fl = Some(u16_be(raw[0], raw[1]) as f64 * 0.1);
                d.wheel_speed_fr = Some(u16_be(raw[2], raw[3]) as f64 * 0.1);
                d.wheel_speed_rl = Some(u16_be(raw[4], raw[5]) as f64 * 0.1);
                d.wheel_speed_rr = Some(u16_be(raw[6], raw[7]) as f64 * 0.1);
            }
        } else {
            d.wheel_speed_fl = None;
            d.wheel_speed_fr = None;
            d.wheel_speed_rl = None;
            d.wheel_speed_rr = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Abs, 0xC002).await {
            if raw.len() >= 4 {
                d.yaw_rate = Some(u16_be(raw[0], raw[1]) as i16 as f64 * 0.1);
                d.lateral_accel = Some(u16_be(raw[2], raw[3]) as i16 as f64 * 0.01);
            }
        } else {
            d.yaw_rate = None;
            d.lateral_accel = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Rcm, 0xD001).await {
            if !raw.is_empty() {
                d.airbag_squib = Some(raw[0]);
            }
        } else {
            d.airbag_squib = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Rcm, 0xD002).await {
            if !raw.is_empty() {
                d.seatbelt_status = Some(raw[0]);
            }
        } else {
            d.seatbelt_status = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Bcm, 0x0201).await {
            if raw.len() >= 2 {
                d.tpms_front_kpa = Some(raw[0] as f64 * KPA_PER_PSI);
                d.tpms_rear_kpa = Some(raw[1] as f64 * KPA_PER_PSI);
            }
        } else {
            d.tpms_front_kpa = None;
            d.tpms_rear_kpa = None;
        }

        self.last_updated = Some(Instant::now());
    }

    /// Render the safety panel into `area`.
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let d = &self.data;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ])
            .split(area);

        let wheel_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let front_text = vec![
            Spans::from("── Front Axle ──"),
            Spans::from(format!("FL: {}", fmt_speed_kmh(d.wheel_speed_fl))),
            Spans::from(format!("FR: {}", fmt_speed_kmh(d.wheel_speed_fr))),
            Spans::from(Span::raw("")),
            Spans::from(format!(
                "Yaw Rate:     {}",
                fmt_opt_f(d.yaw_rate, 1, " °/s")
            )),
            Spans::from(format!(
                "Lateral Accel:{}",
                fmt_opt_f(d.lateral_accel, 2, " g")
            )),
        ];
        let front_para = Paragraph::new(front_text)
            .block(Block::default().borders(Borders::ALL).title("ABS / ESC"));
        f.render_widget(front_para, wheel_cols[0]);

        let rear_text = vec![
            Spans::from("── Rear Axle ──"),
            Spans::from(format!("RL: {}", fmt_speed_kmh(d.wheel_speed_rl))),
            Spans::from(format!("RR: {}", fmt_speed_kmh(d.wheel_speed_rr))),
        ];
        let rear_para = Paragraph::new(rear_text)
            .block(Block::default().borders(Borders::ALL).title("Wheel Speeds"));
        f.render_widget(rear_para, wheel_cols[1]);

        let seatbelt_bits = d.seatbelt_status.unwrap_or(0);
        let positions = ["Driver", "Passenger", "RL", "RR"];
        let belt_lines: Vec<Spans> = positions
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if d.seatbelt_status.is_none() {
                    Spans::from(format!("{}: --", name))
                } else {
                    let buckled = (seatbelt_bits >> i) & 1 == 1;
                    let (status, color) = if buckled {
                        ("BUCKLED", Color::Green)
                    } else {
                        ("open   ", Color::Yellow)
                    };
                    Spans::from(vec![
                        Span::raw(format!("{}: ", name)),
                        Span::styled(status, Style::default().fg(color)),
                    ])
                }
            })
            .collect();

        let squib_status = match d.airbag_squib {
            Some(0xFF) => Span::styled("All OK", Style::default().fg(Color::Green)),
            Some(v) => Span::styled(
                format!("Fault bitmask: {:02X}", v),
                Style::default().fg(Color::Red),
            ),
            None => Span::styled("--", Style::default().fg(Color::DarkGray)),
        };

        let mut rcm_lines: Vec<Spans> = belt_lines;
        rcm_lines.push(Spans::from(Span::raw("")));
        rcm_lines.push(Spans::from(vec![Span::raw("Squib: "), squib_status]));

        let rcm_para = Paragraph::new(rcm_lines)
            .block(Block::default().borders(Borders::ALL).title("RCM / Airbag"));
        f.render_widget(rcm_para, rows[1]);

        let tpms_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[2]);

        for (label, val, col) in [
            ("Front", d.tpms_front_kpa, tpms_cols[0]),
            ("Rear", d.tpms_rear_kpa, tpms_cols[1]),
        ] {
            let color = match val {
                Some(p) if !(172.0..=310.0).contains(&p) => Color::Red,
                Some(p) if p < 207.0 => Color::Yellow,
                Some(_) => Color::Green,
                None => Color::DarkGray,
            };
            let percent = val
                .map(|p| (p / 345.0 * 100.0).clamp(0.0, 100.0) as u16)
                .unwrap_or(0);
            let label_str = match val {
                Some(_) => format!("{}: {}", label, fmt_tpms_kpa(val)),
                None => format!("{}: --", label),
            };
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("TPMS"))
                .gauge_style(Style::default().fg(color))
                .percent(percent)
                .label(label_str);
            f.render_widget(gauge, col);
        }
    }

    /// Return the context-sensitive help string for this panel.
    pub fn help_text(&self) -> &str {
        "Safety: Wheel Speeds · Yaw Rate · Lateral Accel · Seatbelts · Airbag Squib · TPMS Front/Rear"
    }
}

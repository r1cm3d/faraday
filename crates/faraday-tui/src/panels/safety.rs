use super::fmt_opt_f;
use faraday_core::{commands::CommandExecutor, transport::IsoTpTransport, Module};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Instant;

#[derive(Default)]
pub struct SafetyData {
    pub wheel_speed_fl: Option<f64>,
    pub wheel_speed_fr: Option<f64>,
    pub wheel_speed_rl: Option<f64>,
    pub wheel_speed_rr: Option<f64>,
    pub yaw_rate: Option<f64>,
    pub lateral_accel: Option<f64>,
    pub airbag_squib: Option<u8>,
    pub seatbelt_status: Option<u8>,
}

pub struct SafetyPanel {
    pub data: SafetyData,
    pub last_updated: Option<Instant>,
    pub error: Option<String>,
}

impl SafetyPanel {
    pub fn new() -> Self {
        Self {
            data: SafetyData::default(),
            last_updated: None,
            error: None,
        }
    }

    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        self.error = None;
        let d = &mut self.data;

        if let Ok(raw) = executor.read_asbuilt_block(Module::Abs, 0xC001).await {
            if raw.len() >= 8 {
                d.wheel_speed_fl = Some(((raw[0] as u16) << 8 | raw[1] as u16) as f64 * 0.1);
                d.wheel_speed_fr = Some(((raw[2] as u16) << 8 | raw[3] as u16) as f64 * 0.1);
                d.wheel_speed_rl = Some(((raw[4] as u16) << 8 | raw[5] as u16) as f64 * 0.1);
                d.wheel_speed_rr = Some(((raw[6] as u16) << 8 | raw[7] as u16) as f64 * 0.1);
            }
        } else {
            d.wheel_speed_fl = None;
            d.wheel_speed_fr = None;
            d.wheel_speed_rl = None;
            d.wheel_speed_rr = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Abs, 0xC002).await {
            if raw.len() >= 4 {
                d.yaw_rate = Some(((raw[0] as i16) << 8 | raw[1] as i16) as f64 * 0.1);
                d.lateral_accel = Some(((raw[2] as i16) << 8 | raw[3] as i16) as f64 * 0.01);
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

        self.last_updated = Some(Instant::now());
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let d = &self.data;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let wheel_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let front_text = vec![
            Spans::from("── Front Axle ──"),
            Spans::from(format!("FL: {} km/h", fmt_opt_f(d.wheel_speed_fl, 1, ""))),
            Spans::from(format!("FR: {} km/h", fmt_opt_f(d.wheel_speed_fr, 1, ""))),
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
            Spans::from(format!("RL: {} km/h", fmt_opt_f(d.wheel_speed_rl, 1, ""))),
            Spans::from(format!("RR: {} km/h", fmt_opt_f(d.wheel_speed_rr, 1, ""))),
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
    }

    pub fn help_text(&self) -> &str {
        "Safety: Wheel Speeds · Yaw Rate · Lateral Accel · Seatbelts · Airbag Squib"
    }
}

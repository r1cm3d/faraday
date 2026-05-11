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
pub struct InfotainmentData {
    pub gps_satellites: Option<u8>,
    pub gps_fix: Option<u8>,
    pub cellular_rssi: Option<i8>,
    pub bt_device_count: Option<u8>,
    pub software_version: Option<String>,
}

pub struct InfotainmentPanel {
    pub data: InfotainmentData,
    pub last_updated: Option<Instant>,
    pub error: Option<String>,
}

impl InfotainmentPanel {
    pub fn new() -> Self {
        Self {
            data: InfotainmentData::default(),
            last_updated: None,
            error: None,
        }
    }

    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        self.error = None;
        let d = &mut self.data;

        if let Ok(raw) = executor.read_asbuilt_block(Module::Apim, 0xA001).await {
            if raw.len() >= 2 {
                d.gps_fix = Some(raw[0]);
                d.gps_satellites = Some(raw[1]);
            }
        } else {
            d.gps_fix = None;
            d.gps_satellites = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Apim, 0xA002).await {
            if !raw.is_empty() {
                d.cellular_rssi = Some(raw[0] as i8);
            }
        } else {
            d.cellular_rssi = None;
        }

        if let Ok(raw) = executor.read_asbuilt_block(Module::Apim, 0xA010).await {
            if !raw.is_empty() {
                d.software_version = String::from_utf8(raw)
                    .ok()
                    .map(|s| s.trim_end_matches('\0').to_string());
            }
        } else {
            d.software_version = None;
        }

        self.last_updated = Some(Instant::now());
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let d = &self.data;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(4)])
            .split(area);

        let sig_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let sats = d.gps_satellites.unwrap_or(0) as f64;
        let gps_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("GPS Satellites"),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .percent((sats / 15.0 * 100.0).clamp(0.0, 100.0) as u16)
            .label(
                d.gps_satellites
                    .map(|s| format!("{} sats", s))
                    .unwrap_or_else(|| "--".to_string()),
            );
        f.render_widget(gps_gauge, sig_cols[0]);

        let rssi = d.cellular_rssi.unwrap_or(-120) as f64;
        let rssi_pct = ((rssi + 120.0) / 70.0 * 100.0).clamp(0.0, 100.0);
        let rssi_color = if rssi > -70.0 {
            Color::Green
        } else if rssi > -90.0 {
            Color::Yellow
        } else {
            Color::Red
        };
        let cell_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Cellular RSSI"),
            )
            .gauge_style(Style::default().fg(rssi_color))
            .percent(rssi_pct as u16)
            .label(
                d.cellular_rssi
                    .map(|r| format!("{} dBm", r))
                    .unwrap_or_else(|| "--".to_string()),
            );
        f.render_widget(cell_gauge, sig_cols[1]);

        let gps_fix_str = match d.gps_fix {
            Some(0) => "No Fix",
            Some(1) => "GPS Fix",
            Some(2) => "DGPS Fix",
            Some(3) => "RTK Fix",
            Some(_) => "Unknown",
            None => "--",
        };

        let info_lines = vec![
            Spans::from(vec![
                Span::raw("GPS Fix: "),
                Span::styled(
                    gps_fix_str,
                    Style::default().fg(match d.gps_fix {
                        Some(v) if v > 0 => Color::Green,
                        Some(_) => Color::Red,
                        None => Color::DarkGray,
                    }),
                ),
            ]),
            Spans::from(format!(
                "Bluetooth Devices: {}",
                d.bt_device_count
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "--".to_string())
            )),
            Spans::from(Span::raw("")),
            Spans::from(format!(
                "APIM Software: {}",
                d.software_version.as_deref().unwrap_or("--")
            )),
        ];
        let info_para = Paragraph::new(info_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("SYNC / Infotainment"),
        );
        f.render_widget(info_para, rows[1]);
    }

    pub fn help_text(&self) -> &str {
        "Infotainment: GPS · Cellular Signal · Bluetooth · APIM Software Version"
    }
}

//! APIM/SYNC infotainment diagnostic panel.
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

const GPS_SAT_MAX: f64 = 15.0;
const RSSI_FLOOR_DBM: f64 = -120.0;

/// Latest APIM data values read from the CAN bus.
#[derive(Default)]
pub struct InfotainmentData {
    /// Number of GPS satellites in view (APIM DID 0xA001 byte 1).
    pub gps_satellites: Option<u8>,
    /// GPS fix type: 0=none, 1=GPS, 2=DGPS, 3=RTK (APIM DID 0xA001 byte 0).
    pub gps_fix: Option<u8>,
    /// Cellular RSSI in dBm (APIM DID 0xA002, signed byte).
    pub cellular_rssi: Option<i8>,
    /// Number of paired Bluetooth devices (not yet populated).
    pub bt_device_count: Option<u8>,
    /// SYNC software version string (APIM DID 0xA010, null-terminated UTF-8).
    pub software_version: Option<String>,
}

/// Live APIM/SYNC infotainment diagnostic panel.
pub struct InfotainmentPanel {
    /// Most recent APIM data values.
    pub data: InfotainmentData,
    /// Timestamp of the last successful update.
    pub last_updated: Option<Instant>,
    /// Error message from the most recent failed update, if any.
    pub error: Option<String>,
}

impl InfotainmentPanel {
    /// Create a new `InfotainmentPanel` with empty state.
    pub fn new() -> Self {
        Self {
            data: InfotainmentData::default(),
            last_updated: None,
            error: None,
        }
    }

    /// Poll APIM DIDs and update GPS, cellular, and software version fields.
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

    /// Render the infotainment panel into `area`.
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
            .percent((sats / GPS_SAT_MAX * 100.0).clamp(0.0, 100.0) as u16)
            .label(
                d.gps_satellites
                    .map(|s| format!("{} sats", s))
                    .unwrap_or_else(|| "--".to_string()),
            );
        f.render_widget(gps_gauge, sig_cols[0]);

        let rssi = d.cellular_rssi.map(|r| r as f64).unwrap_or(RSSI_FLOOR_DBM);
        let rssi_pct = ((rssi - RSSI_FLOOR_DBM) / 70.0 * 100.0).clamp(0.0, 100.0);
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

    /// Return the context-sensitive help string for this panel.
    pub fn help_text(&self) -> &str {
        "Infotainment: GPS · Cellular Signal · Bluetooth · APIM Software Version"
    }
}

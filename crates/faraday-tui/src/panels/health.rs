//! Per-module heartbeat connectivity status panel.
use faraday_core::{commands::CommandExecutor, transport::IsoTpTransport, Module};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use std::time::Instant;

const HEARTBEAT_DID: u16 = 0xF101;
const STALE_SECS: u64 = 30;
const TIMEOUT_THRESHOLD: u32 = 3;

static MONITORED: &[Module] = &[
    Module::Pcm,
    Module::Tcm,
    Module::Abs,
    Module::Rcm,
    Module::Bcm,
    Module::Ipc,
    Module::Apim,
    Module::Pam,
    Module::Hvac,
];

/// Connectivity state of a single CAN module.
#[derive(Clone)]
pub struct ModuleStatus {
    /// The ECU module being monitored.
    pub module: Module,
    /// Timestamp of the last successful heartbeat read, or `None` if never seen.
    pub last_seen: Option<Instant>,
    /// Number of consecutive failed heartbeat reads since the last success.
    pub consecutive_timeouts: u32,
}

impl ModuleStatus {
    fn new(module: Module) -> Self {
        Self {
            module,
            last_seen: None,
            consecutive_timeouts: 0,
        }
    }

    fn status_label(&self) -> (&str, Color) {
        match self.last_seen {
            None => ("UNKNOWN", Color::DarkGray),
            Some(t) => {
                let age = t.elapsed().as_secs();
                if self.consecutive_timeouts >= TIMEOUT_THRESHOLD {
                    ("TIMEOUT", Color::Red)
                } else if age > STALE_SECS {
                    ("STALE", Color::Yellow)
                } else {
                    ("OK", Color::Green)
                }
            }
        }
    }

    fn age_str(&self) -> String {
        match self.last_seen {
            None => "--".to_string(),
            Some(t) => {
                let secs = t.elapsed().as_secs();
                if secs < 60 {
                    format!("{}s ago", secs)
                } else {
                    format!("{}m ago", secs / 60)
                }
            }
        }
    }
}

/// Live per-module heartbeat connectivity status panel.
pub struct HealthPanel {
    /// Status records for all monitored CAN modules.
    pub modules: Vec<ModuleStatus>,
    /// Round-robin index of the module polled on the next update cycle.
    pub poll_index: usize,
    /// Timestamp of the last update cycle.
    pub last_updated: Option<Instant>,
}

impl HealthPanel {
    /// Create a new `HealthPanel` monitoring all entries in `MONITORED`.
    pub fn new() -> Self {
        let modules = MONITORED.iter().map(|&m| ModuleStatus::new(m)).collect();
        Self {
            modules,
            poll_index: 0,
            last_updated: None,
        }
    }

    /// Poll one module per call (round-robin) and update its heartbeat status.
    pub async fn update<T: IsoTpTransport>(&mut self, executor: &mut CommandExecutor<T>) {
        let idx = self.poll_index % self.modules.len();
        let module = self.modules[idx].module;

        match executor.read_asbuilt_block(module, HEARTBEAT_DID).await {
            Ok(_) => {
                self.modules[idx].last_seen = Some(Instant::now());
                self.modules[idx].consecutive_timeouts = 0;
            }
            Err(_) => {
                self.modules[idx].consecutive_timeouts += 1;
            }
        }

        self.poll_index += 1;
        self.last_updated = Some(Instant::now());
    }

    /// Mark the PCM as seen and reset its timeout counter, called after a successful engine update.
    pub fn record_engine_ok(&mut self) {
        if let Some(m) = self.modules.iter_mut().find(|m| m.module == Module::Pcm) {
            m.last_seen = Some(Instant::now());
            m.consecutive_timeouts = 0;
        }
    }

    /// Render the module health table into `area`.
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let header = Row::new(vec![
            Cell::from("Module").style(Style::default().fg(Color::White)),
            Cell::from("Status").style(Style::default().fg(Color::White)),
            Cell::from("Last Seen").style(Style::default().fg(Color::White)),
            Cell::from("Timeouts").style(Style::default().fg(Color::White)),
        ]);

        let rows: Vec<Row> = self
            .modules
            .iter()
            .map(|m| {
                let (status_label, status_color) = m.status_label();
                Row::new(vec![
                    Cell::from(m.module.name()),
                    Cell::from(Span::styled(
                        status_label,
                        Style::default().fg(status_color),
                    )),
                    Cell::from(m.age_str()),
                    Cell::from(m.consecutive_timeouts.to_string()),
                ])
            })
            .collect();

        let table = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Module Health"),
            )
            .widths(&[
                ratatui::layout::Constraint::Length(8),
                ratatui::layout::Constraint::Length(10),
                ratatui::layout::Constraint::Length(12),
                ratatui::layout::Constraint::Length(10),
            ]);

        f.render_widget(table, area);
    }

    /// Return the context-sensitive help string for this panel.
    pub fn help_text(&self) -> &str {
        "Health: Module connectivity status, last seen time, consecutive timeout count"
    }
}

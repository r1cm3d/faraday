//! Application state and tab navigation for the diagnostic TUI.
use anyhow::Result;
use faraday_core::{commands::CommandExecutor, link::vlinker::VLinkerFs, transport::isotp::IsoTp};
use std::time::{Duration, Instant};

use crate::panels::{
    AdasPanel, AnalyticsPanel, BodyPanel, ClimatePanel, EnginePanel, GlossaryPanel, HealthPanel,
    InfotainmentPanel, SafetyPanel, TransmissionPanel,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveTab {
    Engine = 0,
    Transmission = 1,
    Body = 2,
    Safety = 3,
    Adas = 4,
    Climate = 5,
    Infotainment = 6,
    Analytics = 7,
    Health = 8,
    Glossary = 9,
}

pub const N_TABS: usize = 10;

impl ActiveTab {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => ActiveTab::Engine,
            1 => ActiveTab::Transmission,
            2 => ActiveTab::Body,
            3 => ActiveTab::Safety,
            4 => ActiveTab::Adas,
            5 => ActiveTab::Climate,
            6 => ActiveTab::Infotainment,
            7 => ActiveTab::Analytics,
            8 => ActiveTab::Health,
            _ => ActiveTab::Glossary,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn next(self) -> Self {
        ActiveTab::from_index((self.index() + 1) % N_TABS)
    }

    pub fn prev(self) -> Self {
        ActiveTab::from_index((self.index() + N_TABS - 1) % N_TABS)
    }

    pub fn poll_interval(self) -> Duration {
        match self {
            ActiveTab::Engine => Duration::from_millis(250),
            ActiveTab::Transmission => Duration::from_millis(500),
            ActiveTab::Body | ActiveTab::Safety | ActiveTab::Adas | ActiveTab::Climate => {
                Duration::from_secs(1)
            }
            ActiveTab::Infotainment | ActiveTab::Analytics | ActiveTab::Health => {
                Duration::from_secs(5)
            }
            ActiveTab::Glossary => Duration::from_secs(60),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            ActiveTab::Engine => "1:Engine",
            ActiveTab::Transmission => "2:Trans",
            ActiveTab::Body => "3:Body",
            ActiveTab::Safety => "4:Safety",
            ActiveTab::Adas => "5:ADAS",
            ActiveTab::Climate => "6:Climate",
            ActiveTab::Infotainment => "7:Info",
            ActiveTab::Analytics => "8:Analytics",
            ActiveTab::Health => "9:Health",
            ActiveTab::Glossary => "0:Glossary",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

pub struct App {
    executor: CommandExecutor<IsoTp<VLinkerFs>>,
    pub active_tab: ActiveTab,
    pub paused: bool,
    pub connection_status: ConnectionStatus,
    pub error_message: Option<String>,
    last_panel_update: [Option<Instant>; N_TABS],

    pub engine: EnginePanel,
    pub transmission: TransmissionPanel,
    pub body: BodyPanel,
    pub safety: SafetyPanel,
    pub adas: AdasPanel,
    pub climate: ClimatePanel,
    pub infotainment: InfotainmentPanel,
    pub analytics: AnalyticsPanel,
    pub health: HealthPanel,
    pub glossary: GlossaryPanel,
}

impl App {
    pub async fn new(adapter_path: String) -> Result<Self> {
        let vlinker = VLinkerFs::with_port_name(&adapter_path)?;
        let isotp = IsoTp::new(vlinker);
        let executor = CommandExecutor::new(isotp);

        Ok(Self {
            executor,
            active_tab: ActiveTab::Engine,
            paused: false,
            connection_status: ConnectionStatus::Disconnected,
            error_message: None,
            last_panel_update: [None; N_TABS],
            engine: EnginePanel::new(),
            transmission: TransmissionPanel::new(),
            body: BodyPanel::new(),
            safety: SafetyPanel::new(),
            adas: AdasPanel::new(),
            climate: ClimatePanel::new(),
            infotainment: InfotainmentPanel::new(),
            analytics: AnalyticsPanel::new(),
            health: HealthPanel::new(),
            glossary: GlossaryPanel::new(),
        })
    }

    pub async fn on_tick(&mut self) -> Result<()> {
        if self.paused {
            return Ok(());
        }

        let now = Instant::now();
        let tab = self.active_tab;
        let interval = tab.poll_interval();
        let due = self.last_panel_update[tab.index()]
            .map_or(true, |t| now.duration_since(t) >= interval);

        if due {
            self.update_active_panel().await;
            self.last_panel_update[tab.index()] = Some(Instant::now());
        }

        Ok(())
    }

    async fn update_active_panel(&mut self) {
        self.connection_status = ConnectionStatus::Connecting;
        self.error_message = None;

        match self.active_tab {
            ActiveTab::Engine => {
                self.engine.update(&mut self.executor).await;
                self.health.record_engine_ok();
                if self.engine.error.is_some() {
                    self.connection_status = ConnectionStatus::Error;
                    self.error_message = self.engine.error.clone();
                } else {
                    self.connection_status = ConnectionStatus::Connected;
                    let snap = self.engine.latest.clone();
                    self.analytics.ingest(&snap, 0.25);
                }
            }
            ActiveTab::Transmission => {
                self.transmission.update(&mut self.executor).await;
                self.connection_status = if self.transmission.error.is_some() {
                    ConnectionStatus::Error
                } else {
                    ConnectionStatus::Connected
                };
            }
            ActiveTab::Body => {
                self.body.update(&mut self.executor).await;
                self.connection_status = if self.body.error.is_some() {
                    ConnectionStatus::Error
                } else {
                    ConnectionStatus::Connected
                };
            }
            ActiveTab::Safety => {
                self.safety.update(&mut self.executor).await;
                self.connection_status = if self.safety.error.is_some() {
                    ConnectionStatus::Error
                } else {
                    ConnectionStatus::Connected
                };
            }
            ActiveTab::Adas => {
                self.adas.update(&mut self.executor).await;
                self.connection_status = if self.adas.error.is_some() {
                    ConnectionStatus::Error
                } else {
                    ConnectionStatus::Connected
                };
            }
            ActiveTab::Climate => {
                self.climate.update(&mut self.executor).await;
                self.connection_status = if self.climate.error.is_some() {
                    ConnectionStatus::Error
                } else {
                    ConnectionStatus::Connected
                };
            }
            ActiveTab::Infotainment => {
                self.infotainment.update(&mut self.executor).await;
                self.connection_status = if self.infotainment.error.is_some() {
                    ConnectionStatus::Error
                } else {
                    ConnectionStatus::Connected
                };
            }
            ActiveTab::Analytics => {
                self.connection_status = ConnectionStatus::Connected;
            }
            ActiveTab::Health => {
                self.health.update(&mut self.executor).await;
                self.connection_status = ConnectionStatus::Connected;
            }
            ActiveTab::Glossary => {
                self.connection_status = ConnectionStatus::Connected;
            }
        }
    }

    pub fn goto_tab(&mut self, tab: ActiveTab) {
        self.active_tab = tab;
    }

    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = self.active_tab.prev();
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn reset_data(&mut self) {
        self.engine = EnginePanel::new();
        self.transmission = TransmissionPanel::new();
        self.body = BodyPanel::new();
        self.safety = SafetyPanel::new();
        self.adas = AdasPanel::new();
        self.climate = ClimatePanel::new();
        self.infotainment = InfotainmentPanel::new();
        self.analytics = AnalyticsPanel::new();
        self.health = HealthPanel::new();
        self.glossary = GlossaryPanel::new();
        self.last_panel_update = [None; N_TABS];
        self.error_message = None;
    }

    pub fn battery_voltage(&self) -> Option<f64> {
        self.engine.battery_voltage()
    }

    pub fn active_help_text(&self) -> &str {
        match self.active_tab {
            ActiveTab::Engine => self.engine.help_text(),
            ActiveTab::Transmission => self.transmission.help_text(),
            ActiveTab::Body => self.body.help_text(),
            ActiveTab::Safety => self.safety.help_text(),
            ActiveTab::Adas => self.adas.help_text(),
            ActiveTab::Climate => self.climate.help_text(),
            ActiveTab::Infotainment => self.infotainment.help_text(),
            ActiveTab::Analytics => self.analytics.help_text(),
            ActiveTab::Health => self.health.help_text(),
            ActiveTab::Glossary => self.glossary.help_text(),
        }
    }

    pub fn scroll_glossary_up(&mut self) {
        self.glossary.scroll_up();
    }

    pub fn scroll_glossary_down(&mut self) {
        self.glossary.scroll_down();
    }
}

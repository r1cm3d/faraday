use anyhow::Result;
use faraday_core::{
    commands::CommandExecutor, link::vlinker::VLinkerFs, protocol::j1979::Pid,
    transport::isotp::IsoTp, Module,
};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct App {
    executor: CommandExecutor<IsoTp<VLinkerFs>>,
    update_interval: Duration,
    last_update: Instant,
    paused: bool,

    // Data storage
    engine_data: VecDeque<EngineSnapshot>,
    max_history: usize,

    // Status
    connection_status: ConnectionStatus,
    error_message: Option<String>,
}

#[derive(Clone)]
pub struct EngineSnapshot {
    #[allow(dead_code)]
    pub timestamp: Instant,
    pub rpm: Option<f64>,
    pub speed: Option<f64>,
    pub coolant_temp: Option<f64>,
    pub engine_load: Option<f64>,
    pub throttle_position: Option<f64>,
}

#[derive(Clone, Copy)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl App {
    pub async fn new(adapter_path: String, update_interval: Duration) -> Result<Self> {
        let vlinker = VLinkerFs::with_port_name(&adapter_path)?;
        let isotp = IsoTp::new(vlinker);
        let executor = CommandExecutor::new(isotp);

        Ok(Self {
            executor,
            update_interval,
            last_update: Instant::now(),
            paused: false,
            engine_data: VecDeque::new(),
            max_history: 200, // Keep 200 data points
            connection_status: ConnectionStatus::Disconnected,
            error_message: None,
        })
    }

    pub async fn on_tick(&mut self) -> Result<()> {
        if self.paused {
            return Ok(());
        }

        if self.last_update.elapsed() >= self.update_interval {
            self.update_data().await;
            self.last_update = Instant::now();
        }

        Ok(())
    }

    async fn update_data(&mut self) {
        self.connection_status = ConnectionStatus::Connecting;
        self.error_message = None;

        match self.fetch_engine_data().await {
            Ok(snapshot) => {
                self.connection_status = ConnectionStatus::Connected;
                self.add_snapshot(snapshot);
            }
            Err(e) => {
                self.connection_status = ConnectionStatus::Error;
                self.error_message = Some(format!("Error: {}", e));
            }
        }
    }

    async fn fetch_engine_data(&mut self) -> Result<EngineSnapshot> {
        let pids = vec![
            Pid::ENGINE_RPM,
            Pid::VEHICLE_SPEED,
            Pid::COOLANT_TEMP,
            Pid::ENGINE_LOAD,
            Pid::THROTTLE_POS,
        ];

        let values = self.executor.read_live_data(Module::Pcm, &pids).await?;

        let mut snapshot = EngineSnapshot {
            timestamp: Instant::now(),
            rpm: None,
            speed: None,
            coolant_temp: None,
            engine_load: None,
            throttle_position: None,
        };

        for value in values {
            match value.pid {
                Pid::ENGINE_RPM => snapshot.rpm = value.interpreted_value,
                Pid::VEHICLE_SPEED => snapshot.speed = value.interpreted_value,
                Pid::COOLANT_TEMP => snapshot.coolant_temp = value.interpreted_value,
                Pid::ENGINE_LOAD => snapshot.engine_load = value.interpreted_value,
                Pid::THROTTLE_POS => snapshot.throttle_position = value.interpreted_value,
                _ => {}
            }
        }

        Ok(snapshot)
    }

    fn add_snapshot(&mut self, snapshot: EngineSnapshot) {
        self.engine_data.push_back(snapshot);

        while self.engine_data.len() > self.max_history {
            self.engine_data.pop_front();
        }
    }

    pub fn reset_data(&mut self) {
        self.engine_data.clear();
        self.error_message = None;
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        self.connection_status
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn engine_data(&self) -> &VecDeque<EngineSnapshot> {
        &self.engine_data
    }

    pub fn latest_snapshot(&self) -> Option<&EngineSnapshot> {
        self.engine_data.back()
    }

    pub fn data_points_count(&self) -> usize {
        self.engine_data.len()
    }
}

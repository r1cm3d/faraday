use super::CommandExecutor;
use crate::{
    protocol::j1979::{J1979, Pid, PidValue},
    transport::IsoTpTransport,
    Module, Result,
};
use tracing::info;

impl<T: IsoTpTransport> CommandExecutor<T> {
    pub async fn read_live_data(&mut self, module: Module, pids: &[Pid]) -> Result<Vec<PidValue>> {
        info!("Reading live data from {:?} for PIDs: {:?}", module, pids);
        let mut j1979 = J1979::new(&mut self.transport);
        j1979.read_live_data(module.response_id(), pids).await
    }

    pub async fn read_engine_data(&mut self) -> Result<Vec<PidValue>> {
        let pids = vec![
            Pid::ENGINE_RPM,
            Pid::ENGINE_LOAD,
            Pid::COOLANT_TEMP,
            Pid::VEHICLE_SPEED,
            Pid::THROTTLE_POS,
        ];
        self.read_live_data(Module::Pcm, &pids).await
    }

    pub async fn read_fuel_data(&mut self) -> Result<Vec<PidValue>> {
        let pids = vec![
            Pid::FUEL_TANK_LEVEL,
            Pid::MAF_RATE,
        ];
        self.read_live_data(Module::Pcm, &pids).await
    }

    pub async fn read_temperature_data(&mut self) -> Result<Vec<PidValue>> {
        let pids = vec![
            Pid::COOLANT_TEMP,
            Pid::INTAKE_TEMP,
            Pid::AMBIENT_TEMP,
            Pid::ENGINE_OIL_TEMP,
        ];
        self.read_live_data(Module::Pcm, &pids).await
    }
}
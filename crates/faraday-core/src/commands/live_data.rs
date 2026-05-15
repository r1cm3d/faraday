use super::CommandExecutor;
use crate::{
    protocol::j1979::{Pid, PidValue, J1979},
    transport::IsoTpTransport,
    Module, Result,
};
use tracing::info;

impl<T: IsoTpTransport> CommandExecutor<T> {
    /// Reads live OBD-II data for the given set of PIDs from `module`.
    pub async fn read_live_data(&mut self, module: Module, pids: &[Pid]) -> Result<Vec<PidValue>> {
        info!("Reading live data from {:?} for PIDs: {:?}", module, pids);
        let mut j1979 = J1979::new(&mut self.transport);
        j1979.read_live_data(module.response_id(), pids).await
    }

    /// Reads the core engine PIDs: RPM, load, coolant temp, vehicle speed, throttle.
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

    /// Reads fuel-related PIDs: tank level and MAF rate.
    pub async fn read_fuel_data(&mut self) -> Result<Vec<PidValue>> {
        let pids = vec![Pid::FUEL_TANK_LEVEL, Pid::MAF_RATE];
        self.read_live_data(Module::Pcm, &pids).await
    }

    /// Reads all temperature PIDs: coolant, intake, ambient, and engine oil.
    pub async fn read_temperature_data(&mut self) -> Result<Vec<PidValue>> {
        let pids = vec![
            Pid::COOLANT_TEMP,
            Pid::INTAKE_TEMP,
            Pid::AMBIENT_TEMP,
            Pid::ENGINE_OIL_TEMP,
        ];
        self.read_live_data(Module::Pcm, &pids).await
    }

    /// Reads fuel-trim PIDs for both banks (short and long term).
    pub async fn read_fuel_trim_data(&mut self) -> Result<Vec<PidValue>> {
        let pids = vec![
            Pid::SHORT_FUEL_TRIM_B1,
            Pid::LONG_FUEL_TRIM_B1,
            Pid::SHORT_FUEL_TRIM_B2,
            Pid::LONG_FUEL_TRIM_B2,
        ];
        self.read_live_data(Module::Pcm, &pids).await
    }

    /// Reads emissions-related PIDs: timing advance, O2 sensors, EGR commanded and error.
    pub async fn read_emissions_data(&mut self) -> Result<Vec<PidValue>> {
        let pids = vec![
            Pid::TIMING_ADVANCE,
            Pid::O2_B1S1_VOLTAGE,
            Pid::O2_B1S2_VOLTAGE,
            Pid::EGR_COMMANDED,
            Pid::EGR_ERROR,
        ];
        self.read_live_data(Module::Pcm, &pids).await
    }

    /// Reads powertrain PIDs: fuel rate, driver demand/actual torque, throttle, module voltage.
    pub async fn read_powertrain_data(&mut self) -> Result<Vec<PidValue>> {
        let pids = vec![
            Pid::ENGINE_FUEL_RATE,
            Pid::DRIVER_DEMAND_TORQUE,
            Pid::ACTUAL_ENGINE_TORQUE,
            Pid::REL_THROTTLE_POS,
            Pid::CONTROL_MODULE_VOLTAGE,
        ];
        self.read_live_data(Module::Pcm, &pids).await
    }
}

//! Live OBD-II data streaming command.

use crate::{cli::ModuleArg, output::OutputFormatter};
use anyhow::Result;
use faraday_core::{protocol::j1979::Pid, Module};
use std::time::Duration;

pub async fn execute(
    adapter_path: String,
    module: ModuleArg,
    pid_strings: Vec<String>,
    interval_ms: u64,
) -> Result<()> {
    let mut executor = super::create_executor(adapter_path).await?;
    let mut formatter = OutputFormatter::new(false);
    let module: Module = module.into();

    let pids: Result<Vec<Pid>, _> = pid_strings
        .iter()
        .map(|s| {
            u8::from_str_radix(s.trim_start_matches("0x"), 16)
                .map(Pid)
                .map_err(|_| anyhow::anyhow!("Invalid PID: {}", s))
        })
        .collect();

    let pids = pids?;

    formatter.print_info(&format!(
        "Reading live data from {:?}, PIDs: {:02X?}, interval: {}ms",
        module,
        pids.iter().map(|p| p.0).collect::<Vec<_>>(),
        interval_ms
    ))?;

    formatter.print_info("Press Ctrl+C to stop")?;

    loop {
        let values = executor.read_live_data(module, &pids).await?;

        print!("\x1B[2J\x1B[1;1H");
        formatter.print_live_data(&values)?;

        tokio::time::sleep(Duration::from_millis(interval_ms)).await;
    }
}

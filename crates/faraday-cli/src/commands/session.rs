use crate::{
    cli::{ModuleArg, SessionArg},
    output::OutputFormatter,
};
use anyhow::Result;
use faraday_core::{protocol::uds::DiagnosticSession, Module};

pub async fn execute(adapter_path: String, module: ModuleArg, session: SessionArg) -> Result<()> {
    let mut executor = super::create_executor(adapter_path).await?;
    let mut formatter = OutputFormatter::new(false);
    let module: Module = module.into();
    let session: DiagnosticSession = session.into();

    formatter.print_info(&format!(
        "Setting diagnostic session to {:?} for {:?}...",
        session, module
    ))?;

    executor
        .diagnostic_session_control(module.request_id(), module.response_id(), session)
        .await?;

    formatter.print_success(&format!("Diagnostic session set to {:?}", session))?;

    Ok(())
}

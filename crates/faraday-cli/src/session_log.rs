use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::audit::data_dir;

#[derive(Serialize)]
pub struct SessionEntry {
    pub timestamp: String,
    pub command: String,
    pub adapter: String,
    pub duration_ms: u64,
    pub result: String,
}

pub struct SessionLogger {
    path: PathBuf,
}

impl SessionLogger {
    pub fn new() -> Result<Self> {
        let dir = data_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create data directory: {}", dir.display()))?;
        Ok(Self {
            path: dir.join("sessions.jsonl"),
        })
    }

    pub fn log(&self, entry: &SessionEntry) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("failed to open session log: {}", self.path.display()))?;

        let line = serde_json::to_string(entry).context("failed to serialize session entry")?;
        writeln!(file, "{}", line).context("failed to write session entry")?;
        Ok(())
    }
}

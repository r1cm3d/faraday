//! Session-level diagnostic command logging.

use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::audit::data_dir;

/// One structured session record serialized to JSONL.
#[derive(Serialize)]
pub struct SessionEntry {
    /// ISO-8601 timestamp when the command ran.
    pub timestamp: String,
    /// The CLI sub-command that was executed (e.g. `"read-dtc"`).
    pub command: String,
    /// Adapter device path used for this session.
    pub adapter: String,
    /// Wall-clock duration of the command in milliseconds.
    pub duration_ms: u64,
    /// `"ok"` on success, or a short error description.
    pub result: String,
}

/// Appends session entries to `~/.local/share/faraday/sessions.jsonl`.
pub struct SessionLogger {
    path: PathBuf,
}

impl SessionLogger {
    /// Creates the data directory if necessary and returns a ready logger.
    pub fn new() -> Result<Self> {
        let dir = data_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create data directory: {}", dir.display()))?;
        Ok(Self {
            path: dir.join("sessions.jsonl"),
        })
    }

    /// Serializes `entry` as a JSON line and appends it to the session log.
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

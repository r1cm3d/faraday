//! Audit logging for as-built write operations.

use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

/// Which write operation triggered the audit record.
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditOperation {
    /// A direct feature write (`asbuilt write`).
    Write,
    /// A full module restore from snapshot (`asbuilt restore`).
    Restore,
}

/// One structured audit record serialized to JSONL.
#[derive(Serialize)]
pub struct AuditEntry {
    /// ISO-8601 timestamp of the operation.
    pub timestamp: String,
    /// The type of write operation performed.
    pub operation: AuditOperation,
    /// Short module name (e.g. `"BCM"`).
    pub module: String,
    /// UDS data identifier that was written.
    pub did: u16,
    /// Hex representation of the block data before the write.
    pub before_hex: String,
    /// Hex representation of the block data after the write.
    pub after_hex: String,
    /// `true` when the write was a dry-run and no data was actually sent.
    pub dry_run: bool,
    /// `"ok"` on success, or an error message.
    pub result: String,
}

/// Appends audit entries to `~/.local/share/faraday/audit.jsonl`.
pub struct AuditLogger {
    path: PathBuf,
}

impl AuditLogger {
    /// Creates the data directory if necessary and returns a ready logger.
    pub fn new() -> Result<Self> {
        let dir = data_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create data directory: {}", dir.display()))?;
        Ok(Self {
            path: dir.join("audit.jsonl"),
        })
    }

    /// Serializes `entry` as a JSON line and appends it to the audit file.
    pub fn log(&self, entry: &AuditEntry) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("failed to open audit log: {}", self.path.display()))?;

        let line = serde_json::to_string(entry).context("failed to serialize audit entry")?;
        writeln!(file, "{}", line).context("failed to write audit entry")?;
        Ok(())
    }
}

/// Returns the directory used for as-built snapshots.
pub fn snapshot_dir() -> PathBuf {
    data_dir().join("snapshots")
}

/// Returns the faraday data directory (`$XDG_DATA_HOME/faraday` or `~/.local/share/faraday`).
pub fn data_dir() -> PathBuf {
    std::env::var("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".local").join("share"))
        })
        .unwrap_or_else(|| PathBuf::from("."))
        .join("faraday")
}

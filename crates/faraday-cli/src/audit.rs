use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditOperation {
    Write,
    Restore,
}

#[derive(Serialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub operation: AuditOperation,
    pub module: String,
    pub did: u16,
    pub before_hex: String,
    pub after_hex: String,
    pub dry_run: bool,
    pub result: String,
}

pub struct AuditLogger {
    path: PathBuf,
}

impl AuditLogger {
    pub fn new() -> Result<Self> {
        let dir = data_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create data directory: {}", dir.display()))?;
        Ok(Self {
            path: dir.join("audit.jsonl"),
        })
    }

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

pub fn snapshot_dir() -> PathBuf {
    data_dir().join("snapshots")
}

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

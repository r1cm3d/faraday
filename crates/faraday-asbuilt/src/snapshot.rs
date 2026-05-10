use crate::{AsBuiltSnapshot, Error, Result};
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};

pub fn save_snapshot(path: &Path, snapshot: &AsBuiltSnapshot) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            Error::invalid_block(format!("failed to create snapshot directory: {}", e))
        })?;
    }

    let file = fs::File::create(path)
        .map_err(|e| Error::invalid_block(format!("failed to create snapshot file: {}", e)))?;

    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, snapshot)
        .map_err(|e| Error::invalid_block(format!("failed to serialize snapshot: {}", e)))?;
    writer
        .flush()
        .map_err(|e| Error::invalid_block(format!("failed to flush snapshot: {}", e)))?;

    Ok(())
}

pub fn load_snapshot(path: &Path) -> Result<AsBuiltSnapshot> {
    let file = fs::File::open(path)
        .map_err(|e| Error::invalid_block(format!("failed to open snapshot file: {}", e)))?;

    serde_json::from_reader(file)
        .map_err(|e| Error::invalid_block(format!("failed to deserialize snapshot: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AsBuiltBlock, BlockId};

    fn make_snapshot() -> AsBuiltSnapshot {
        AsBuiltSnapshot {
            timestamp: "2026-05-10T14:00:00Z".to_string(),
            vehicle_vin: Some("3FADP0L33HR123456".to_string()),
            blocks: vec![AsBuiltBlock {
                id: BlockId::new("726", "01"),
                description: "test block".to_string(),
                did: 0x0701,
                data: vec![0xAB, 0xCD],
                features: vec![],
            }],
        }
    }

    #[test]
    fn roundtrip_save_and_load() {
        let dir = std::env::temp_dir().join("faraday_test_snapshot");
        let path = dir.join("test.json");
        let snapshot = make_snapshot();

        save_snapshot(&path, &snapshot).unwrap();
        let loaded = load_snapshot(&path).unwrap();

        assert_eq!(loaded.timestamp, snapshot.timestamp);
        assert_eq!(loaded.vehicle_vin, snapshot.vehicle_vin);
        assert_eq!(loaded.blocks.len(), 1);
        assert_eq!(loaded.blocks[0].data, vec![0xAB, 0xCD]);

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(dir);
    }

    #[test]
    fn load_returns_error_for_missing_file() {
        let result = load_snapshot(Path::new("/nonexistent/path/snapshot.json"));
        assert!(result.is_err());
    }
}

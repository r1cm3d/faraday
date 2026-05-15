//! Human-readable and JSON output formatting for CLI commands.

use faraday_asbuilt::{AsBuiltBlock, FeatureValue};
use faraday_core::protocol::j1979::{Dtc, PidValue};
use serde::Serialize;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Renders CLI command output as coloured text or JSON, depending on mode.
pub struct OutputFormatter {
    stdout: StandardStream,
    json_mode: bool,
}

impl OutputFormatter {
    /// Creates a new formatter. Pass `json_mode = true` to emit machine-readable JSON.
    pub fn new(json_mode: bool) -> Self {
        Self {
            stdout: StandardStream::stdout(ColorChoice::Auto),
            json_mode,
        }
    }

    /// Prints a list of DTCs, or a success message when the list is empty.
    pub fn print_dtcs(&mut self, dtcs: &[Dtc]) -> io::Result<()> {
        if self.json_mode {
            println!("{}", serde_json::to_string_pretty(dtcs)?);
            return Ok(());
        }

        if dtcs.is_empty() {
            self.print_success("No diagnostic trouble codes found")?;
            return Ok(());
        }

        self.print_header(&format!("Found {} DTC(s)", dtcs.len()))?;

        for dtc in dtcs {
            self.print_error(&format!("  {} - {}", dtc.code, dtc.description))?;
        }

        Ok(())
    }

    /// Prints a table of live PID values with engineering units where available.
    pub fn print_live_data(&mut self, values: &[PidValue]) -> io::Result<()> {
        if self.json_mode {
            println!("{}", serde_json::to_string_pretty(values)?);
            return Ok(());
        }

        for value in values {
            match (&value.interpreted_value, &value.unit) {
                (Some(val), Some(unit)) => {
                    println!("PID {:02X}: {:.2} {}", value.pid.0, val, unit);
                }
                (Some(val), None) => {
                    println!("PID {:02X}: {:.2}", value.pid.0, val);
                }
                _ => {
                    println!(
                        "PID {:02X}: {} (raw)",
                        value.pid.0,
                        hex::encode(&value.raw_value)
                    );
                }
            }
        }

        Ok(())
    }

    /// Prints the vehicle VIN string.
    pub fn print_vin(&mut self, vin: &str) -> io::Result<()> {
        if self.json_mode {
            let json = serde_json::json!({ "vin": vin });
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            self.print_info(&format!("VIN: {}", vin))?;
        }
        Ok(())
    }

    /// Prints a UDS DID value as uppercase hex.
    pub fn print_did_data(&mut self, did: u16, data: &[u8]) -> io::Result<()> {
        if self.json_mode {
            let json = serde_json::json!({
                "did": format!("{:04X}", did),
                "data": hex::encode(data)
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("DID {:04X}: {}", did, hex::encode_upper(data));
        }
        Ok(())
    }

    /// Prints a green success message prefixed with `✓`.
    pub fn print_success(&mut self, message: &str) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        writeln!(self.stdout, "✓ {}", message)?;
        self.stdout.reset()?;
        Ok(())
    }

    /// Prints a blue informational message prefixed with `ℹ`.
    pub fn print_info(&mut self, message: &str) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        writeln!(self.stdout, "ℹ {}", message)?;
        self.stdout.reset()?;
        Ok(())
    }

    /// Prints a red error message prefixed with `✗`.
    pub fn print_error(&mut self, message: &str) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(self.stdout, "✗ {}", message)?;
        self.stdout.reset()?;
        Ok(())
    }

    /// Prints a bold cyan section header.
    pub fn print_header(&mut self, message: &str) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        writeln!(self.stdout, "{}", message)?;
        self.stdout.reset()?;
        Ok(())
    }

    /// Dumps all as-built blocks with their decoded feature values as YAML or JSON.
    pub fn print_asbuilt_dump(
        &mut self,
        blocks: &[(AsBuiltBlock, Vec<FeatureValue>)],
    ) -> io::Result<()> {
        #[derive(Serialize)]
        struct DumpOutput {
            blocks: Vec<BlockOutput>,
        }

        #[derive(Serialize)]
        struct BlockOutput {
            id: String,
            description: String,
            did: String,
            data: String,
            features: Vec<FeatureOutput>,
        }

        #[derive(Serialize)]
        struct FeatureOutput {
            name: String,
            description: String,
            value: String,
            raw: u8,
        }

        let output = DumpOutput {
            blocks: blocks
                .iter()
                .map(|(block, features)| BlockOutput {
                    id: block.id.to_string(),
                    description: block.description.clone(),
                    did: format!("0x{:04X}", block.did),
                    data: hex::encode_upper(&block.data),
                    features: features
                        .iter()
                        .map(|fv| FeatureOutput {
                            name: fv.feature.name.clone(),
                            description: fv.feature.description.clone(),
                            value: fv.interpreted_value.clone(),
                            raw: fv.raw_value,
                        })
                        .collect(),
                })
                .collect(),
        };

        let yaml = serde_yaml::to_string(&output).map_err(io::Error::other)?;
        print!("{}", yaml);
        Ok(())
    }

    /// Prints a single decoded as-built feature value.
    pub fn print_asbuilt_feature(&mut self, fv: &FeatureValue) -> io::Result<()> {
        println!(
            "{}: {} (raw: {})",
            fv.feature.name, fv.interpreted_value, fv.raw_value
        );
        Ok(())
    }
}

use faraday_core::protocol::j1979::{Dtc, PidValue};
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct OutputFormatter {
    stdout: StandardStream,
    json_mode: bool,
}

impl OutputFormatter {
    pub fn new(json_mode: bool) -> Self {
        Self {
            stdout: StandardStream::stdout(ColorChoice::Auto),
            json_mode,
        }
    }

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
                    println!("PID {:02X}: {} (raw)", value.pid.0, hex::encode(&value.raw_value));
                }
            }
        }

        Ok(())
    }

    pub fn print_vin(&mut self, vin: &str) -> io::Result<()> {
        if self.json_mode {
            let json = serde_json::json!({ "vin": vin });
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            self.print_info(&format!("VIN: {}", vin))?;
        }
        Ok(())
    }

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

    pub fn print_success(&mut self, message: &str) -> io::Result<()> {
        self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        writeln!(self.stdout, "✓ {}", message)?;
        self.stdout.reset()?;
        Ok(())
    }

    pub fn print_info(&mut self, message: &str) -> io::Result<()> {
        self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        writeln!(self.stdout, "ℹ {}", message)?;
        self.stdout.reset()?;
        Ok(())
    }


    pub fn print_error(&mut self, message: &str) -> io::Result<()> {
        self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(self.stdout, "✗ {}", message)?;
        self.stdout.reset()?;
        Ok(())
    }

    pub fn print_header(&mut self, message: &str) -> io::Result<()> {
        self.stdout.set_color(
            ColorSpec::new()
                .set_fg(Some(Color::Cyan))
                .set_bold(true)
        )?;
        writeln!(self.stdout, "{}", message)?;
        self.stdout.reset()?;
        Ok(())
    }
}
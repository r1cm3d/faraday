/*!
vLinker FS adapter implementation.

The vLinker FS uses proprietary STN1170/STN2120 commands rather than standard
ELM327 AT commands for Ford-specific functionality including automatic
MS-CAN/HS-CAN switching.
*/

use crate::{CanBus, CanFrame, CanId, Error, Result};
use async_trait::async_trait;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_serial::SerialStream;
use tracing::{debug, trace};

pub struct VLinkerFs {
    port: Option<SerialStream>,
    current_bus: Option<CanBus>,
    connected: bool,
}

impl VLinkerFs {
    pub fn new() -> Self {
        Self {
            port: None,
            current_bus: None,
            connected: false,
        }
    }

    pub fn with_port_name(port_name: &str) -> Result<Self> {
        let builder = tokio_serial::new(port_name, 38400)
            .timeout(Duration::from_millis(1000))
            .data_bits(tokio_serial::DataBits::Eight)
            .stop_bits(tokio_serial::StopBits::One)
            .parity(tokio_serial::Parity::None)
            .flow_control(tokio_serial::FlowControl::None);

        let port = SerialStream::open(&builder)?;

        Ok(Self {
            port: Some(port),
            current_bus: None,
            connected: false,
        })
    }

    async fn send_command(&mut self, command: &str) -> Result<String> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| Error::link("Port not initialized"))?;

        trace!("Sending command: {}", command);

        let mut cmd = command.to_string();
        cmd.push('\r');
        port.write_all(cmd.as_bytes()).await?;

        let mut reader = BufReader::new(port);
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        let response = response.trim();
        trace!("Received response: {}", response);

        if response.contains("ERROR") {
            return Err(Error::link(format!("Command failed: {}", response)));
        }

        Ok(response.to_string())
    }

    async fn initialize(&mut self) -> Result<()> {
        self.send_command("STI").await?;

        let version = self.send_command("STI").await?;
        debug!("vLinker FS version: {}", version);

        self.send_command("STSLBRP").await?;

        self.send_command("STFAP").await?;

        Ok(())
    }
}

impl Default for VLinkerFs {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::LinkLayer for VLinkerFs {
    async fn connect(&mut self) -> Result<()> {
        if self.port.is_none() {
            return Err(Error::link("Port not configured"));
        }

        self.initialize().await?;
        self.connected = true;

        debug!("vLinker FS connected successfully");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        self.port = None;
        debug!("vLinker FS disconnected");
        Ok(())
    }

    async fn send_frame(&mut self, frame: &CanFrame) -> Result<()> {
        let data_hex = hex::encode_upper(&frame.data);
        let command = format!("STPX H:{:03X},0,{}", frame.id.id(), data_hex);

        trace!("Sending CAN frame: ID={:03X}, Data={}", frame.id.id(), data_hex);

        self.send_command(&command).await?;
        Ok(())
    }

    async fn receive_frame(&mut self) -> Result<CanFrame> {
        loop {
            let response = self.send_command("STRX").await?;

            if response.trim().is_empty() {
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }

            if let Some(frame) = self.parse_can_frame(&response)? {
                trace!("Received CAN frame: ID={:03X}, Data={}", frame.id.id(), hex::encode_upper(&frame.data));
                return Ok(frame);
            }
        }
    }

    async fn set_can_bus(&mut self, bus: CanBus) -> Result<()> {
        if self.current_bus == Some(bus) {
            return Ok(());
        }

        let command = match bus {
            CanBus::HsCan => "STCP 24",  // HS-CAN protocol
            CanBus::MsCan => "STCP 25",  // MS-CAN protocol
        };

        self.send_command(command).await?;
        self.current_bus = Some(bus);

        debug!("Switched to {:?}", bus);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl VLinkerFs {
    fn parse_can_frame(&self, response: &str) -> Result<Option<CanFrame>> {
        if response.trim().is_empty() || response.contains("NO DATA") {
            return Ok(None);
        }

        let parts: Vec<&str> = response.trim().split(',').collect();
        if parts.len() < 2 {
            return Err(Error::invalid_frame("Invalid frame format"));
        }

        let id_str = parts[0].trim();
        let data_str = parts[1].trim();

        let id = u32::from_str_radix(id_str, 16)
            .map_err(|_| Error::invalid_frame("Invalid CAN ID"))?;

        let data = hex::decode(data_str)
            .map_err(|_| Error::invalid_frame("Invalid hex data"))?;

        Ok(Some(CanFrame::new(CanId::new(id), data)))
    }
}
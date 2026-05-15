use serde::{Deserialize, Serialize};

/// A CAN message identifier (11-bit standard or 29-bit extended).
///
/// The high bit of the raw `u32` value distinguishes extended frames
/// (`0x80000000` set) from standard frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanId(pub u32);

impl CanId {
    /// Constructs a `CanId` from a raw integer value.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns `true` if the frame uses a 29-bit extended identifier.
    pub fn is_extended(&self) -> bool {
        self.0 & 0x80000000 != 0
    }

    /// Returns the effective identifier value (11 or 29 bits, sign-bit stripped).
    pub fn id(&self) -> u32 {
        if self.is_extended() {
            self.0 & 0x1FFFFFFF
        } else {
            self.0 & 0x7FF
        }
    }
}

impl From<u32> for CanId {
    fn from(id: u32) -> Self {
        Self::new(id)
    }
}

/// A raw CAN frame consisting of an identifier and up to 8 bytes of payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanFrame {
    /// The frame identifier.
    pub id: CanId,
    /// The raw payload bytes (0–8 bytes for classical CAN).
    pub data: Vec<u8>,
}

impl CanFrame {
    /// Creates a new frame with the given identifier and data.
    pub fn new(id: CanId, data: Vec<u8>) -> Self {
        Self { id, data }
    }

    /// Returns the number of data bytes in the frame.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the frame carries no payload bytes.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Which physical CAN bus a module lives on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanBus {
    /// High-Speed CAN — 500 kbps, pins 6/14 (PCM, TCM, ABS, RCM, PSCM).
    HsCan,
    /// Medium-Speed CAN — 125 kbps, pins 3/11 (BCM, IPC, APIM, HVAC, DSM, PAM).
    MsCan,
}

impl CanBus {
    /// Returns the nominal bit rate in bits per second.
    pub fn speed(&self) -> u32 {
        match self {
            CanBus::HsCan => 500_000,
            CanBus::MsCan => 125_000,
        }
    }
}

/// Ford Fusion 2017 SEL ECU module identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Module {
    /// Powertrain Control Module (HS-CAN).
    Pcm,
    /// Transmission Control Module (HS-CAN).
    Tcm,
    /// Anti-lock Braking System module (HS-CAN).
    Abs,
    /// Restraint Control Module / airbag controller (HS-CAN).
    Rcm,
    /// Power Steering Control Module (HS-CAN).
    Pscm,
    /// Body Control Module (MS-CAN).
    Bcm,
    /// Instrument Panel Cluster (MS-CAN).
    Ipc,
    /// Accessory Protocol Interface Module / SYNC (MS-CAN).
    Apim,
    /// Parking Aid Module / ultrasonic sensors (MS-CAN).
    Pam,
    /// Driver Seat Module (MS-CAN).
    Dsm,
    /// HVAC control module (MS-CAN).
    Hvac,
}

impl Module {
    /// Returns the CAN bus this module communicates on.
    pub fn bus(&self) -> CanBus {
        match self {
            Module::Pcm | Module::Tcm | Module::Abs | Module::Rcm | Module::Pscm => CanBus::HsCan,
            Module::Bcm | Module::Ipc | Module::Apim | Module::Pam | Module::Dsm | Module::Hvac => {
                CanBus::MsCan
            }
        }
    }

    /// Returns the standard UDS request CAN ID for this module.
    pub fn request_id(&self) -> CanId {
        match self {
            Module::Pcm => CanId::new(0x7E0),
            Module::Tcm => CanId::new(0x7E1),
            Module::Abs => CanId::new(0x7E2),
            Module::Rcm => CanId::new(0x7E3),
            Module::Pscm => CanId::new(0x7E4),
            Module::Bcm => CanId::new(0x726),
            Module::Ipc => CanId::new(0x720),
            Module::Apim => CanId::new(0x7D0),
            Module::Pam => CanId::new(0x733),
            Module::Dsm => CanId::new(0x727),
            Module::Hvac => CanId::new(0x703),
        }
    }

    /// Returns the UDS response CAN ID for this module.
    pub fn response_id(&self) -> CanId {
        match self {
            Module::Pcm => CanId::new(0x7E8),
            Module::Tcm => CanId::new(0x7E9),
            Module::Abs => CanId::new(0x7EA),
            Module::Rcm => CanId::new(0x7EB),
            Module::Pscm => CanId::new(0x7EC),
            Module::Bcm => CanId::new(0x72E),
            Module::Ipc => CanId::new(0x728),
            Module::Apim => CanId::new(0x7D8),
            Module::Pam => CanId::new(0x73B),
            Module::Dsm => CanId::new(0x72F),
            Module::Hvac => CanId::new(0x70B),
        }
    }

    /// Returns the short display name of this module (e.g. `"PCM"`, `"BCM"`).
    pub fn name(&self) -> &'static str {
        match self {
            Module::Pcm => "PCM",
            Module::Tcm => "TCM",
            Module::Abs => "ABS",
            Module::Rcm => "RCM",
            Module::Pscm => "PSCM",
            Module::Bcm => "BCM",
            Module::Ipc => "IPC",
            Module::Apim => "APIM",
            Module::Pam => "PAM",
            Module::Dsm => "DSM",
            Module::Hvac => "HVAC",
        }
    }

    /// Returns the canonical ordered slice of all supported modules.
    pub fn all() -> &'static [Module] {
        &[
            Module::Pcm,
            Module::Tcm,
            Module::Abs,
            Module::Rcm,
            Module::Bcm,
            Module::Ipc,
            Module::Apim,
            Module::Pam,
            Module::Hvac,
        ]
    }
}

/// Which DTC memory bank to query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtcKind {
    /// Confirmed fault codes stored in non-volatile memory (Mode 03).
    Stored,
    /// Fault codes that have not yet confirmed (Mode 07).
    Pending,
    /// Fault codes that cannot be cleared by Mode 04 (Mode 0A).
    Permanent,
}

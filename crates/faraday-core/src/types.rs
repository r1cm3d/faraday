use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanId(pub u32);

impl CanId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn is_extended(&self) -> bool {
        self.0 & 0x80000000 != 0
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanFrame {
    pub id: CanId,
    pub data: Vec<u8>,
}

impl CanFrame {
    pub fn new(id: CanId, data: Vec<u8>) -> Self {
        Self { id, data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanBus {
    HsCan,
    MsCan,
}

impl CanBus {
    pub fn speed(&self) -> u32 {
        match self {
            CanBus::HsCan => 500_000,
            CanBus::MsCan => 125_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Module {
    Pcm,
    Tcm,
    Abs,
    Rcm,
    Pscm,
    Bcm,
    Ipc,
    Apim,
    Pam,
    Dsm,
    Hvac,
}

impl Module {
    pub fn bus(&self) -> CanBus {
        match self {
            Module::Pcm | Module::Tcm | Module::Abs | Module::Rcm | Module::Pscm => CanBus::HsCan,
            Module::Bcm | Module::Ipc | Module::Apim | Module::Pam | Module::Dsm | Module::Hvac => {
                CanBus::MsCan
            }
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtcKind {
    Stored,
    Pending,
    Permanent,
}

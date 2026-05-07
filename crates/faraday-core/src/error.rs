
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serial port error: {0}")]
    Serial(#[from] serialport::Error),

    #[error("Protocol error: {message}")]
    Protocol { message: String },

    #[error("Transport error: {message}")]
    Transport { message: String },

    #[error("Link layer error: {message}")]
    Link { message: String },

    #[error("Invalid frame: {reason}")]
    InvalidFrame { reason: String },

    #[error("UDS negative response: service {service:#04X}, code {code:#04X} ({description})")]
    UdsNegativeResponse {
        service: u8,
        code: u8,
        description: String,
    },

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Connection lost")]
    ConnectionLost,

    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    #[error("Unsupported operation: {operation}")]
    Unsupported { operation: String },
}

impl Error {
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    pub fn link(message: impl Into<String>) -> Self {
        Self::Link {
            message: message.into(),
        }
    }

    pub fn invalid_frame(reason: impl Into<String>) -> Self {
        Self::InvalidFrame {
            reason: reason.into(),
        }
    }

    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData {
            message: message.into(),
        }
    }

    pub fn unsupported(operation: impl Into<String>) -> Self {
        Self::Unsupported {
            operation: operation.into(),
        }
    }
}
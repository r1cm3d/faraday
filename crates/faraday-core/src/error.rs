/// Convenience alias so callers write `Result<T>` instead of `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// All errors that can be produced by the faraday-core crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wraps a standard I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Wraps a serial-port open or configuration failure.
    #[error("Serial port error: {0}")]
    Serial(#[from] serialport::Error),

    /// A violation of the OBD-II or UDS protocol framing rules.
    #[error("Protocol error: {message}")]
    Protocol { message: String },

    /// An ISO-TP segmentation or reassembly failure.
    #[error("Transport error: {message}")]
    Transport { message: String },

    /// A link-layer (adapter) communication failure.
    #[error("Link layer error: {message}")]
    Link { message: String },

    /// A CAN frame whose structure does not match expectations.
    #[error("Invalid frame: {reason}")]
    InvalidFrame { reason: String },

    /// The ECU returned a UDS negative response (NRC).
    #[error("UDS negative response: service {service:#04X}, code {code:#04X} ({description})")]
    UdsNegativeResponse {
        service: u8,
        code: u8,
        description: String,
    },

    /// No response was received within the configured timeout.
    #[error("Timeout waiting for response")]
    Timeout,

    /// The adapter disconnected unexpectedly.
    #[error("Connection lost")]
    ConnectionLost,

    /// The received payload cannot be interpreted as the expected type.
    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    /// The requested operation is not supported by this adapter or module.
    #[error("Unsupported operation: {operation}")]
    Unsupported { operation: String },
}

impl Error {
    /// Creates a [`Error::Protocol`] variant with the given message.
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    /// Creates a [`Error::Transport`] variant with the given message.
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    /// Creates a [`Error::Link`] variant with the given message.
    pub fn link(message: impl Into<String>) -> Self {
        Self::Link {
            message: message.into(),
        }
    }

    /// Creates a [`Error::InvalidFrame`] variant with the given reason.
    pub fn invalid_frame(reason: impl Into<String>) -> Self {
        Self::InvalidFrame {
            reason: reason.into(),
        }
    }

    /// Creates a [`Error::InvalidData`] variant with the given message.
    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData {
            message: message.into(),
        }
    }

    /// Creates a [`Error::Unsupported`] variant describing the operation.
    pub fn unsupported(operation: impl Into<String>) -> Self {
        Self::Unsupported {
            operation: operation.into(),
        }
    }
}

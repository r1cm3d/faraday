//! Error types for the faraday-asbuilt crate.

/// Convenience alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when decoding, encoding, or persisting as-built data.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The block data is structurally invalid.
    #[error("Invalid block data: {message}")]
    InvalidBlock { message: String },

    /// The requested block is not in the catalog.
    #[error("Unknown block: {block_id}")]
    UnknownBlock { block_id: String },

    /// A bit position references a bit index outside 0–7.
    #[error("Invalid bit position: byte {byte}, bit {bit}")]
    InvalidBitPosition { byte: usize, bit: usize },

    /// The data slice is shorter than required by the bit position.
    #[error("Data too short: expected at least {expected} bytes, got {actual}")]
    DataTooShort { expected: usize, actual: usize },
}

impl Error {
    /// Constructs an [`Error::InvalidBlock`] with the given message.
    pub fn invalid_block(message: impl Into<String>) -> Self {
        Self::InvalidBlock {
            message: message.into(),
        }
    }

    /// Constructs an [`Error::UnknownBlock`] with the given block identifier.
    pub fn unknown_block(block_id: impl Into<String>) -> Self {
        Self::UnknownBlock {
            block_id: block_id.into(),
        }
    }

    /// Constructs an [`Error::InvalidBitPosition`] for the given byte and bit index.
    pub fn invalid_bit_position(byte: usize, bit: usize) -> Self {
        Self::InvalidBitPosition { byte, bit }
    }

    /// Constructs an [`Error::DataTooShort`] with the expected and actual byte counts.
    pub fn data_too_short(expected: usize, actual: usize) -> Self {
        Self::DataTooShort { expected, actual }
    }
}

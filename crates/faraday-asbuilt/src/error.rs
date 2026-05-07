pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid block data: {message}")]
    InvalidBlock { message: String },

    #[error("Unknown block: {block_id}")]
    UnknownBlock { block_id: String },

    #[error("Invalid bit position: byte {byte}, bit {bit}")]
    InvalidBitPosition { byte: usize, bit: usize },

    #[error("Data too short: expected at least {expected} bytes, got {actual}")]
    DataTooShort { expected: usize, actual: usize },
}

impl Error {
    pub fn invalid_block(message: impl Into<String>) -> Self {
        Self::InvalidBlock {
            message: message.into(),
        }
    }

    pub fn unknown_block(block_id: impl Into<String>) -> Self {
        Self::UnknownBlock {
            block_id: block_id.into(),
        }
    }

    pub fn invalid_bit_position(byte: usize, bit: usize) -> Self {
        Self::InvalidBitPosition { byte, bit }
    }

    pub fn data_too_short(expected: usize, actual: usize) -> Self {
        Self::DataTooShort { expected, actual }
    }
}
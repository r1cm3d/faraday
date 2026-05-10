/*!
As-built configuration data and decoders for Ford Fusion 2017 SEL.

This crate contains the known configuration blocks and bit mappings
for various Ford modules in the 2017 Fusion SEL. It provides semantic
interpretation of raw as-built data.

# Safety

This crate is data-only and contains no write functionality. It provides
read-only access to configuration interpretations.
*/

pub mod bcm;
pub mod ipc;
pub mod pcm;

pub mod decoder;
pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

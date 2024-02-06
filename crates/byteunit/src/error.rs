use core::num::ParseFloatError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseByteUnitError {
    #[error("Failed to parse float from str")]
    ParseFloat(#[from] ParseFloatError),

    #[error("Empty Float String")]
    EmptyFloatStr,

    #[error("Empty Unit String")]
    EmptyUnitStr,
    // #[error(transparent)]
    // Other(#[from] std::io::Error),
}

use serde_json::error::Error as SerdeJsonError;
use std::net::AddrParseError;
use std::str::Utf8Error;
use std::{error::Error, fmt, io::Error as IoError};

#[derive(Debug)]
pub enum NodeError {
    // InvalidSocketAddressError(AddrParseError),
    IoError(IoError),
    Utf8ConversionError(Utf8Error),
    InvalidRequest,
    JsonSerializationError(SerdeJsonError),
    NodeRegistrationError,
    InvalidSocketAddressError(AddrParseError),
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeError::IoError(e) => write!(f, "IO error: {}", e),
            NodeError::InvalidRequest => write!(f, "Bad request"),
            NodeError::Utf8ConversionError(e) => {
                write!(f, "Failed to parse buffer into utf8: {}", e)
            }
            NodeError::JsonSerializationError(e) => {
                write!(f, "Failed to serialize str: {}", e)
            }
            NodeError::NodeRegistrationError => write!(f, "Bad request"),
            NodeError::InvalidSocketAddressError(e) => write!(f, "Failed to parse address: {}", e),
        }
    }
}

impl From<IoError> for NodeError {
    fn from(err: IoError) -> NodeError {
        NodeError::IoError(err)
    }
}

impl From<Utf8Error> for NodeError {
    fn from(err: Utf8Error) -> NodeError {
        NodeError::Utf8ConversionError(err)
    }
}

impl From<SerdeJsonError> for NodeError {
    fn from(err: SerdeJsonError) -> NodeError {
        NodeError::JsonSerializationError(err)
    }
}

impl From<AddrParseError> for NodeError {
    fn from(err: AddrParseError) -> NodeError {
        NodeError::InvalidSocketAddressError(err)
    }
}

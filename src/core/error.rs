use libp2p_gossipsub::{ConfigBuilderError, SubscriptionError};
use serde_json::error::Error as SerdeJsonError;
use std::net::AddrParseError;
use std::str::Utf8Error;
use std::{error::Error, fmt, io::Error as IoError};

#[derive(Debug)]
pub enum NodeError {
    IoError(IoError),
    Utf8ConversionError(Utf8Error),
    InvalidRequest,
    JsonSerializationError(SerdeJsonError),
    NodeRegistrationError,
    InvalidSocketAddressError(AddrParseError),
    SwarmFailure,
    GossipConfigError(ConfigBuilderError),
    GossipSubError(SubscriptionError),
    NodeCreationError,
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeError::IoError(e) => write!(f, "IO error: {}", e),
            NodeError::InvalidRequest => write!(f, "Bad request"),
            NodeError::Utf8ConversionError(e) => {
                write!(f, "Failed to parse buffer into utf8: {}", e)
            }
            NodeError::NodeCreationError => write!(f, "Bad request"),
            NodeError::GossipConfigError(e) => {
                write!(f, "Failed to parse buffer into utf8: {}", e)
            }
            NodeError::JsonSerializationError(e) => {
                write!(f, "Failed to serialize str: {}", e)
            }
            NodeError::GossipSubError(e) => {
                write!(f, "Failed to serialize str: {}", e)
            }
            NodeError::NodeRegistrationError => write!(f, "Failed to register node"),
            NodeError::SwarmFailure => write!(f, "Swarm failed"),
            NodeError::InvalidSocketAddressError(e) => write!(f, "Failed to parse address: {}", e),
        }
    }
}

impl Error for NodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NodeError::InvalidSocketAddressError(e) => Some(e),
            NodeError::GossipSubError(e) => Some(e),
            NodeError::IoError(e) => Some(e),
            NodeError::JsonSerializationError(e) => Some(e),
            NodeError::GossipConfigError(e) => Some(e),
            NodeError::Utf8ConversionError(e) => Some(e),
            NodeError::NodeRegistrationError => None,
            NodeError::InvalidRequest => None,
            NodeError::NodeCreationError => None,
            NodeError::SwarmFailure => None,
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

impl From<SubscriptionError> for NodeError {
    fn from(err: SubscriptionError) -> NodeError {
        NodeError::GossipSubError(err)
    }
}

impl From<AddrParseError> for NodeError {
    fn from(err: AddrParseError) -> NodeError {
        NodeError::InvalidSocketAddressError(err)
    }
}

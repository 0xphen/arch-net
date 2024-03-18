use libp2p_gossipsub::{ConfigBuilderError, SubscriptionError};
use serde_json::error::Error as SerdeJsonError;
use std::net::AddrParseError;
use std::str::Utf8Error;
use std::{error::Error, fmt, io::Error as IoError};

#[derive(Debug)]
pub enum ArchError {
    IoError(IoError),
    Utf8ConversionError(Utf8Error),
    InvalidRequest,
    JsonSerializationError(SerdeJsonError),
    NodeRegistrationError,
    InvalidSocketAddressError(AddrParseError),
    SwarmFailure,
    GossipSubError(SubscriptionError),
    NodeCreationError,

    GossipConfigError(ConfigBuilderError),
    GossipBehaviourError,
    SwarmBuilderError,
}

impl fmt::Display for ArchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchError::GossipConfigError(e) => {
                write!(f, "Failed to parse buffer into utf8: {}", e)
            }
            ArchError::GossipBehaviourError => write!(f, "Swarm failed"),

            ArchError::SwarmBuilderError => write!(f, "Bad request"),

            ArchError::IoError(e) => write!(f, "IO error: {}", e),
            ArchError::InvalidRequest => write!(f, "Bad request"),
            ArchError::Utf8ConversionError(e) => {
                write!(f, "Failed to parse buffer into utf8: {}", e)
            }
            ArchError::NodeCreationError => write!(f, "Bad request"),

            ArchError::JsonSerializationError(e) => {
                write!(f, "Failed to serialize str: {}", e)
            }
            ArchError::GossipSubError(e) => {
                write!(f, "Failed to serialize str: {}", e)
            }
            ArchError::NodeRegistrationError => write!(f, "Failed to register node"),
            ArchError::SwarmFailure => write!(f, "Swarm failed"),
            ArchError::InvalidSocketAddressError(e) => write!(f, "Failed to parse address: {}", e),
        }
    }
}

impl Error for ArchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ArchError::InvalidSocketAddressError(e) => Some(e),
            ArchError::GossipSubError(e) => Some(e),
            ArchError::IoError(e) => Some(e),
            ArchError::JsonSerializationError(e) => Some(e),

            ArchError::GossipConfigError(e) => Some(e),
            ArchError::GossipBehaviourError => None,

            ArchError::Utf8ConversionError(e) => Some(e),
            ArchError::NodeRegistrationError => None,
            ArchError::InvalidRequest => None,
            ArchError::NodeCreationError => None,
            ArchError::SwarmFailure => None,
            ArchError::SwarmBuilderError => None,
        }
    }
}

impl From<IoError> for ArchError {
    fn from(err: IoError) -> ArchError {
        ArchError::IoError(err)
    }
}

impl From<Utf8Error> for ArchError {
    fn from(err: Utf8Error) -> ArchError {
        ArchError::Utf8ConversionError(err)
    }
}

impl From<SerdeJsonError> for ArchError {
    fn from(err: SerdeJsonError) -> ArchError {
        ArchError::JsonSerializationError(err)
    }
}

impl From<SubscriptionError> for ArchError {
    fn from(err: SubscriptionError) -> ArchError {
        ArchError::GossipSubError(err)
    }
}

impl From<AddrParseError> for ArchError {
    fn from(err: AddrParseError) -> ArchError {
        ArchError::InvalidSocketAddressError(err)
    }
}

impl From<ConfigBuilderError> for ArchError {
    fn from(err: ConfigBuilderError) -> ArchError {
        ArchError::GossipConfigError(err)
    }
}

impl From<&str> for ArchError {
    fn from(_err: &str) -> ArchError {
        ArchError::GossipBehaviourError
    }
}

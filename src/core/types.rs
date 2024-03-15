use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub addr: std::net::SocketAddr,
}

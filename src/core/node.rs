use crate::core::bootstrap_node;

use super::{config, error::NodeError, types::NodeInfo};
use log::{debug, error, info};
use serde_json;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Node {
    pub node_info: NodeInfo,
    pub peer_list: HashMap<String, SocketAddr>,
}

impl Node {
    pub fn new(addr: SocketAddr, id: String) -> Self {
        Self {
            node_info: NodeInfo { id, addr },
            peer_list: HashMap::new(),
        }
    }

    pub async fn join_network(&mut self, bootstrap_addr: SocketAddr) -> Result<(), NodeError> {
        let mut stream = TcpStream::connect(&bootstrap_addr).await?;

        info!(
            "Successfully connected to bootstrap node at address {:?}",
            bootstrap_addr
        );

        let node_info_str = serde_json::to_string(&self.node_info)?;
        stream.write_all(node_info_str.as_bytes()).await?;

        // Read the response from the bootstrap node
        let mut response_buf = String::new();
        stream.read_to_string(&mut response_buf).await?;

        let nodes: Vec<NodeInfo> = serde_json::from_str(&response_buf)?;

        debug!("Received response from bootstrap node {:?}", nodes);
        self.update_peerlist(nodes);

        Ok(())
    }

    fn update_peerlist(&mut self, nodes: Vec<NodeInfo>) {
        nodes.into_iter().for_each(
            |node| match self.peer_list.insert(node.id.clone(), node.addr) {
                Some(v) => debug!("Node #{} already exists", node.id),
                None => debug!("New node #{} added to peer list", node.id),
            },
        )
    }
}

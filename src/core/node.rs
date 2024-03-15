use super::{
    common::address_to_multiaddr, config::GOSSIP_TOPIC, error::NodeError,
    node_registry::NodeRegistry, peer_router::PeerRouter, types::NodeInfo,
};
use libp2p::Multiaddr;
use libp2p::{identity::Keypair, PeerId};
use log::{debug, error, info};
use serde_json;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Node {
    pub node_info: NodeInfo,
    pub peer_list: NodeRegistry,
    pub router: PeerRouter,
}

impl Node {
    pub fn new(addr: SocketAddr) -> Result<Self, NodeError> {
        let key_pair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(&key_pair.public());

        let router: PeerRouter =
            PeerRouter::new(&peer_id, &key_pair).map_err(|_| NodeError::NodeCreationError)?;

        Ok(Self {
            node_info: NodeInfo {
                id: peer_id.to_string(),
                addr,
            },
            peer_list: NodeRegistry::new(),
            router,
        })
    }

    pub async fn run(&mut self, bootstrap_addr: SocketAddr) -> Result<(), NodeError> {
        info!(
            "Local node #{}, Connecting to bootstrap node at IP {:?}",
            self.node_info.id, bootstrap_addr
        );

        self.bootstrap_node(bootstrap_addr).await?;

        info!("Successfull connected to boot node. Joining network.");

        let peers = self.peer_list.get_registered_nodes_subset();
        let peers_multi_addresses: Vec<Multiaddr> = peers
            .into_iter()
            .filter_map(|peer| address_to_multiaddr(peer.addr))
            .collect();

        self.router
            .run_swarm(GOSSIP_TOPIC, &peers_multi_addresses, &self.node_info)
            .await
            .map_err(|err| {
                error!("Swarm failed wih error {}", err);
                NodeError::SwarmFailure
            })?;

        Ok(())
    }

    async fn bootstrap_node(&mut self, bootstrap_addr: SocketAddr) -> Result<(), NodeError> {
        let mut stream = TcpStream::connect(&bootstrap_addr).await?;

        info!(
            "Successfully connected to boot node at address {:?}",
            bootstrap_addr
        );

        let node_info_str = serde_json::to_string(&self.node_info)?;
        stream.write_all(node_info_str.as_bytes()).await?;

        // Read the response from the boot node
        let mut response_buf = String::new();
        stream.read_to_string(&mut response_buf).await?;

        let nodes: Vec<NodeInfo> = serde_json::from_str(&response_buf)?;

        debug!("Received response from bootstrap node {:?}", nodes);
        self.update_peerlist(nodes);

        Ok(())
    }

    fn update_peerlist(&mut self, nodes: Vec<NodeInfo>) {
        nodes
            .into_iter()
            .for_each(|node_info| match self.peer_list.add_node(&node_info) {
                Some(_v) => debug!("Node #{} already exists", node_info.id),
                None => debug!("New node #{} added to peer list", node_info.id),
            })
    }
}

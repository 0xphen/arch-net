use super::{error::NodeError, node_registry::NodeRegistry, types::NodeInfo};
use log::{debug, error, info};
use serde_json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct BootstrapNode {
    pub registry: NodeRegistry,
    pub addr: SocketAddr,
}

impl BootstrapNode {
    fn new(socket_addr: SocketAddr) -> Self {
        BootstrapNode {
            registry: NodeRegistry::new(),
            addr: socket_addr,
        }
    }

    pub async fn run(&mut self) -> Result<(), NodeError> {
        let listener = TcpListener::bind(&self.addr).await?;
        info!("Proxy server listening on address {}", &self.addr);

        let shared_self = Arc::new(Mutex::new(self.clone()));

        loop {
            let (client_stream, _) = listener.accept().await?;

            let shared_self = Arc::clone(&shared_self);

            tokio::spawn(async move {
                let mut node_guard = shared_self.lock().await;

                if let Err(err) = node_guard
                    .handle_node_registration_request(client_stream)
                    .await
                {
                    error!("Error handling connection: {:?}", err);
                }
            });
        }
    }

    /// Handles incoming TCP connections representing requests for registering a node in the network.
    ///
    /// This function reads the incoming data from the TCP stream, parses it as a `NodeInfo` struct,
    /// and registers the node if the received information is valid. Upon successful registration,
    /// it gathers a subset list of registered nodes and sends it back to the client node.
    ///
    async fn handle_node_registration_request(
        &mut self,
        mut stream: TcpStream,
    ) -> Result<(), NodeError> {
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await?;
        let request_str = std::str::from_utf8(&buf[..n])?;
        let node_info: NodeInfo = serde_json::from_str(request_str)?;

        // Check if the received node info is valid
        if node_info.id.is_empty() || node_info.addr.ip().is_unspecified() {
            return Err(NodeError::InvalidRequest);
        }

        // If the request is valid, register the node
        match self.register_node(&node_info) {
            true => {
                // Gather a subset list of registered nodes
                let registered_nodes_subset = self.registry.get_registered_nodes_subset();

                // Serialize the subset list of nodes into JSON
                let registered_nodes_json = serde_json::to_string(&registered_nodes_subset)?;

                // Send the JSON string to the client node
                stream.write_all(registered_nodes_json.as_bytes()).await?;

                Ok(())
            }
            false => {
                error!("Failed to register node #{}", node_info.id);
                Err(NodeError::NodeRegistrationError)
            }
        }
    }

    fn register_node(&mut self, node_info: &NodeInfo) -> bool {
        self.registry.add_address(node_info.addr)
    }
}

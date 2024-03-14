use rand::prelude::*;
use std::collections::HashSet;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct NodeRegistry {
    pub node_addresses: HashSet<SocketAddr>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        NodeRegistry {
            node_addresses: HashSet::new(),
        }
    }

    pub fn add_address(&mut self, address: SocketAddr) -> bool {
        self.node_addresses.insert(address)
    }

    pub fn remove_address(&mut self, address: &SocketAddr) -> bool {
        self.node_addresses.remove(address)
    }

    pub fn get_addresses(&self) -> &HashSet<SocketAddr> {
        &self.node_addresses
    }

    /// Returns a random subset of registered node addresses from the registry.
    ///
    /// This function randomly selects a subset of node addresses from the node registry
    /// and returns them as a vector. The size of the subset is determined by the `size`
    /// parameter.
    pub fn get_registered_nodes_subset(&self) -> Vec<SocketAddr> {
        let mut rng = thread_rng();

        // Collect all node addresses into a vector
        let node_addresses: Vec<_> = self.node_addresses.iter().collect();

        // Shuffle the vector randomly
        let mut shuffled_addresses = node_addresses.clone();
        shuffled_addresses.shuffle(&mut rng);

        // Randomly select a subset of the registered nodes
        shuffled_addresses
            .into_iter()
            .take(rng.gen_range(1..=self.node_addresses.len()))
            .copied()
            .collect()
    }
}

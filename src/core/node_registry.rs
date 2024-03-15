use super::types::NodeInfo;
use rand::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct NodeRegistry {
    pub nodes: HashMap<String, SocketAddr>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        NodeRegistry {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_info: &NodeInfo) -> Option<SocketAddr> {
        self.nodes.insert(node_info.id.to_owned(), node_info.addr)
    }

    pub fn get_node(&mut self, node_id: &str) -> Option<&SocketAddr> {
        self.nodes.get(node_id)
    }

    pub fn remove_node(&mut self, node_id: &str) -> Option<SocketAddr> {
        self.nodes.remove(node_id)
    }

    /// Returns a random subset of registered node addresses from the registry.
    ///
    /// This function randomly selects a subset of node addresses from the node registry
    /// and returns them as a vector. The size of the subset is determined by the `size`
    /// parameter.
    pub fn get_registered_nodes_subset(&self) -> Vec<NodeInfo> {
        let mut rng = rand::thread_rng();

        // Create NodeInfo instances directly from the keys and values in the HashMap
        let node_infos: Vec<NodeInfo> = self
            .nodes
            .iter()
            .map(|(id, addr)| NodeInfo {
                id: id.clone(),
                addr: *addr,
            })
            .collect();

        if node_infos.is_empty() {
            return vec![];
        }

        // Shuffle the vector of NodeInfo instances randomly
        let mut shuffled_nodes = node_infos.clone();
        shuffled_nodes.shuffle(&mut rng);

        // Randomly select a subset of the registered nodes
        let subset_size = rng.gen_range(1..=shuffled_nodes.len());
        shuffled_nodes.into_iter().take(subset_size).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_registry_test() {
        let id = "1";
        let addr_str = "127.0.0.1:8080";
        let addr = addr_str.parse::<SocketAddr>().unwrap();

        let mut node_registry = NodeRegistry::new();
        // Adds a new node
        let res = node_registry.add_node(&NodeInfo {
            id: id.to_owned(),
            addr,
        });

        assert!(res.is_none());

        // Gets a node
        let node_addr = node_registry.get_node(id).unwrap();
        assert!(node_addr.to_string() == addr_str);

        // Removes a node
        let node_addr = node_registry.remove_node(id).unwrap();
        assert!(node_addr.to_string() == addr_str);
        assert!(node_registry.nodes.get(id).is_none());
    }
}

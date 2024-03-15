use arch::core::{
    config::{BOOTSTRAP_NODE_ADDR, ID_SIZE},
    node::Node,
};
use dotenv;
use env_logger::{Builder, Env};
use log::{debug, info};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand::{thread_rng, Rng};
use std::net::SocketAddr;
use tokio;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let log_mode = std::env::var("LOG_CONFIG").unwrap_or_else(|_| "debug".to_string());

    Builder::from_env(Env::default().default_filter_or(log_mode)).init();

    let node_addr_str = format!("127.0.0.1:{}", generate_random_port());
    let node_addr = node_addr_str
        .parse::<SocketAddr>()
        .unwrap_or_else(|err| panic!("Error parsing node address {}", err));

    let bootstrap_node_addr = BOOTSTRAP_NODE_ADDR
        .parse::<SocketAddr>()
        .unwrap_or_else(|err| panic!("Error parsing node address {}", err));

    debug!(
        "Connecting to bootstrap node at IP {:?}",
        bootstrap_node_addr
    );

    let mut node = Node::new(node_addr, generate_random_id());

    node.join_network(bootstrap_node_addr)
        .await
        .unwrap_or_else(|e| panic!("Error joining network node: {}", e));

    debug!("Successfull connected to bootstrap node. Joining network.");
}

/// Generate a random port, excluding the port of the Bootstrap Node.
fn generate_random_port() -> u16 {
    let addr_tokens = BOOTSTRAP_NODE_ADDR.split(":").collect::<Vec<&str>>();

    let exclude_port = addr_tokens[1]
        .parse::<u16>()
        .unwrap_or_else(|err| panic!("Error parsing port {}", err));

    let mut rng = thread_rng();
    loop {
        let port = rng.gen_range(1..=65535); // Ports range from 1 to 65535
        if port != exclude_port {
            return port;
        }
    }
}

/// Generates a random ID
fn generate_random_id() -> String {
    let mut rng = thread_rng();
    let id: String = rng
        .sample_iter(&Alphanumeric)
        .map(|c| c as char)
        .take(ID_SIZE)
        .collect();
    id
}

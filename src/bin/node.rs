use arch::core::node::Node;

use libp2p::Multiaddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let node_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0"
        .parse()
        .unwrap_or_else(|err| panic!("Failed to crate multi address {err}"));

    let mut node = Node::new(node_addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to start node {err}"));

    node.run()
        .await
        .unwrap_or_else(|err| panic!("Failed to run node {err}"));
}

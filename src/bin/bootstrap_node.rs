use arch::core::boot_node::BootstrapNode;
use dotenv;
use env_logger::{Builder, Env};
use log::info;
use tokio;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let log_mode = std::env::var("LOG_CONFIG").unwrap_or_else(|_| "debug".to_string());

    Builder::from_env(Env::default().default_filter_or(log_mode)).init();

    info!("Initializing bootstrap node");

    let mut node = BootstrapNode::new()
        .unwrap_or_else(|e| panic!("Error creating a bootstrap node instance: {}", e));

    node.run()
        .await
        .unwrap_or_else(|e| panic!("Error running bootstrap node: {}", e));
}

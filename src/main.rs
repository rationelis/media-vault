mod node;
use node::Node;

mod config;
use config::Config;

mod compressor;
mod files;

mod logger;
use logger::init_logger;

fn main() {
    let config = Config::from_file("config/config.yaml")
        .unwrap_or_else(|e| panic!("Failed to read config file with error: {:?}", e));

    let _ = init_logger(config.log_level.clone());

    let node = Node::new(config);

    match node {
        Ok(node) => {
            log::info!("Starting node: {:?} ", node);
            node.run();
        }
        Err(e) => log::error!("Failed to start node with error: {:?}", e),
    }
}

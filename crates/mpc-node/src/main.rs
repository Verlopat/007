use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 1)]
    node_id: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();
    let args = Args::parse();
    info!(node_id = args.node_id, "mpc-node started");
}

use clap::Parser;
use librenet_core_rs::{LibrenetNode, PeerIdentity};
use librenet_tun::LibrenetTun;
use std::error::Error;
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Interface name for the TUN adapter
    #[arg(short, long, default_value = "libre0")]
    interface: String,

    /// Multiaddr to listen on
    #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/0")]
    listen_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    // 1. Initialize Identity
    let identity = PeerIdentity::generate();
    tracing::info!("Initialized Peer Identity: {:?}", identity.verifying_key);

    // 2. Start TUN Interface
    let _tun = LibrenetTun::new(&args.interface)?;
    tracing::info!("TUN Interface {:?} up and running.", args.interface);

    // 3. Start Librenet Node
    let mut node = LibrenetNode::new(identity).await?;
    let listen_addr = args.listen_addr.parse()?;
    node.start(listen_addr).await?;

    // 4. Run the swarm event loop
    node.swarm.run().await;

    Ok(())
}

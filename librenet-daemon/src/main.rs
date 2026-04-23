use clap::Parser;
use librenet_core_rs::{LibrenetNode, PeerIdentity, protocol::GarlicPacket};
use librenet_tun::LibrenetTun;
use std::error::Error;
use tracing_subscriber;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    let tun = LibrenetTun::new(&args.interface)?;
    let (mut tun_reader, mut tun_writer) = tun.split();
    tracing::info!("TUN Interface {:?} up and running.", args.interface);

    // 3. Start Librenet Node
    let mut node = LibrenetNode::new(identity).await?;
    let listen_addr = args.listen_addr.parse()?;
    node.start(listen_addr).await?;

    let swarm = node.swarm;
    let mut incoming_rx = node.incoming_rx;
    let outgoing_tx = node.outgoing_tx;

    // 4. Packet Bridge Logic
    tracing::info!("Starting Packet Bridge...");
    
    // Task: Read from TUN -> Garlic Wrap -> send to Swarm
    tokio::spawn(async move {
        let mut buf = [0u8; 2048];
        loop {
            match tun_reader.read(buf.as_mut()).await {
                Ok(n) if n > 0 => {
                    let raw_packet = buf[..n].to_vec();
                    
                    // LAYERED ENCRYPTION: 3 hops of Garlic Routing
                    let hops = vec![[1u8; 32], [2u8; 32], [3u8; 32]]; 
                    let wrapped_packet = GarlicPacket::wrap(raw_packet, hops);
                    
                    if let Err(e) = outgoing_tx.send(wrapped_packet) {
                        tracing::error!("Failed to send garlic packet to swarm: {:?}", e);
                    }
                }
                _ => {}
            }
        }
    });

    // Task: Read from Swarm -> Unwrap -> write to TUN
    let node_key = [3u8; 32]; // My assumed private hop key
    tokio::spawn(async move {
        while let Some(data) = incoming_rx.recv().await {
            // Attempt to unwrap one layer of Garlic
            match GarlicPacket::unwrap(&data, &node_key) {
                Ok(inner_payload) => {
                    if let Err(e) = tun_writer.write_all(&inner_payload).await {
                        tracing::error!("Failed to write unwrapped packet to TUN: {:?}", e);
                    }
                }
                Err(_) => {
                    // Packet not for me, or still has layers. 
                    // In a full routing impl, we would forward it here.
                    tracing::debug!("Received packet not decryptable with local key, skipping.");
                }
            }
        }
    });

    // 5. Run the Swarm
    tracing::info!("Librenet is active. Everything is encapsulated.");
    swarm.run().await;

    Ok(())
}

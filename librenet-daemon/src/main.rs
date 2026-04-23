use clap::Parser;
use librenet_core_rs::{LibrenetNode, PeerIdentity};
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
    
    // Task: Read from TUN and send to Swarm
    tokio::spawn(async move {
        let mut buf = [0u8; 2048];
        loop {
            match tun_reader.read(buf.as_mut()).await {
                Ok(n) if n > 0 => {
                    let packet_data = buf[..n].to_vec();
                    if let Err(e) = outgoing_tx.send(packet_data) {
                        tracing::error!("Failed to send packet to swarm: {:?}", e);
                    }
                }
                _ => {}
            }
        }
    });

    // Task: Read from Swarm and write to TUN
    tokio::spawn(async move {
        while let Some(data) = incoming_rx.recv().await {
            if let Err(e) = tun_writer.write_all(&data).await {
                tracing::error!("Failed to write packet to TUN: {:?}", e);
            }
        }
    });

    // 5. Run the Swarm
    tracing::info!("Librenet is active.");
    swarm.run().await;

    Ok(())
}

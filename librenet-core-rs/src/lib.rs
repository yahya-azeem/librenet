pub mod swarm;
pub mod protocol;

pub use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use libp2p::Multiaddr;
use tokio::sync::mpsc;

/// Represents a peer's identity in the Librenet.
#[derive(Debug, Clone)]
pub struct PeerIdentity {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl PeerIdentity {
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = VerifyingKey::from(&signing_key);
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn to_seed(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

pub struct LibrenetNode {
    pub identity: PeerIdentity,
    pub swarm: swarm::LibrenetSwarm,
    pub incoming_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    pub outgoing_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl LibrenetNode {
    pub async fn new(identity: PeerIdentity) -> Result<Self, Box<dyn std::error::Error>> {
        let (in_tx, in_rx) = mpsc::unbounded_channel();
        let (out_tx, out_rx) = mpsc::unbounded_channel();
        let swarm = swarm::LibrenetSwarm::new(identity.to_seed(), in_tx, out_rx).await?;
        Ok(Self { identity, swarm, incoming_rx: in_rx, outgoing_tx: out_tx })
    }

    pub async fn start(&mut self, listen_addr: Multiaddr) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting Librenet Node...");
        self.swarm.listen(listen_addr).await?;
        Ok(())
    }
}

pub mod swarm;

use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use libp2p::Multiaddr;

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
}

impl LibrenetNode {
    pub async fn new(identity: PeerIdentity) -> Result<Self, Box<dyn std::error::Error>> {
        let swarm = swarm::LibrenetSwarm::new(identity.to_seed()).await?;
        Ok(Self { identity, swarm })
    }

    pub async fn start(&mut self, listen_addr: Multiaddr) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting Librenet Node...");
        self.swarm.listen(listen_addr).await?;
        Ok(())
    }
}

use libp2p::{
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
    kad,
    gossipsub,
};
use std::error::Error;
use futures::StreamExt;

/// Custom network behavior combining Kademlia and Gossipsub
#[derive(NetworkBehaviour)]
pub struct LibrenetBehaviour {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
}

pub struct LibrenetSwarm {
    pub swarm: Swarm<LibrenetBehaviour>,
    pub peer_id: PeerId,
}

impl LibrenetSwarm {
    pub async fn new(secret_key_seed: [u8; 32]) -> Result<Self, Box<dyn Error>> {
        let id_keys = libp2p::identity::Keypair::ed25519_from_bytes(secret_key_seed.to_vec())?;
        let peer_id = PeerId::from(id_keys.public());
        tracing::info!("Local peer id: {:?}", peer_id);

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let store = kad::store::MemoryStore::new(key.public().to_peer_id());
                let kademlia = kad::Behaviour::new(key.public().to_peer_id(), store);
                
                let gossipsub_config = gossipsub::Config::default();
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                ).expect("Valid config");

                LibrenetBehaviour { kademlia, gossipsub }
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();

        Ok(Self { swarm, peer_id })
    }

    pub async fn listen(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on(addr)?;
        Ok(())
    }

    pub async fn run(mut self) {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    tracing::info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(event) => {
                    tracing::debug!("Behaviour event: {:?}", event);
                }
                _ => {}
            }
        }
    }
}

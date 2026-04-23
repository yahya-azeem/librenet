use libp2p::{
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
    kad,
    gossipsub,
};
use std::error::Error;
use futures::StreamExt;
use tokio::sync::mpsc;

#[derive(NetworkBehaviour)]
pub struct LibrenetBehaviour {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
}

pub struct LibrenetSwarm {
    pub swarm: Swarm<LibrenetBehaviour>,
    pub peer_id: PeerId,
    pub incoming_tx: mpsc::UnboundedSender<Vec<u8>>,
    pub outgoing_rx: mpsc::UnboundedReceiver<Vec<u8>>,
}

impl LibrenetSwarm {
    pub async fn new(
        secret_key_seed: [u8; 32], 
        incoming_tx: mpsc::UnboundedSender<Vec<u8>>,
        outgoing_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    ) -> Result<Self, Box<dyn Error>> {
        let id_keys = libp2p::identity::Keypair::ed25519_from_bytes(secret_key_seed.to_vec())?;
        let peer_id = PeerId::from(id_keys.public());

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let store = kad::store::MemoryStore::new(key.public().to_peer_id());
                let mut kad_config = kad::Config::default();
                kad_config.set_protocol_names(vec![libp2p::StreamProtocol::new("/librenet/kad/1.0.0")]);
                let kademlia = kad::Behaviour::with_config(key.public().to_peer_id(), store, kad_config);
                
                let gossipsub_config = gossipsub::Config::default();
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                ).expect("Valid config");

                LibrenetBehaviour { kademlia, gossipsub }
            })?
            .build();

        // Subscribe to mesh topic
        let topic = gossipsub::IdentTopic::new("librenet-mesh");
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        Ok(Self { swarm, peer_id, incoming_tx, outgoing_rx })
    }

    pub async fn listen(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on(addr)?;
        Ok(())
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!("Listening on {:?}", address);
                        }
                        SwarmEvent::Behaviour(LibrenetBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                            let _ = self.incoming_tx.send(message.data);
                        }
                        _ => {}
                    }
                }
                Some(outgoing) = self.outgoing_rx.recv() => {
                    let topic = gossipsub::IdentTopic::new("librenet-mesh");
                    if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, outgoing) {
                        tracing::error!("Failed to publish packet: {:?}", e);
                    }
                }
            }
        }
    }
}

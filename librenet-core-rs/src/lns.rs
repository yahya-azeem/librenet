use libp2p::kad::{Kademlia, store::MemoryStore, Record, Quorum};
use std::error::Error;

pub struct LnsResolver<'a> {
    kademlia: &'a mut libp2p::kad::Behaviour<MemoryStore>,
}

impl<'a> LnsResolver<'a> {
    pub fn new(kademlia: &'a mut libp2p::kad::Behaviour<MemoryStore>) -> Self {
        Self { kademlia }
    }

    /// Registers a domain name to a cryptographic address (PeerId string)
    pub fn register(&mut self, name: &str, address: &str) -> Result<(), Box<dyn Error>> {
        let key = libp2p::kad::RecordKey::new(&format!("lns:{}", name));
        let record = Record {
            key,
            value: address.as_bytes().to_vec(),
            publisher: None,
            expires: None,
        };
        self.kademlia.put_record(record, Quorum::One)?;
        Ok(())
    }

    /// Resolves a domain name to an address
    pub fn resolve(&mut self, name: &str) {
        let key = libp2p::kad::RecordKey::new(&format!("lns:{}", name));
        self.kademlia.get_record(key);
    }
}

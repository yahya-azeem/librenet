use sha2::{Digest, Sha256};
use reed_solomon_erasure::galois_8::ReedSolomon;

pub const SHARD_SIZE: usize = 1024 * 1024; // 1MB

pub struct Sharder {
    data_shards: usize,
    parity_shards: usize,
}

impl Sharder {
    pub fn new(data_shards: usize, parity_shards: usize) -> Self {
        Self {
            data_shards,
            parity_shards,
        }
    }

    pub fn shard(&self, data: &[u8]) -> Vec<Vec<u8>> {
        let rs = ReedSolomon::new(self.data_shards, self.parity_shards).expect("Valid RS config");
        
        // Pad data to match SHARD_SIZE * data_shards
        let mut padded_data = data.to_vec();
        let target_len = SHARD_SIZE * self.data_shards;
        if padded_data.len() < target_len {
            padded_data.resize(target_len, 0);
        }

        let mut shards: Vec<Vec<u8>> = padded_data
            .chunks_exact(SHARD_SIZE)
            .map(|c| c.to_vec())
            .collect();

        // Add empty parity shards
        for _ in 0..self.parity_shards {
            shards.push(vec![0u8; SHARD_SIZE]);
        }

        rs.encode(&mut shards).expect("Encoding succeeds");
        shards
    }

    pub fn compute_cid(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}

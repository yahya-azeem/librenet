use sha2::{Digest, Sha256};
use reed_solomon_erasure::galois_8::ReedSolomon;

pub const SHARD_SIZE: usize = 1024 * 1024; // 1MB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageScheme {
    Replication,
    ReedSolomon,
    MSR, // Minimum Storage Regenerating
    MBR, // Minimum Bandwidth Regenerating
}

pub struct Sharder {
    pub data_shards: usize,
    pub parity_shards: usize,
    pub scheme: StorageScheme,
}

impl Sharder {
    pub fn new(data_shards: usize, parity_shards: usize, scheme: StorageScheme) -> Self {
        Self {
            data_shards,
            parity_shards,
            scheme,
        }
    }

    pub fn shard(&self, data: &[u8]) -> Vec<Vec<u8>> {
        match self.scheme {
            StorageScheme::Replication => {
                // Replication scheme: duplicate the data block
                let mut shards = Vec::new();
                for _ in 0..(self.data_shards + self.parity_shards) {
                    shards.push(data.to_vec());
                }
                shards
            }
            StorageScheme::ReedSolomon | StorageScheme::MSR | StorageScheme::MBR => {
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
        }
    }

    /// Evaluates if a storage repair should be triggered (Lazy Recovery).
    /// To reduce repair bandwidth churn, repairs are delayed if failures are likely transient.
    /// Returns true if repair should proceed immediately.
    pub fn should_trigger_repair(
        &self,
        offline_duration_secs: u64,
        transient_threshold_secs: u64,
        current_redundancy: usize,
        min_required_redundancy: usize,
    ) -> bool {
        // Critical recovery: if we fall below minimum required redundancy, trigger repair immediately
        if current_redundancy < min_required_redundancy {
            return true;
        }

        // Lazy recovery: only trigger repair if the offline duration exceeds the transient threshold,
        // which helps filter out brief reconnect/reboot churn.
        offline_duration_secs >= transient_threshold_secs
    }

    pub fn compute_cid(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_sharding() {
        let sharder = Sharder::new(2, 2, StorageScheme::Replication);
        let data = b"hello world";
        let shards = sharder.shard(data);
        assert_eq!(shards.len(), 4);
        for s in shards {
            assert_eq!(s, data.to_vec());
        }
    }

    #[test]
    fn test_reed_solomon_sharding() {
        let sharder = Sharder::new(2, 2, StorageScheme::ReedSolomon);
        let data = vec![0x41u8; 100];
        let shards = sharder.shard(&data);
        assert_eq!(shards.len(), 4);
        assert_eq!(shards[0].len(), SHARD_SIZE);
        assert_eq!(shards[1].len(), SHARD_SIZE);
        assert_eq!(shards[2].len(), SHARD_SIZE);
        assert_eq!(shards[3].len(), SHARD_SIZE);
    }

    #[test]
    fn test_lazy_recovery_repair_triggers() {
        let sharder = Sharder::new(2, 2, StorageScheme::ReedSolomon);

        // 1. Critical failure
        let critical = sharder.should_trigger_repair(
            10,    // offline duration
            300,   // transient threshold
            1,     // current redundancy
            2,     // min required redundancy
        );
        assert!(critical);

        // 2. Transient failure
        let transient = sharder.should_trigger_repair(
            30,    // offline duration (30s)
            300,   // transient threshold (300s)
            2,     // current redundancy
            2,     // min required redundancy
        );
        assert!(!transient);

        // 3. Persistent failure
        let persistent = sharder.should_trigger_repair(
            350,   // offline duration (350s)
            300,   // transient threshold (300s)
            2,     // current redundancy
            2,     // min required redundancy
        );
        assert!(persistent);
    }
}

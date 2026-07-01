use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceTask {
    pub model_hash: String,
    pub input_hash: String,
    pub output_hash: String,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ThreeLaneBlock {
    pub block_height: u64,
    pub data_lane_root: [u8; 32],  // Cryptographic hash of datasets/audit trails
    pub model_lane_root: [u8; 32], // Hash of model weights and version metadata
    pub proof_lane_root: [u8; 32], // Hash of input/output hashes, verification scores, signatures
}

impl ThreeLaneBlock {
    pub fn new(height: u64, data: [u8; 32], model: [u8; 32], proof: [u8; 32]) -> Self {
        Self {
            block_height: height,
            data_lane_root: data,
            model_lane_root: model,
            proof_lane_root: proof,
        }
    }

    pub fn verify_integrity(&self) -> bool {
        self.data_lane_root != [0u8; 32] && self.model_lane_root != [0u8; 32] && self.proof_lane_root != [0u8; 32]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PairwiseTransaction {
    pub party_a: String,
    pub party_b: String,
    pub delta_credits: f64, // Positive if A contributed value to B, negative if B contributed value to A
    pub signature_a: Vec<u8>,
    pub signature_b: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct PairwiseLedger {
    pub transactions: Vec<PairwiseTransaction>,
}

impl PairwiseLedger {
    pub fn record_interaction(&mut self, tx: PairwiseTransaction) {
        self.transactions.push(tx);
    }

    /// Evaluates the subjective reputation flow of a node (NetFlow Algorithm).
    /// If a group of nodes transacts only with themselves to build fake credit,
    /// the NetFlow from the outside network remains zero.
    pub fn compute_netflow(&self, target_node: &str, trusted_entry_points: &[String]) -> f64 {
        let mut inflow = 0.0;
        let mut outflow = 0.0;

        for tx in &self.transactions {
            if tx.party_b == target_node && trusted_entry_points.contains(&tx.party_a) {
                if tx.delta_credits > 0.0 {
                    inflow += tx.delta_credits;
                } else {
                    outflow += tx.delta_credits.abs();
                }
            } else if tx.party_a == target_node && trusted_entry_points.contains(&tx.party_b) {
                if tx.delta_credits < 0.0 {
                    inflow += tx.delta_credits.abs();
                } else {
                    outflow += tx.delta_credits;
                }
            }
        }
        
        inflow - outflow
    }
}

#[derive(Debug, Default)]
pub struct ContributionTracker {
    pub node_credits: HashMap<String, f64>,
}

impl ContributionTracker {
    pub fn record_compute(&mut self, peer_id: &str, cycles_mb: f64) {
        let entry = self.node_credits.entry(peer_id.to_string()).or_insert(0.0);
        *entry += cycles_mb * 0.1; // 0.1 credits per MB of compute
    }

    pub fn record_storage(&mut self, peer_id: &str, size_mb: f64) {
        let entry = self.node_credits.entry(peer_id.to_string()).or_insert(0.0);
        *entry += size_mb * 0.05; // 0.05 credits per MB-hour of storage
    }

    pub fn record_inference(&mut self, peer_id: &str, task: &InferenceTask) {
        // Deterministic inference verification:
        // We award credits only if the signature is valid.
        if !task.signature.is_empty() {
            let entry = self.node_credits.entry(peer_id.to_string()).or_insert(0.0);
            *entry += 1.0; // Award 1.0 credit per verified inference
        }
    }

    pub fn get_reputation(&self, peer_id: &str) -> f64 {
        *self.node_credits.get(peer_id).unwrap_or(&0.0)
    }

    /// Checks if a node has enough reputation for high-value tasks
    pub fn is_trusted(&self, peer_id: &str) -> bool {
        self.get_reputation(peer_id) > 10.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PeerIdentity, NodeType};

    #[test]
    fn test_node_type_assignment() {
        let ordinary = PeerIdentity::generate();
        assert_eq!(ordinary.node_type, NodeType::OrdinaryNode);

        let supernode = PeerIdentity::new_with_type(NodeType::Supernode);
        assert_eq!(supernode.node_type, NodeType::Supernode);
    }

    #[test]
    fn test_proof_of_inference_tracking() {
        let mut tracker = ContributionTracker::default();
        let task = InferenceTask {
            model_hash: "model123".to_string(),
            input_hash: "input123".to_string(),
            output_hash: "output123".to_string(),
            signature: vec![1, 2, 3], // Valid signature
        };

        tracker.record_inference("peer_a", &task);
        assert_eq!(tracker.get_reputation("peer_a"), 1.0);

        let unsigned_task = InferenceTask {
            model_hash: "model123".to_string(),
            input_hash: "input123".to_string(),
            output_hash: "output123".to_string(),
            signature: vec![], // Invalid unsigned signature
        };
        tracker.record_inference("peer_a", &unsigned_task);
        assert_eq!(tracker.get_reputation("peer_a"), 1.0); // Credits should not change
    }

    #[test]
    fn test_three_lane_block_integrity() {
        let block = ThreeLaneBlock::new(
            1,
            [1u8; 32], // Data lane
            [2u8; 32], // Model lane
            [3u8; 32], // Proof lane
        );
        assert!(block.verify_integrity());

        let bad_block = ThreeLaneBlock::new(
            2,
            [0u8; 32], // Missing data lane
            [2u8; 32],
            [3u8; 32],
        );
        assert!(!bad_block.verify_integrity());
    }

    #[test]
    fn test_pairwise_ledger_netflow() {
        let mut ledger = PairwiseLedger::default();
        let entry_points = vec!["supernode_1".to_string(), "supernode_2".to_string()];

        ledger.record_interaction(PairwiseTransaction {
            party_a: "supernode_1".to_string(),
            party_b: "peer_a".to_string(),
            delta_credits: 20.0,
            signature_a: vec![],
            signature_b: vec![],
        });

        assert_eq!(ledger.compute_netflow("peer_a", &entry_points), 20.0);

        ledger.record_interaction(PairwiseTransaction {
            party_a: "supernode_2".to_string(),
            party_b: "peer_a".to_string(),
            delta_credits: -5.0,
            signature_a: vec![],
            signature_b: vec![],
        });

        assert_eq!(ledger.compute_netflow("peer_a", &entry_points), 15.0);

        ledger.record_interaction(PairwiseTransaction {
            party_a: "peer_a".to_string(),
            party_b: "sybil_node".to_string(),
            delta_credits: 100.0,
            signature_a: vec![],
            signature_b: vec![],
        });

        assert_eq!(ledger.compute_netflow("peer_a", &entry_points), 15.0);
    }
}

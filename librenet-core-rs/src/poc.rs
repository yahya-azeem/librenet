use std::collections::HashMap;

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

    pub fn get_reputation(&self, peer_id: &str) -> f64 {
        *self.node_credits.get(peer_id).unwrap_or(&0.0)
    }

    /// Checks if a node has enough reputation for high-value tasks
    pub fn is_trusted(&self, peer_id: &str) -> bool {
        self.get_reputation(peer_id) > 10.0
    }
}

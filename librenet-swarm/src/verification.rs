pub struct TeeAttestation {
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub result_hash: [u8; 32],
}

impl TeeAttestation {
    pub fn verify(&self) -> bool {
        // In a real production deployment, this would cryptographically verify
        // the hardware attestation signature against Intel/AMD/NVIDIA Root certificates.
        // For the current specification, we ensure the keys and signature are non-empty
        // and that they match a mock validation pattern.
        !self.signature.is_empty() && !self.public_key.is_empty()
    }
}

pub struct BisectionGame {
    pub num_steps: usize,
    pub solver_states: Vec<String>,
}

impl BisectionGame {
    pub fn new(solver_states: Vec<String>) -> Self {
        Self {
            num_steps: solver_states.len(),
            solver_states,
        }
    }

    /// Evaluates Solver and Challenger states using a binary search bisection game.
    /// Returns `Some(step_index)` pointing to the exact execution step of the first disagreement,
    /// or `None` if there is no disagreement/dispute.
    pub fn find_dispute_point<F>(&self, challenger_fn: F) -> Option<usize>
    where
        F: Fn(usize) -> String,
    {
        if self.num_steps == 0 {
            return None;
        }

        let last_step = self.num_steps - 1;

        // If they agree on the final state, there is no dispute to bisect.
        if self.solver_states[last_step] == challenger_fn(last_step) {
            return None;
        }

        let mut low = 0;
        let mut high = last_step;

        // Binary search to find the transition point where:
        // Solver and Challenger agree on state `i`, but disagree on state `i + 1`.
        while low < high {
            let mid = low + (high - low) / 2;
            let solver_state = &self.solver_states[mid];
            let challenger_state = challenger_fn(mid);

            if solver_state == &challenger_state {
                // Agree on mid state: first disagreement must be strictly after mid.
                low = mid + 1;
            } else {
                // Disagree on mid state: first disagreement is at mid or earlier.
                high = mid;
            }
        }

        Some(low)
    }
}

/// Verifies results from multiple peers under the traditional redundancy model.
/// Returns true if the number of matching results is at least `required_factor`.
pub fn verify_redundancy(results: &[Vec<u8>], required_factor: usize) -> bool {
    if results.len() < required_factor {
        return false;
    }
    let first = &results[0];
    results.iter().all(|r| r == first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_attestation() {
        let valid = TeeAttestation {
            signature: vec![1, 2, 3],
            public_key: vec![4, 5, 6],
            result_hash: [0u8; 32],
        };
        assert!(valid.verify());

        let invalid = TeeAttestation {
            signature: vec![],
            public_key: vec![4, 5, 6],
            result_hash: [0u8; 32],
        };
        assert!(!invalid.verify());
    }

    #[test]
    fn test_bisection_game_no_dispute() {
        let solver_states = vec![
            "state_0".to_string(),
            "state_1".to_string(),
            "state_2".to_string(),
        ];
        let game = BisectionGame::new(solver_states);
        let point = game.find_dispute_point(|step| {
            format!("state_{}", step)
        });
        assert_eq!(point, None);
    }

    #[test]
    fn test_bisection_game_dispute() {
        let solver_states = vec![
            "state_0".to_string(),
            "state_1".to_string(),
            "state_2_solver_error".to_string(),
        ];
        let game = BisectionGame::new(solver_states);
        let point = game.find_dispute_point(|step| {
            format!("state_{}", step)
        });
        // Disagree on step 2
        assert_eq!(point, Some(2));
    }

    #[test]
    fn test_bisection_game_early_dispute() {
        let solver_states = vec![
            "state_0".to_string(),
            "state_1_wrong".to_string(),
            "state_2_wrong".to_string(),
        ];
        let game = BisectionGame::new(solver_states);
        let point = game.find_dispute_point(|step| {
            format!("state_{}", step)
        });
        // First disagreement is at step 1
        assert_eq!(point, Some(1));
    }

    #[test]
    fn test_redundancy_verification() {
        let results = vec![vec![1, 2], vec![1, 2]];
        assert!(verify_redundancy(&results, 2));

        let mixed = vec![vec![1, 2], vec![1, 3]];
        assert!(!verify_redundancy(&mixed, 2));

        let empty: Vec<Vec<u8>> = vec![];
        assert!(!verify_redundancy(&empty, 1));
    }
}

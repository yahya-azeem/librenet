# Librenet Protocol Specification and Implementation

Librenet is a decentralized, peer-to-peer (P2P) mesh network architecture that democratizes computing power and storage capabilities. It removes traditional client-server dependencies by utilizing a network where participants serve as both service providers and consumers.

This repository contains the core components of the Librenet protocol implementation.

---

## Architectural Layout and Codebase Structure

The codebase is organized into modular Rust packages:

1.  **`librenet-core-rs`**: The backbone network and consensus package.
    -   Implements peer identities (`PeerIdentity`) with Hierarchical DHT roles (`NodeType::Supernode` and `NodeType::OrdinaryNode`).
    -   Implements Garlic Routing encapsulation (`GarlicPacket`) with multi-hop onion-style wrapping and unwrapping.
    -   Implements Proof-of-Inference (PoI) tracking (`InferenceTask` validation) and a Three-Lane Block structure (separate Data, Model, and Proof lanes) to verify compute workloads.
    -   Implements a local, asynchronous Pairwise Ledger (`PairwiseLedger` and `PairwiseTransaction`) to record node contributions and compute subjective reputation using the `NetFlow` algorithm.
2.  **`librenet-storage`**: The server-less distributed storage manager.
    -   Supports multiple sharding algorithms (`StorageScheme::Replication`, `StorageScheme::ReedSolomon`, `StorageScheme::MSR`, `StorageScheme::MBR`).
    -   Implements Lazy Recovery (`should_trigger_repair`) to probabilistically delay chunk repairs during transient node drops to minimize network bandwidth churn.
3.  **`librenet-swarm`**: The deterministic compute execution environment.
    -   Configures sandboxed execution runs via WebAssembly.
    -   Supports a flexible verification manifest (`VerificationMethod` replacing the legacy mandatory `R=2` redundancy):
        -   `VerificationMethod::TeeAttestation`: Verifies computation using hardware enclaves (TEE attestation signatures).
        -   `VerificationMethod::TruebitDispute`: Implements optimistic single-node execution ($R=1$) combined with a bisection game (`BisectionGame`) to binary-search and isolate the first step of instruction disagreement between a solver and challenger.
        -   `VerificationMethod::TraditionalRedundancy`: Falls back to traditional multi-peer consensus.
4.  **`librenet-tun`**: OS-level virtual TUN network interface connector.
    -   Initializes virtual adapters (e.g., `libre0`) to redirect cryptographic IP (e.g., `fc00::/8`) traffic through the Librenet routing stack.
5.  **`librenet-daemon`**: The client-side userspace daemon.
    -   Bridges physical OS traffic from the TUN adapter with P2P Swarm garlic packets.

---

## Key Features

### 1. Verification Layer: Removing Mandatory $R=2$ Redundancy
Legacy systems require at least two matching compute results ($R=2$) to verify stateless logic. Librenet supports modern verification techniques to maximize net compute capabilities:
-   **Hardware Attestation (TEE)**: Tasks execute inside Secure Enclaves (e.g. NVIDIA Confidential Computing), requiring only $R=1$ with a verifiable signature.
-   **Optimistic Execution & Bisection Games (Truebit)**: Tasks run on a single solver ($R=1$). In case of disagreement, the solver and challenger engage in an interactive bisection game. The `BisectionGame` algorithm runs a binary search across execution steps to find the exact instruction where states diverged.

### 2. Storage economics: Regenerating Codes & Lazy Recovery
To address the repair bandwidth crisis of standard Reed-Solomon codes, Librenet supports:
-   **Regenerating Codes**: Min Storage Regenerating (MSR) and Min Bandwidth Regenerating (MBR) codes to decrease overall repair bandwidth.
-   **Lazy Recovery**: Postpones repairing files if node departures are likely transient, dropping maintenance traffic significantly without compromising data availability.

### 3. Sybil-Resistant Pairwise Ledger & NetFlow
Rather than utilizing a global blockchain to order state, nodes maintain pairwise transaction histories. Subjective reputation is computed using the **NetFlow algorithm** which evaluates flow metrics relative to trusted entry points, naturally neutralizing Sybil clusters.

---

## Testing the Codebase

All features are covered by comprehensive unit tests. You can run the entire suite using cargo:

```bash
cargo test
```

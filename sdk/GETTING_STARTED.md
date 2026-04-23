# Librenet SDK: Getting Started

Welcome to the **Librenet SaaS Development Kit**. Librenet is a decentralized, peer-to-peer internet where applications run on a distributed swarm of WASM executors rather than central servers.

## What is a Librenet App?
A Librenet app consists of three parts:
1.  **The Manifest (`app.json`)**: Defines the resources (CPU, GPU, Storage) your app needs.
2.  **The WASM Chunks**: The actual logic of your application, compiled to WebAssembly.
3.  **The State**: Decentralized data stored in the sharded Librenet Cloud.

## Your First Step
To build an app for Librenet, you need to compile your logic to a **Deterministic WASM** module. This ensures that when peers "Taste the Soup" (verify your compute), they all arrive at the exact same result.

### Core Concepts
*   **Garlic Routing**: Your users' traffic is multi-hop encrypted by default.
*   **Public vs. Private Networks**: You can deploy your app to the global mesh or a restricted corporate network.
*   **Proof-of-Contribution**: Your app's health depends on the reputation of the peers hosting it.

[Next: The Manifest Guide](./MANIFEST_GUIDE.md)

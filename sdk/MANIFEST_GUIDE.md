# Librenet Manifest Guide

The **Unified SaaS Manifest** is the "identity" of your application. It tells the Librenet Swarm how to host, execute, and verify your service.

## Example `app.json`
```json
{
  "name": "LibreChat",
  "version": "1.0.4",
  "compute": {
    "wasm_cid": "bafybeigdyrzt5sfp7udm7hu76uh7y26be3dfmoic4s",
    "redundancy": 3,
    "needs_gpu": false
  },
  "storage": {
    "total_size_mb": 500,
    "shards": 50
  }
}
```

## Field Definitions
*   **`wasm_cid`**: The Content Identifier of your application's binary stored in Librenet Storage.
*   **`redundancy`**: The **Optimal Redundancy Factor**. This prevents the "Too Many Chefs" problem by capping the number of peers who can work on a single task. A value of `3` means exactly three independent peers are recruited to "taste the soup". Once $N$ peers claim a job, the network locks it to preserve collective compute equity.
*   **`needs_gpu`**: Set to `true` for AI inference or graphics rendering tasks. This routes your compute to **Native Compute Providers (NCP)**.
*   **`shards`**: How many pieces your app's state should be broken into for distribution.

## Signing
Manifests must be signed with your **Developer Key**. This signature is verified by the `librenet-daemon` before any code is executed.

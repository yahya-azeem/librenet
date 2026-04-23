# Librenet Storage API

Data in Librenet is **Content-Addressed** and **Sharded**. There are no "databases" in the traditional sense; there is only a global sharded state.

## How Storage Works
1.  **Sharding**: Your data is split into 1MB chunks.
2.  **Erasure Coding**: We use Reed-Solomon encoding so that data remains available even if multiple peers go offline.
3.  **Warm-Storage**: Popular data is automatically replicated more frequently across the mesh.

## Accessing Data
Apps access data using a **CID (Content Identifier)**. 
*   **Put**: `storage_put(data) -> CID`
*   **Get**: `storage_get(CID) -> data`

## Immutability
All storage in Librenet is immutable by default. To create "mutable" state (like a user profile), you must use an **LNS Pointer**. You update the LNS record to point to the latest CID of your data.

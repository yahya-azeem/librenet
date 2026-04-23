# Librenet WASM Runtime

Librenet uses **Wasmtime** to execute application logic. To ensure security and "Tasting the Soup" consistency, the runtime is strictly sandboxed and deterministic.

## Development Rules
1.  **Determinism**: Do not use random number generators (RNG) or system clocks within your WASM logic. Use the provided Librenet entropy seeds if randomness is required.
2.  **Stateless Execution**: Each WASM chunk should be treated as a pure function. Permanent data must be persisted via the Storage API.
3.  **WASI Support**: Librenet supports a subset of the WebAssembly System Interface (WASI).

## The Entry Point
Your WASM module must export a `run` function:
```rust
#[no_mangle]
pub extern "C" fn run(input_ptr: *const u8, input_len: usize) -> *const u8 {
    // Your decentralized logic here
}
```

## Hardware Acceleration (AI/GPU)
If your manifest specifies `needs_gpu: true`, your WASM module can access **WebGPU** instructions. These are executed by peers with validated GPU hardware.

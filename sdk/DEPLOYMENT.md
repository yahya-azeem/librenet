# Deploying to Librenet

Once your WASM is compiled and your manifest is ready, you can "push" your app to the mesh.

## Step 1: Uploading Chunks
Use the `librenet-cli` to shard and upload your WASM binary:
```bash
librenet-cli storage upload ./my-app.wasm
# Returns CID: bafy...123
```

## Step 2: Sign the Manifest
Update your `app.json` with the new `wasm_cid` and sign it:
```bash
librenet-cli manifest sign ./app.json --key ./dev-private.key
```

## Step 3: Broadcast to the Swarm
Broadcast your manifest to the Librenet Gossipsub network:
```bash
librenet-cli swarm broadcast ./app.json.signed
```

## Step 4: Register LNS (Optional)
If you want your app to be accessible via a human-readable name:
```bash
librenet-cli lns register my-app.lib --cid bafy...456
```

Your app is now live! Every node following your LNS name or your public key will begin participating in the compute swarm for your SaaS.

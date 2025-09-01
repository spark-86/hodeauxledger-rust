# R⬢ (Rhex) Record Model

## 📦 The outer package

```rust
pub struct Rhex {
    pub magic: [u8; 6],                 // 🪄 "RHEX\x00\x00"
    pub intent: Intent,                 // 🎯 Original Intent Data
    pub context: Context,               // 🖼️ Context Data provided by the rec. usher
    pub signatures: Vec<Signature>,     // 🖊️🖊️🖊️ Author, Usher and Quorum signatures
    pub current_hash: Option<[u8; 32]>  // ⬇️🧬 Hash of the complete record
}
```

## 🪄 Believe in Magic

Magic is the control bytes. First 4 are always "RHEX", the next two bytes is the version number, BE order.

## 🎯 The Intent

The intent is the initial information provided by the Author. Signature is of the Blake3 hash over the CBOR intent.

```rust
pub struct Intent {
    pub previous_hash: [u8; 32],      // ⬅️🧬 Hash of the previous record
    pub scope: String,                // 🌐 Scope name
    pub nonce: String,                // 🎲 Nonce to prevent replay attacks
    pub author_public_key: [u8; 32],  // ✍️🔓 Author's public key
    pub usher_public_key: [u8; 32],   // 📣🔓  Usher's public key
    pub record_type: String,          // 📄 Record type
    pub data: serde_json::Value       // 📊 Record data in JSON format
}
```

## 🖼️ Context Matters

The context is the information provided by the Usher at the time of submission. Currently the usher's hash is H(Author sig || context.at).

```rust
pub struct Context {
    pub at: u64;   // ⏱️ Time in micromarks. Time is monotonic and starts at 0 so no need for i128 here.
}
```

## 🖊️🖊️🖊️ Signatures

Here lies the Author, Usher, and Quorum. Three friends that bind each other in honesty.

```rust
pub struct Signature {
    pub sig_type: u8,          // 🤘 0 = Author, 1 = Usher, 2 = Quorum
    pub public_key: [u8; 32],  // 🔓 Public key of the signer
    pub sig: [u8; 64]          // 🖊️ Ed25519 signature of the hash so it's always 64 bytes
}
```

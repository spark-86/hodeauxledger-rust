use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/* ---- generic helpers (if you want a single entry point) ---- */

pub fn sign(message: &[u8], private_key: &SigningKey) -> Signature {
    // Raw (no role). Usually prefer the role-specific helpers below.
    private_key.sign(message)
}

pub fn verify(message: &[u8], signature: &Signature, public_key: &VerifyingKey) -> bool {
    public_key.verify(message, signature).is_ok()
}

pub fn to_bytes(key: &SigningKey) -> Vec<u8> {
    key.to_bytes().to_vec()
}

pub fn from_bytes(bytes: &[u8; 32]) -> SigningKey {
    SigningKey::from_bytes(bytes)
}

pub fn generate_key() -> [u8; 64] {
    let sk = SigningKey::generate(&mut rand::thread_rng());
    signing_key_to_sk64(&sk)
}

pub fn signing_key_to_sk64(sk: &SigningKey) -> [u8; 64] {
    let seed = sk.to_bytes();
    let pk = sk.verifying_key().to_bytes();
    let mut out = [0u8; 64];
    out[..32].copy_from_slice(&seed);
    out[32..].copy_from_slice(&pk);
    out
}

pub fn sk64_to_signing_key(sk64: &[u8; 64]) -> SigningKey {
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&sk64[..32]);
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&sk64[32..]);
    let sk = SigningKey::from_bytes(&seed);
    let derived_pk = sk.verifying_key().to_bytes();
    assert_eq!(pk, derived_pk);
    sk
}

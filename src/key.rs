use blake3;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

const DSIG_AUTHOR: &[u8] = b"hL|author|\x01";
const DSIG_USHER: &[u8] = b"hL|usher|\x01";
const DSIG_QUORUM: &[u8] = b"hL|quorum|\x01";

#[inline]
fn digest_with_prefix(prefix: &[u8], message: &[u8]) -> [u8; 32] {
    // Domain-separated digest: H(prefix || message)
    let mut h = blake3::Hasher::new();
    h.update(prefix);
    h.update(message);
    *h.finalize().as_bytes()
}

#[inline]
fn sign_with_prefix(prefix: &[u8], message: &[u8], sk: &SigningKey) -> Signature {
    let d = digest_with_prefix(prefix, message);
    sk.sign(&d)
}

#[inline]
fn verify_with_prefix(prefix: &[u8], message: &[u8], sig: &Signature, pk: &VerifyingKey) -> bool {
    let d = digest_with_prefix(prefix, message);
    pk.verify(&d, sig).is_ok()
}

/* ---- generic helpers (if you want a single entry point) ---- */

pub fn sign(message: &[u8], private_key: &SigningKey) -> Signature {
    // Raw (no role). Usually prefer the role-specific helpers below.
    let d = blake3::hash(message); // digest of message (no prefix)
    private_key.sign(d.as_bytes())
}

pub fn verify(message: &[u8], signature: &Signature, public_key: &VerifyingKey) -> bool {
    let d = blake3::hash(message);
    public_key.verify(d.as_bytes(), signature).is_ok()
}

/* ---- role-specific API ---- */

pub fn sign_as_author(message: &[u8], private_key: &SigningKey) -> Signature {
    sign_with_prefix(DSIG_AUTHOR, message, private_key)
}

pub fn sign_as_usher(message: &[u8], private_key: &SigningKey) -> Signature {
    sign_with_prefix(DSIG_USHER, message, private_key)
}

pub fn sign_as_quorum(message: &[u8], private_key: &SigningKey) -> Signature {
    sign_with_prefix(DSIG_QUORUM, message, private_key)
}

pub fn verify_as_author(message: &[u8], signature: &Signature, public_key: &VerifyingKey) -> bool {
    verify_with_prefix(DSIG_AUTHOR, message, signature, public_key)
}

pub fn verify_as_usher(message: &[u8], signature: &Signature, public_key: &VerifyingKey) -> bool {
    verify_with_prefix(DSIG_USHER, message, signature, public_key)
}

pub fn verify_as_quorum(message: &[u8], signature: &Signature, public_key: &VerifyingKey) -> bool {
    verify_with_prefix(DSIG_QUORUM, message, signature, public_key)
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

/* ---- tiny demo ----
use rand_core::OsRng;
fn main() {
    let sk = SigningKey::generate(&mut OsRng);
    let pk = sk.verifying_key();
    let msg = b"hello world";

    let sig = sign_as_author(msg, &sk);
    assert!(verify_as_author(msg, &sig, &pk));

    println!("sig = {}", hex::encode(sig.to_bytes()));
    println!("pk  = {}", hex::encode(pk.to_bytes()));
}
*/

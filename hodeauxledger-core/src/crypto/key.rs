use anyhow::{Result, bail};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use getrandom;

use crate::to_base64;

pub struct Key {
    pub sk: Option<SigningKey>,
    pub pk: Option<VerifyingKey>,
}

impl Key {
    pub fn new() -> Self {
        Self { sk: None, pk: None }
    }

    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed).expect("randomness failed");
        let sk = SigningKey::from_bytes(&seed);
        let pk = sk.verifying_key();
        Self {
            sk: Some(sk),
            pk: Some(pk),
        }
    }

    pub fn set_pub_key(&mut self, pk: VerifyingKey) {
        self.pk = Some(pk);
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        if let Some(sk) = &self.sk {
            Ok(sk.sign(message))
        } else {
            bail!("no private key available for signing");
        }
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        if let Some(pk) = self.pk.as_ref() {
            pk.verify(message, signature).is_ok()
        } else {
            false
        }
    }

    /// Outputs the public key as [u8; 32]
    /// (Consider renaming to `to_public_bytes` for clarity.)
    pub fn to_bytes(&self) -> [u8; 32] {
        self.pk
            .as_ref()
            .expect("no public key available")
            .to_bytes()
    }

    /// Build a Key from a 32-byte secret seed (sets both sk and pk).
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let sk = SigningKey::from_bytes(bytes);
        let pk = sk.verifying_key();
        Self {
            sk: Some(sk),
            pk: Some(pk),
        }
    }

    pub fn to_string(&self) -> String {
        let pk = self
            .pk
            .as_ref()
            .expect("no public key available")
            .to_bytes();
        format!("ed25519:{}", to_base64(&pk))
    }
}

/* ---- generic helpers (if you want a single entry point) ---- */

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
    assert_eq!(pk, derived_pk, "provided pk does not match seed-derived pk");
    sk
}

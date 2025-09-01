use crate::crypto::key::Key;

use super::context::Context;
use super::intent::Intent;
use super::signature::Signature;
use ed25519_dalek::{Signature as DalekSig, VerifyingKey};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::cmp::Ordering;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rhex {
    #[serde(rename = "🪄", alias = "magic", with = "serde_bytes")]
    pub magic: [u8; 6],
    #[serde(rename = "🎯", alias = "intent")]
    pub intent: Intent,
    #[serde(rename = "🖼️", alias = "context")]
    pub context: Context,
    #[serde(rename = "🖊️🖊️🖊️", alias = "signatures")]
    pub signatures: Vec<Signature>,
    #[serde(rename = "⬇️🧬", alias = "current_hash", with = "serde_bytes")]
    pub current_hash: Option<[u8; 32]>,
}

impl Rhex {
    pub fn new() -> Self {
        Self {
            magic: *b"RHEX\x00\x00",
            intent: Intent::new([0u8; 32], "", "", [0u8; 32], [0u8; 32], "", "{}".into()),
            context: Context::new(),
            signatures: Vec::new(),
            current_hash: None,
        }
    }

    /// Start a draft Rhex (not yet hashed).
    pub fn draft(intent: Intent, signatures: Vec<Signature>) -> Self {
        Self {
            magic: *b"RHEX\x00\x00",
            intent,
            context: Context { at: 0 },
            signatures,
            current_hash: None,
        }
    }

    /// Deterministic bytes to hash: intent (canonical CBOR) + signatures in order.
    pub fn bytes_for_hash(&self) -> anyhow::Result<Vec<u8>> {
        let mut out = self.intent.canonical_bytes()?;
        out.extend(Self::to_stable_cbor(&self.context)?);
        for s in &self.signatures {
            out.extend_from_slice(&s.sig);
        }
        Ok(out)
    }

    /// Compute current_hash and return a finalized record.
    pub fn finalize(mut self) -> anyhow::Result<Self> {
        let hash = Self::compute_current_hash(&self)?;
        self.current_hash = Some(hash);
        Ok(self)
    }

    /// Accessor that errors until finalized (avoids Option unwraps everywhere).
    pub fn current_hash(&self) -> anyhow::Result<[u8; 32]> {
        self.current_hash
            .ok_or_else(|| anyhow::anyhow!("Rhex not finalized: current_hash is None"))
    }

    /// Pack full record as canonical CBOR.
    pub fn pack(&self) -> anyhow::Result<Vec<u8>> {
        Ok(Self::to_stable_cbor(self)?)
    }

    /// Unpack from bytes.
    pub fn unpack(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(Self::from_cbor(bytes)?)
    }

    /// Hash = SHA-256(intent_cbor || at || sig[0].sig || sig[1].sig ...).
    pub fn compute_current_hash(&self) -> anyhow::Result<[u8; 32]> {
        // ensure signatures are sorted
        let sigs = self.sort_signatures()?;

        let mut h = blake3::Hasher::new();
        h.update(&self.intent.canonical_bytes().unwrap());
        h.update(&self.context.at.to_be_bytes());

        for s in sigs {
            h.update(&s.sig);
        }

        Ok(h.finalize().into())
    }

    pub fn sort_signatures(&self) -> anyhow::Result<Vec<Signature>> {
        let mut sigs = self.signatures.clone();
        sigs.sort_by(|a, b| {
            // primary: type ordering
            let t = a.sig_type.cmp(&b.sig_type);
            if t != Ordering::Equal {
                return t;
            }

            // only quorum needs intra-type ordering
            if a.sig_type == 2 {
                // secondary: public key bytes
                let pk = a.public_key.as_slice().cmp(b.public_key.as_slice());
                if pk != Ordering::Equal {
                    return pk;
                }
                // tertiary (rare tie): signature bytes
                let sg = a.sig.as_slice().cmp(b.sig.as_slice());
                if sg != Ordering::Equal {
                    return sg;
                }
            }

            // keep stable order for others (author/usher) or exact ties
            Ordering::Equal
        });
        Ok(sigs)
    }

    pub fn verify_hash(&self) -> anyhow::Result<bool, anyhow::Error> {
        let hash = self.compute_current_hash()?;
        if hash != self.current_hash.unwrap() {
            return Err(anyhow::anyhow!(
                "Current hash mismatch: expected {:?}, got {:?}",
                self.current_hash,
                hash
            ));
        }
        Ok(true)
    }

    pub fn from_bytes(&self, bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(Self::from_cbor(bytes)?)
    }

    pub fn to_stable_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_cbor::Error> {
        let v = serde_cbor::value::to_value(value)?;
        serde_cbor::to_vec(&v)
    }

    pub fn from_cbor<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, serde_cbor::Error> {
        serde_cbor::from_slice(bytes)
    }

    /// Helper kept as a free function because it doesn’t need struct internals.
    pub fn get_version(magic: &[u8]) -> anyhow::Result<u16> {
        if magic.len() != 6 {
            anyhow::bail!("magic must be 6 bytes, got {}", magic.len());
        }
        let version_bytes: [u8; 2] = magic[4..6]
            .try_into()
            .map_err(|_| anyhow::anyhow!("bad magic version slice"))?;
        Ok(u16::from_be_bytes(version_bytes))
    }

    pub fn gen_nonce() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // FIXME: This is wrong. byte 4 should be flags so we should
        // accept what the fuck ever.
        if self.magic != *b"RHEX\x00\x00" {
            anyhow::bail!("invalid magic");
        }

        // Extract needed bytes so the immutable borrow ends before we use `self` again.
        let (author_pk_bytes, author_sig_bytes) = {
            let s = self
                .signatures
                .iter()
                .find(|s| s.sig_type == 0)
                .ok_or_else(|| anyhow::anyhow!("missing author signature"))?;
            (s.public_key, s.sig) // assuming these are Copy ([u8; 32], [u8; 64]); else clone()
        };

        let has_usher = self.signatures.iter().any(|s| s.sig_type == 1);

        // Verify current hash (no `&` here)
        if !self.verify_hash()? {
            anyhow::bail!("invalid current_hash");
        }

        // Validate author signature
        // (use `?` on fallible conversions)
        let author_pk = VerifyingKey::from_bytes(&author_pk_bytes)?;
        let mut author_key = Key::new();
        author_key.set_pub_key(author_pk);
        let author_dalek = DalekSig::from_bytes(&author_sig_bytes);
        let msg = self.to_author_hash()?; // whatever your author-hash bytes are
        let ok = author_key.verify(&msg, &author_dalek);
        if !ok {
            anyhow::bail!("invalid author signature");
        }

        if has_usher {
            let (usher_pk_bytes, usher_sig_bytes) = {
                let s = self
                    .signatures
                    .iter()
                    .find(|s| s.sig_type == 1)
                    .ok_or_else(|| anyhow::anyhow!("missing usher signature"))?;
                (s.public_key, s.sig)
            };
            let msg = self.to_usher_hash()?;
            let usher_pk = VerifyingKey::from_bytes(&usher_pk_bytes)?;
            let mut usher_key = Key::new();
            usher_key.set_pub_key(usher_pk);
            let usher_dalek = DalekSig::from_bytes(&usher_sig_bytes);
            let ok = usher_key.verify(&msg, &usher_dalek);
            if !ok {
                anyhow::bail!("invalid usher signature");
            }
        }

        let has_quorum = self.signatures.iter().any(|s| s.sig_type == 2);

        if has_quorum {
            let quorum_sigs: Vec<&Signature> =
                self.signatures.iter().filter(|s| s.sig_type == 2).collect();
            let msg = self.to_quorum_hash()?;
            for sig in quorum_sigs {
                let pk = VerifyingKey::from_bytes(&sig.public_key)?;
                let mut quorum_key = Key::new();
                quorum_key.set_pub_key(pk);
                let dalek = DalekSig::from_bytes(&sig.sig);
                let ok = quorum_key.verify(&msg, &dalek);
                if !ok {
                    anyhow::bail!("invalid quorum signature");
                }
            }
        }

        Ok(())
    }

    pub fn to_author_hash(&self) -> anyhow::Result<[u8; 32]> {
        let mut author_bytes = "RSIG/A/1".as_bytes().to_vec();
        author_bytes.extend_from_slice(&Self::to_stable_cbor(&self.intent)?);
        // return the hash of the bytes
        Ok(*blake3::hash(&author_bytes).as_bytes())
    }

    pub fn to_usher_hash(&self) -> anyhow::Result<[u8; 32]> {
        let mut usher_bytes = "RSIG/U/1".as_bytes().to_vec();
        let author_sig = self.signatures.iter().find(|s| s.sig_type == 0);
        if author_sig.is_none() {
            anyhow::bail!("missing author signature");
        }
        let author_sig = author_sig.unwrap();
        usher_bytes.extend_from_slice(&author_sig.sig);
        usher_bytes.extend_from_slice(&self.context.at.to_be_bytes());
        Ok(*blake3::hash(&usher_bytes).as_bytes())
    }

    pub fn to_quorum_hash(&self) -> anyhow::Result<[u8; 32]> {
        let mut quorum_bytes = "RSIG/Q/1".as_bytes().to_vec();
        let author_sig = self.signatures.iter().find(|s| s.sig_type == 0);
        let usher_sig = self.signatures.iter().find(|s| s.sig_type == 1);
        if author_sig.is_none() {
            anyhow::bail!("missing author signature");
        }
        if usher_sig.is_none() {
            anyhow::bail!("missing usher signature");
        }
        let author_sig = author_sig.unwrap();
        let usher_sig = usher_sig.unwrap();
        quorum_bytes.extend_from_slice(&author_sig.sig);
        quorum_bytes.extend_from_slice(&usher_sig.sig);
        Ok(*blake3::hash(&quorum_bytes).as_bytes())
    }
}

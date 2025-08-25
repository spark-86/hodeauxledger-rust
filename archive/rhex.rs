use crate::time::GTClock as GT;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

use std::cmp::Ordering;

/// --------- Core types ---------

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Intent {
    #[serde_as(as = "Bytes")]
    pub previous_hash: [u8; 32],
    pub scope: String,
    pub nonce: String,
    #[serde_as(as = "Bytes")]
    pub author_public_key: [u8; 32],
    #[serde_as(as = "Bytes")]
    pub usher_public_key: [u8; 32],
    pub record_type: String,
    pub data: serde_json::Value,
}

impl Intent {
    /// Builder for a fresh intent (fills nonce and µmark time).
    pub fn new(
        previous_hash: [u8; 32],
        scope: &str,
        nonce: &str,
        author_pk: [u8; 32],
        usher_pk: [u8; 32],
        record_type: &str,
        data: serde_json::Value,
    ) -> Self {
        Self {
            previous_hash,
            scope: scope.to_string(),
            nonce: nonce.to_string(),
            author_public_key: author_pk,
            usher_public_key: usher_pk,
            record_type: record_type.to_string(),
            data,
        }
    }

    /// Canonical bytes used for signing/hashing (canonical CBOR).
    pub fn canonical_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(to_stable_cbor(self)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    pub at: u64,
}

impl Context {
    pub fn new() -> Self {
        // FIXME: This doesn't set epoch so our at is wrong, always
        let clock = GT::new(0);
        Self {
            at: clock.now_micromarks().try_into().unwrap(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Signature {
    /// 0=author, 1=usher, 2..=quorum
    pub sig_type: u8,
    #[serde_as(as = "Bytes")]
    pub public_key: [u8; 32],
    #[serde_as(as = "Bytes")]
    pub sig: [u8; 64],
}

impl Signature {
    pub fn new() -> Self {
        Self {
            sig_type: 0,
            public_key: [0u8; 32],
            sig: [0u8; 64],
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rhex {
    /// e.g., b"RHEX\x01\x00"
    #[serde_as(as = "Bytes")]
    pub magic: [u8; 6],
    pub intent: Intent,
    pub context: Context,
    pub signatures: Vec<Signature>,
    /// Present once finalized.
    #[serde_as(as = "Option<Bytes>")]
    pub current_hash: Option<[u8; 32]>,
}

impl Rhex {
    /// Start a draft Rhex (not yet hashed).
    pub fn draft(intent: Intent, signatures: Vec<Signature>) -> Self {
        Self {
            magic: *b"RHEX\x01\x00",
            intent,
            context: Context { at: 0 },
            signatures,
            current_hash: None,
        }
    }

    /// Deterministic bytes to hash: intent (canonical CBOR) + signatures in order.
    pub fn bytes_for_hash(&self) -> anyhow::Result<Vec<u8>> {
        let mut out = self.intent.canonical_bytes()?;
        out.extend(to_canonical_cbor(&self.context)?);
        for s in &self.signatures {
            out.extend_from_slice(&s.sig);
        }
        Ok(out)
    }

    /// Compute current_hash and return a finalized record.
    pub fn finalize(mut self) -> anyhow::Result<Self> {
        let hash = Self::compute_current_hash(&mut self);
        self.current_hash = Some(hash);
        Ok(self)
    }

    /// Accessor that errors until finalized (avoids Option unwraps everywhere).
    pub fn current_hash(&self) -> anyhow::Result<[u8; 32]> {
        self.current_hash
            .ok_or_else(|| anyhow::anyhow!("Rhex not finalized: current_hash is None"))
    }

    /// Version from magic.
    pub fn version(&self) -> anyhow::Result<u16> {
        get_version(&self.magic)
    }

    /// Pack full record as canonical CBOR.
    pub fn pack(&self) -> anyhow::Result<Vec<u8>> {
        Ok(to_canonical_cbor(self)?)
    }

    /// Unpack from bytes.
    pub fn unpack(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(from_cbor(bytes)?)
    }

    /// Hash = SHA-256(intent_cbor || at || sig[0].sig || sig[1].sig ...).
    pub fn compute_current_hash(&mut self) -> [u8; 32] {
        // ensure signatures are sorted
        self.sort_signatures();

        let mut h = blake3::Hasher::new();
        h.update(&self.intent.canonical_bytes().unwrap());
        h.update(&self.context.at.to_be_bytes());

        for s in &self.signatures {
            h.update(&s.sig);
        }

        h.finalize().into()
    }

    pub fn sort_signatures(&mut self) {
        self.signatures.sort_by(|a, b| {
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
    }

    pub fn verify_hash(&mut self) -> anyhow::Result<()> {
        let hash = self.compute_current_hash();
        if hash != self.current_hash.unwrap() {
            return Err(anyhow::anyhow!(
                "Current hash mismatch: expected {:?}, got {:?}",
                self.current_hash,
                hash
            ));
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(from_cbor(bytes)?)
    }
}

/// --------- Canonical CBOR helpers ---------

// FIXME: Um. This doesn't actually canonicalize, I'm just scared to take
// it out now.
pub fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_cbor::Error> {
    // Ensure canonical map ordering. (Requires serde_cbor >= 0.11 with `.canonical()`.)
    let mut buf = Vec::with_capacity(256);
    let mut ser = serde_cbor::ser::Serializer::new(&mut buf);
    // Optional: include self-describe tag; comment out if you want pure canonical without tag
    // ser.self_describe()?;
    value.serialize(&mut ser)?;
    Ok(buf)
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
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

use super::{context::Context, intent::Intent, signature::Signature};
use crate::key::key::Key;

use anyhow::{Result, anyhow, bail};
use blake3::Hasher;
use ed25519_dalek::{Signature as DalekSig, VerifyingKey};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::cmp::Ordering;

const MAGIC_PREFIX: &[u8; 4] = b"RHEX";
const MAGIC_V1: [u8; 6] = *b"RHEX\x00\x00";

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

/* ─────────────────────────────────  Constructors  ───────────────────────────── */

impl Rhex {
    pub fn new() -> Self {
        Self {
            magic: MAGIC_V1,
            intent: Intent::new(&[0u8; 32], "", "", &[0u8; 32], &[0u8; 32], "", "{}".into()),
            context: Context::new(),
            signatures: Vec::new(),
            current_hash: None,
        }
    }

    pub fn draft(intent: Intent) -> Self {
        Self {
            magic: MAGIC_V1,
            intent,
            context: Context { at: 0 },
            signatures: Vec::new(),
            current_hash: None,
        }
    }
}

/* ────────────────────────────────  Domains & Prehash  ───────────────────────── */

impl Rhex {
    pub const DOMAIN_CONTENT: &'static [u8] = b"RHEXv1|CONTENT";
    pub const DOMAIN_RECORD: &'static [u8] = b"RHEXv1|RECORD";
    pub const DOMAIN_USHER: &'static [u8] = b"RSIG/U/1";
    pub const DOMAIN_QUORUM: &'static [u8] = b"RSIG/Q/1";

    /// Author prehash: H("RHEXv1|CONTENT" || c14n(intent))
    pub fn author_prehash(&self) -> Result<[u8; 32]> {
        let mut h = Hasher::new();
        h.update(Self::DOMAIN_CONTENT);
        h.update(&Self::to_stable_cbor(&self.intent)?);
        Ok(h.finalize().into())
    }

    /// Usher prehash: H("RSIG/U/1" || author.sig || context.at_be)
    pub fn usher_prehash(&self, author_sig: &[u8; 64]) -> Result<[u8; 32]> {
        let mut h = Hasher::new();
        h.update(Self::DOMAIN_USHER);
        h.update(author_sig);
        h.update(&self.context.at.to_be_bytes());
        Ok(h.finalize().into())
    }

    /// Quorum prehash: H("RSIG/Q/1" || author.sig || usher.sig_or_zeros)
    pub fn quorum_prehash(
        &self,
        author_sig: &[u8; 64],
        usher_sig: Option<&[u8; 64]>,
    ) -> Result<[u8; 32]> {
        let mut h = Hasher::new();
        h.update(Self::DOMAIN_QUORUM);
        h.update(author_sig);
        match usher_sig {
            Some(u) => h.update(u),
            None => h.update(&[0u8; 64]),
        };
        Ok(h.finalize().into())
    }
}

/* ────────────────────────────────  Hashing model  ───────────────────────────── */

impl Rhex {
    /// Alias for author prehash to keep the old name (if callers expect it).
    pub fn compute_content_hash(&self) -> Result<[u8; 32]> {
        self.author_prehash()
    }

    fn canonical_sigs_bytes(&self) -> Result<Vec<u8>> {
        let mut sigs = self.signatures.clone();
        Self::sort_signatures_in_place(&mut sigs);
        Self::to_stable_cbor(&sigs)
    }

    /// current_hash = H("RHEXv1|RECORD" || content_prehash || context.at_be || canonical(sigs))
    pub fn compute_current_hash(&self) -> Result<[u8; 32]> {
        let content = self.author_prehash()?;
        let sigs = self.canonical_sigs_bytes()?;

        let mut h = Hasher::new();
        h.update(Self::DOMAIN_RECORD);
        h.update(&content);
        h.update(&self.context.at.to_be_bytes());
        h.update(&sigs);
        Ok(h.finalize().into())
    }

    pub fn finalize(mut self) -> Result<Self> {
        self.current_hash = Some(self.compute_current_hash()?);
        Ok(self)
    }

    pub fn current_hash(&self) -> Result<[u8; 32]> {
        self.current_hash
            .ok_or_else(|| anyhow!("Rhex not finalized: current_hash is None"))
    }
}

/* ───────────────────────────────  Validation & Sig  ─────────────────────────── */

impl Rhex {
    pub fn validate(&self) -> Result<()> {
        if self.magic.len() != 6 || &self.magic[0..4] != MAGIC_PREFIX {
            bail!("invalid magic");
        }

        if let Some(ch) = self.current_hash {
            let recomputed = self.compute_current_hash()?;
            if recomputed != ch {
                bail!("current_hash mismatch");
            }
        }

        // Author (required) over author_prehash
        let author = self
            .signatures
            .iter()
            .find(|s| s.sig_type == 0)
            .ok_or_else(|| anyhow!("missing author signature"))?;
        let author_pre = self.author_prehash()?;
        Self::verify_one(author, &author_pre, "author")?;

        // Usher (optional) over usher_prehash(author.sig)
        let usher_opt = self.signatures.iter().find(|s| s.sig_type == 1);
        if let Some(usher) = usher_opt {
            let pre = self.usher_prehash(&author.sig)?;
            Self::verify_one(usher, &pre, "usher")?;
        }

        // Quorum (optional) over quorum_prehash(author.sig, usher.sig)
        let quorum: Vec<&Signature> = self.signatures.iter().filter(|s| s.sig_type == 2).collect();
        if !quorum.is_empty() {
            let pre = self.quorum_prehash(&author.sig, usher_opt.map(|u| &u.sig))?;
            for (i, q) in quorum.into_iter().enumerate() {
                Self::verify_one(q, &pre, &format!("quorum[{i}]"))?;
            }
        }

        Ok(())
    }

    fn verify_one(sig: &Signature, msg_hash32: &[u8; 32], label: &str) -> Result<()> {
        let pk = VerifyingKey::from_bytes(&sig.public_key)?;
        let mut k = Key::new();
        k.set_pub_key(pk);
        let dalek = DalekSig::from_bytes(&sig.sig);
        if !k.verify(msg_hash32, &dalek) {
            bail!("invalid {label} signature");
        }
        Ok(())
    }
}

/* ───────────────────────────────  Sig ordering  ─────────────────────────────── */

impl Rhex {
    /// Sorts by (sig_type, public_key bytes, sig bytes). Stable & deterministic.
    fn sort_signatures_in_place(sigs: &mut [Signature]) {
        sigs.sort_by(|a, b| {
            let t = a.sig_type.cmp(&b.sig_type);
            if t != Ordering::Equal {
                return t;
            }

            let pk = a.public_key.as_slice().cmp(b.public_key.as_slice());
            if pk != Ordering::Equal {
                return pk;
            }

            a.sig.as_slice().cmp(b.sig.as_slice())
        });
    }
}

/* ───────────────────────────────  (De)serialization  ────────────────────────── */

impl Rhex {
    pub fn pack(&self) -> Result<Vec<u8>> {
        Self::to_stable_cbor(self)
    }

    pub fn unpack(bytes: &[u8]) -> Result<Self> {
        Self::from_cbor(bytes)
    }

    pub fn to_stable_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>> {
        let v = serde_cbor::value::to_value(value)?;
        Ok(serde_cbor::to_vec(&v)?)
    }

    pub fn from_cbor<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
        Ok(serde_cbor::from_slice(bytes)?)
    }
}

/* ───────────────────────────────────  Utils  ────────────────────────────────── */

impl Rhex {
    pub fn get_version(magic: &[u8]) -> Result<u16> {
        if magic.len() != 6 {
            bail!("magic must be 6 bytes, got {}", magic.len());
        }
        if &magic[0..4] != MAGIC_PREFIX {
            bail!("bad magic prefix");
        }
        let version_bytes: [u8; 2] = magic[4..6]
            .try_into()
            .map_err(|_| anyhow!("bad magic version slice"))?;
        Ok(u16::from_be_bytes(version_bytes))
    }

    pub fn gen_nonce() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }
}

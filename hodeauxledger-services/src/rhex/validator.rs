use ed25519_dalek::{Signature as DalekSig, VerifyingKey};
use hodeauxledger_core::{Key, Rhex};

pub fn validate_rhex(rhex: &Rhex) -> bool {
    true
}

pub fn validate_sigs(rhex: &Rhex) -> bool {
    // must have at least one signature
    if rhex.signatures.is_empty() {
        return false;
    }

    // precompute the three possible message hashes once
    let author_hash = rhex.to_author_hash().ok();
    let usher_hash = rhex.to_usher_hash().ok();
    let quorum_hash = rhex.to_quorum_hash().ok();

    for sig in &rhex.signatures {
        // pick the right message for this signature type
        let msg: &[u8] = match sig.sig_type {
            0 => match author_hash.as_ref() {
                Some(h) => &h[..],
                None => return false,
            },
            1 => match usher_hash.as_ref() {
                Some(h) => &h[..],
                None => return false,
            },
            2 => match quorum_hash.as_ref() {
                Some(h) => &h[..],
                None => return false,
            },
            _ => return false, // unknown sig_type
        };

        // build the verifying key from bytes
        let mut key = Key::new();
        key.set_pub_key(VerifyingKey::from_bytes(&sig.public_key).unwrap());

        // verify this signature; any failure -> whole record invalid
        if key.verify(msg, &DalekSig::from_bytes(&sig.sig)) {
            return false;
        }
    }

    // all signatures verified
    true
}

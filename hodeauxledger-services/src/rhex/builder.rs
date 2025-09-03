use hodeauxledger_core::{Intent, Key, Rhex, Signature};

/// Builds a R⬢ using supplied data and author signs
/// * `scope` - R⬢ scope
/// * `sk` - secret key
/// * `usher_pk` - target usher's public key
/// * `record_type` - R⬢ record type
/// * `data` - JSON data payload
pub fn build_rhex(
    previous_hash: &[u8; 32],
    scope: &str,
    sk: &Key,
    usher_pk: &[u8; 32],
    record_type: &str,
    data: serde_json::Value,
) -> Rhex {
    let nonce = &Rhex::gen_nonce();
    let key = sk;
    let author_pk = key.to_bytes();

    // Build the intent
    let intent = Intent::new(
        previous_hash,
        scope,
        nonce,
        &author_pk,
        usher_pk,
        record_type,
        data,
    );
    let mut rhex = Rhex::draft(intent);

    // Sign the intent
    let author_hash = rhex.compute_content_hash().unwrap();
    let signature = key.sign(&author_hash).unwrap();

    // Push sig on stack
    let author_sig = Signature {
        sig_type: 0,
        public_key: author_pk,
        sig: signature.to_bytes(),
    };
    rhex.signatures.push(author_sig);
    rhex
}

pub fn usher_sign(rhex: &Rhex, at: u64, sk: [u8; 32]) -> Rhex {
    let mut rhex = rhex.clone();
    rhex.context.at = at;
    let key = Key::from_bytes(&sk);
    let usher_pk = key.to_bytes();
    let author_sig = rhex.signatures.iter().find(|s| s.sig_type == 0).unwrap();
    let msg = rhex.usher_prehash(&author_sig.sig);
    if msg.is_err() {
        return rhex;
    }
    let msg = msg.unwrap();
    //let msg = rhex.to_usher_hash().unwrap();
    let signature = key.sign(&msg).unwrap();
    let usher_sig = Signature {
        sig_type: 1,
        public_key: usher_pk,
        sig: signature.to_bytes(),
    };
    rhex.signatures.push(usher_sig);
    rhex
}

pub fn quorum_sign(rhex: &Rhex, sk: [u8; 32]) -> Rhex {
    let mut rhex = rhex.clone();
    let key = Key::from_bytes(&sk);
    let quorum_pk = key.to_bytes();
    let author_sig = rhex.signatures.iter().find(|s| s.sig_type == 0).unwrap();
    let usher_sig = rhex.signatures.iter().find(|s| s.sig_type == 1);
    let msg = rhex.quorum_prehash(&author_sig.sig, Some(&usher_sig.unwrap().sig));
    let signature = key.sign(&msg.unwrap()).unwrap();
    let quorum_sig = Signature {
        sig_type: 2,
        public_key: quorum_pk,
        sig: signature.to_bytes(),
    };
    rhex.signatures.push(quorum_sig);
    rhex
}

use crate::Cli;
use hodeauxledger_core::Key;
use hodeauxledger_core::rhex::intent::Intent;
use hodeauxledger_core::rhex::rhex::Rhex;
use hodeauxledger_core::rhex::signature::Signature;
use hodeauxledger_io::disk::disk;
use std::path::Path;
use std::time::SystemTime;

/// Creates a genesis record. In theory only used once, ever. Why it's
/// part of the tool and not it's own standalone thing I don't know.
pub fn create_genesis(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let save_path = args
        .save
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("save must be specified"))?;
    let keyfile = args
        .keyfile
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("keyfile must be specified"))?;
    let password = args
        .password
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("password must be specified"))?;
    let sk = disk::load_key(Path::new(keyfile), password)?;
    let key = Key::new();
    key.from_bytes(&sk.to_bytes());
    let pk_bytes = key.to_bytes();
    let data = serde_json::json!({
        "schema": "rhex://schema/scope.genesis#1",
        "description": "Trust Architecture Core Scope Genesis",
        "unix_at": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as u64,
    });
    let mut rhex = Rhex::draft(
        Intent::new(
            [0u8; 32],
            "",
            Rhex::gen_nonce().as_str(),
            pk_bytes,
            pk_bytes,
            "scope:genesis",
            data,
        ),
        Vec::new(),
    );

    // Sign as author
    let intent_hash = rhex.to_author_hash()?;
    let author_sig = key.sign(&intent_hash)?;
    rhex.signatures.push(Signature {
        sig_type: 0,
        public_key: pk_bytes,
        sig: author_sig.into(),
    });

    // Sign as usher
    let usher_hash = rhex.to_usher_hash()?;
    let usher_sig = key.sign(&usher_hash)?;
    rhex.signatures.push(Signature {
        sig_type: 1,
        public_key: pk_bytes,
        sig: usher_sig.into(),
    });

    // Sign quorum
    let quorum_hash = rhex.to_quorum_hash()?;
    let quorum_sig = key.sign(&quorum_hash)?;
    rhex.signatures.push(Signature {
        sig_type: 2,
        public_key: pk_bytes,
        sig: quorum_sig.into(),
    });
    let final_rhex = rhex.finalize()?;
    disk::save_rhex(&Path::new(save_path).to_path_buf(), &final_rhex)?;
    Ok(())
}

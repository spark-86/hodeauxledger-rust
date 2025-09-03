use crate::argv::GenesisArgs;
use hodeauxledger_core::Key;
use hodeauxledger_io::disk::key as diskkey;
use hodeauxledger_io::disk::rhex as diskrhex;
use hodeauxledger_services::rhex::builder;
use std::path::Path;
use std::time::SystemTime;

/// Creates a genesis record. In theory only used once, ever. Why it's
/// part of the tool and not it's own standalone thing I don't know.
pub fn create_genesis(args: &GenesisArgs) -> anyhow::Result<(), anyhow::Error> {
    let save_path = &args.output;
    let scope = &args.scope;
    let desc = args.description.clone();
    let keyfile = args.keys.keyfile.as_deref().unwrap_or("");
    let password = args.keys.password.as_deref().unwrap_or("");
    let sk = diskkey::load_key(Path::new(keyfile), password)?;
    let key = Key::from_bytes(&sk.to_bytes());
    let pk_bytes = key.to_bytes();
    let description = if desc.is_some() {
        desc.unwrap()
    } else {
        "Trust Architecture Scope Genesis".to_string()
    };

    let data = if scope.len() > 0 {
        serde_json::json!({
            "schema": "rhex://schema/scope_genesis@0",
            "description": description,
        })
    } else {
        serde_json::json!({
            "schema": "rhex://schema/scope_genesis@0",
            "description": "The HodeauxLedger Root Scope Genesis Record",
            "unix_at": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as u64,
        })
    };
    let mut rhex = builder::build_rhex(
        &[0u8; 32],
        &scope,
        &Key::from_bytes(&sk.to_bytes()),
        &pk_bytes,
        "scope:genesis",
        data,
    );

    rhex = builder::usher_sign(&rhex, 0, sk.to_bytes());
    rhex = builder::quorum_sign(&rhex, sk.to_bytes());

    let final_rhex = rhex.finalize()?;
    diskrhex::save_rhex(&Path::new(save_path).to_path_buf(), &final_rhex)?;
    Ok(())
}

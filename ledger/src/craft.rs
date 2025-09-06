/// Craft an intent entirely from the command line to be signed by
/// the keytool.
use crate::argv::CraftArgs;
use hodeauxledger_core::crypto::b64::from_base64_to_32;
use hodeauxledger_core::rhex::intent::Intent;
use hodeauxledger_core::rhex::rhex::Rhex;
use hodeauxledger_io::disk::disk;
use hodeauxledger_io::disk::rhex as diskrhex;
use std::path::Path;

pub fn craft_intent(args: &CraftArgs) -> anyhow::Result<(), anyhow::Error> {
    let save_path = args
        .save
        .save
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("save must be specified"))?;
    let mut author_pk_b64 = args.author_public_key.clone();
    let mut usher_pk_b64 = args.usher_public_key.clone();
    let scope = &args.scope;
    let data_file = &args.data_file;
    let record_type = &args.record_type;
    let previous_hash_b64 = args.previous_hash.clone().unwrap_or_else(|| "".to_string());
    let nonce = args.nonce.clone().unwrap_or_else(|| Rhex::gen_nonce());
    let data = disk::load_json_data(&data_file)?;
    let ph_bytes = match previous_hash_b64.len() {
        44 | 43 => from_base64_to_32(&previous_hash_b64)?,
        0 => [0u8; 32],
        _ => anyhow::bail!("previous_hash must be 32 bytes or empty"),
    };
    let nonce = nonce.to_string();
    if author_pk_b64.starts_with("\\") {
        author_pk_b64.remove(0);
    }
    if usher_pk_b64.starts_with("\\") {
        usher_pk_b64.remove(0);
    }
    let author_pk = from_base64_to_32(&author_pk_b64)?;
    let usher_pk = from_base64_to_32(&usher_pk_b64)?;
    let intent = Intent::new(
        &ph_bytes,
        &scope,
        &nonce,
        &author_pk,
        &usher_pk,
        &record_type,
        data,
    );

    let rhex = Rhex::draft(intent);

    // output rhex intent
    diskrhex::save_rhex(&Path::new(save_path).to_path_buf(), &rhex)?;
    Ok(())
}

/// Craft an intent entirely from the command line to be signed by
/// the keytool.
use crate::Cli;
use hodeauxledger_core::crypto::b64::from_base64_to_32;
use hodeauxledger_core::rhex::intent::Intent;
use hodeauxledger_core::rhex::rhex::Rhex;
use hodeauxledger_io::disk;
use std::path::Path;

pub fn craft_intent(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let save_path = args
        .save
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("save must be specified"))?;
    let author_pk_b64 = args
        .author_public_key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("author_public_key must be specified"))?;
    let usher_pk_b64 = args
        .usher_public_key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("usher_public_key must be specified"))?;
    let scope = args
        .scope
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("scope must be specified"))?;
    let data_file = args
        .data_file
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("data_file must be specified"))?;
    let record_type = args
        .record_type
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("record_type must be specified"))?;
    let previous_hash_b64 = args
        .previous_hash
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("previous_hash must be specified"))?;
    let nonce = args.nonce.clone().unwrap_or_else(|| Rhex::gen_nonce());
    let data = disk::load_json_data(data_file)?;
    let ph_bytes = match previous_hash_b64.len() {
        32 => from_base64_to_32(previous_hash_b64)?,
        0 => [0u8; 32],
        _ => anyhow::bail!("previous_hash must be 32 bytes or empty"),
    };
    let nonce = nonce.to_string();
    let author_pk = from_base64_to_32(author_pk_b64)?;
    let usher_pk = from_base64_to_32(usher_pk_b64)?;
    let intent = Intent::new(
        ph_bytes,
        &scope,
        &nonce,
        author_pk,
        usher_pk,
        record_type,
        data,
    );

    let rhex = Rhex::draft(intent, Vec::new());
    // output rhex intent
    //disk::save_intent(save_path, &rhex.intent)?;
    disk::save_rhex(&Path::new(save_path).to_path_buf(), &rhex)?;
    Ok(())
}

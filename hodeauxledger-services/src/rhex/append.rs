use std::path::Path;

use crate::scope;
use ed25519_dalek::VerifyingKey;
use hodeauxledger_core::{Key, Rhex};
use hodeauxledger_io::disk;

pub fn append_rhex(path: &str, rhex: &Rhex) -> Result<(), anyhow::Error> {
    // Check policy to make sure we can append
    let author_vk = VerifyingKey::from_bytes(&rhex.intent.author_public_key)?;
    let mut key = Key::new();
    key.pk = Some(author_vk);

    if scope::append::can_append(&rhex.intent.scope, &key) {
        let path_buf = Path::new(path).to_path_buf();
        let _ = disk::rhex::save_rhex(&path_buf, rhex);
    } else {
        Err(anyhow::bail!("cannot append to scope"))?
    }
    Ok(())
}

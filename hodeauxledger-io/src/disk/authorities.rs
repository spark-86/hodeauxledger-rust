use std::fs;

use hodeauxledger_core::scope::authority::Authority;

pub fn get_root_authorities_from_disk() -> Result<Vec<Authority>, anyhow::Error> {
    let root_auth_json = fs::read_to_string("./data/root_authorities.json");
    if root_auth_json.is_err() {
        return Err(anyhow::anyhow!("root authorities not found"));
    }
    let root_auth_json = root_auth_json.unwrap();
    let root_auth: Vec<Authority> = serde_json::from_str(&root_auth_json)?;
    Ok(root_auth)
}

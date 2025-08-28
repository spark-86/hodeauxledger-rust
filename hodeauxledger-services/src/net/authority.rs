use crate::scope::scope::get_scope_table;
use hodeauxledger_core::scope::authority::Authority;

pub fn get_scope_authorities(scope: &str) -> Result<Vec<Authority>, anyhow::Error> {
    // First we check our scope table to see if we handle the scope
    // in question. If so, we should be an authority on it
    Ok(Vec::new())
}

pub fn get_auth_from_cache(scope: &str) -> Result<Vec<Authority>, anyhow::Error> {
    Ok(Vec::new())
}

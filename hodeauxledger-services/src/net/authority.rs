use hodeauxledger_core::scope::{self, authority::Authority};
use hodeauxledger_io::disk::disk;

pub fn get_scope_authorities(scope: &str) -> Result<Vec<Authority>, anyhow::Error> {
    // First we check our cache and see if we have a list of authorities
    // for this scope.
    let authorities = get_auth_from_cache(scope)?;
    if authorities.len() > 0 {
        return Ok(authorities);
    }
    Ok(Vec::new())
}

pub fn get_auth_from_cache(scope: &str) -> Result<Vec<Authority>, anyhow::Error> {
    let _ = scope;
    Ok(Vec::new())
}

fn find_next_step(scope_parts: Vec<&str>) -> Result<String, anyhow::Error> {
    Ok(String::new())
}

pub fn get_auth_from_net(scope: &str) -> Result<Vec<Authority>, anyhow::Error> {
    let auth_table = disk::load_root_auth("./root_auth.json")?;
    if scope.len() == 0 {
        return Ok(auth_table);
    }
    Ok(Vec::new())
}

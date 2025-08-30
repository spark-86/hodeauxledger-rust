use hodeauxledger_core::scope::authority::Authority;

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

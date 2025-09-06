use hodeauxledger_core::{Key, scope::authority::Authority};
use hodeauxledger_io::Cache;
use hodeauxledger_io::cache;
use hodeauxledger_io::disk;

pub fn get_authorities(scope: &str, key: &Key) -> Result<Vec<Authority>, anyhow::Error> {
    let cache = Cache::connect("")?;
    let authorities = cache::authorities::retrieve_authorities(&cache.conn, scope)?;
    if authorities.len() > 0 {
        return Ok(authorities);
    }
    // Ok, we had zero authorties in the scope, so let's go find the
    // scope and get the authorities
    let scope_parts = &scope.split(".");
    let mut scope_parts = scope_parts.clone().collect::<Vec<&str>>();
    if scope_parts.len() > 1 {
        scope_parts.pop();
        let scope = scope_parts.join(".");
        let authorities = get_authorities(&scope, key)?;
        return Ok(authorities);
    }
    if scope_parts.len() == 1 {
        let authorities = get_authorities(&scope_parts[0], key)?;
        if authorities.len() > 0 {
            return Ok(authorities);
        } else {
            let authorities = get_root_authorities()?;
            return Ok(authorities);
        }
    };
    Ok(Vec::new())
}

pub fn get_root_authorities() -> Result<Vec<Authority>, anyhow::Error> {
    let authorities = disk::authorities::get_root_authorities_from_disk()?;
    Ok(authorities)
}

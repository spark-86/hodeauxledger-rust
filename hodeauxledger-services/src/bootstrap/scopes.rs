use crate::scope::scope::{get_scope_table, scope_from_disk_to_cache};

pub fn populate_scope_cache_from_disk() -> Result<(), anyhow::Error> {
    let scope_table = get_scope_table();
    for scope in scope_table.scopes {
        scope_from_disk_to_cache(scope.name.as_str())?;
    }
    Ok(())
}

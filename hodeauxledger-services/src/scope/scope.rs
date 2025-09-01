use anyhow::Result;
use hodeauxledger_core::scope::{
    authority::{self, Authority},
    scope::Scope,
    table::ScopeTable,
};
use hodeauxledger_io::disk::disk;
use hodeauxledger_io::disk::scope as diskscope;

pub fn sync_scope(scope_name: &str, starting_head: &[u8; 32]) -> Result<(), anyhow::Error> {
    let scope_table = get_scope_table();

    // prefer pattern matching, avoid unwraps
    let root: Vec<Authority> = if let Some(scope) = scope_table.lookup(scope_name) {
        let k = authority::byzantine_quorum_k(scope.authorities.len());
        // pick returns Vec<&Authority>; clone them into owned Vec<Authority>
        authority::pick_k_weighted_unique(&scope.authorities, k)
            .into_iter()
            .cloned()
            .collect()
    } else if let Some(root_scope) = scope_table.lookup("") {
        let k = authority::byzantine_quorum_k(root_scope.authorities.len());
        authority::pick_k_weighted_unique(&root_scope.authorities, k)
            .into_iter()
            .cloned()
            .collect()
    } else {
        // first install — load from bootstrap
        disk::load_root_auth("bootstrap.json")?
    };

    //

    // TODO: use `root` (connect, sync, etc.)
    let _ = starting_head; // silence for now if unused
    let _ = root; // silence for now if unused
    Ok(())
}

pub fn get_scope_table() -> ScopeTable {
    ScopeTable::from_json(diskscope::load_scope_table("scope_table.json").unwrap()).unwrap()
}

pub fn save_scope_table(st: &ScopeTable) -> Result<(), anyhow::Error> {
    diskscope::save_scope_table("/ledger", st)?;
    Ok(())
}

pub fn add_scope_to_table(scope: &Scope) -> Result<(), anyhow::Error> {
    let mut st = get_scope_table();
    st.scopes.push(scope.clone());
    save_scope_table(&st)?;
    Ok(())
}

pub fn remove_scope_from_table(scope_name: &str) -> Result<(), anyhow::Error> {
    let mut st = get_scope_table();
    st.remove_scope(scope_name);
    save_scope_table(&st)?;
    Ok(())
}

pub fn scope_from_disk_to_cache(scope_name: &str) -> Result<(), anyhow::Error> {
    let scope_table = get_scope_table();
    let scope = scope_table.lookup(scope_name);
    if scope.is_none() {
        return Err(anyhow::anyhow!("scope not found"));
    }
    diskscope::load_scope("./data/ledger", scope_name, diskscope::ScopeSink::Db)?;

    Ok(())
}

pub fn bootstrap_rhex_cache() -> Result<(), anyhow::Error> {
    let scope_table = get_scope_table();
    for scope in scope_table.scopes {
        scope_from_disk_to_cache(scope.name.as_str())?;
    }
    Ok(())
}

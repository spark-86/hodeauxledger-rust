use hodeauxledger_core::scope::{scope::Scope, table::ScopeTable};
use hodeauxledger_io::disk;

pub fn sync_scope(scope_name: &str, starting_head: &[u8; 32]) {
    let scope_table = get_scope_table();
}

pub fn get_scope_table() -> ScopeTable {
    ScopeTable::from_json(disk::load_scope_table("scope_table.json").unwrap()).unwrap()
}

pub fn save_scope_table(st: &ScopeTable) -> Result<(), anyhow::Error> {
    disk::save_scope_table("/ledger", st)?;
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

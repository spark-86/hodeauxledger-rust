use anyhow::Result;
use hodeauxledger_core::{
    Rhex,
    scope::{
        self,
        authority::{self, Authority},
        scope::Scope,
        table::ScopeTable,
    },
};
use hodeauxledger_io::disk::disk;
use hodeauxledger_io::disk::scope as diskscope;

pub fn get_scope_table() -> Result<ScopeTable, anyhow::Error> {
    // Load the scope table to see which scopes we need to take care
    // of.
    println!("Loading 🌐 table...");
    let scope_table_raw = diskscope::load_scope_table("./data/ledger")?;
    let scope_table: Vec<Scope> = serde_json::from_str(&scope_table_raw)?;
    let mut out = Vec::new();
    for scope in scope_table {
        out.push(scope);
    }
    let out = ScopeTable::new(out);

    Ok(out)
}

pub fn save_scope_table(st: &ScopeTable) -> Result<(), anyhow::Error> {
    diskscope::save_scope_table("/ledger", st)?;
    Ok(())
}

pub fn add_scope_to_table(scope: &Scope) -> Result<(), anyhow::Error> {
    let mut st = get_scope_table()?;
    st.scopes.push(scope.clone());
    save_scope_table(&st)?;
    Ok(())
}

pub fn remove_scope_from_table(scope_name: &str) -> Result<(), anyhow::Error> {
    let mut st = get_scope_table()?;
    st.remove_scope(scope_name);
    save_scope_table(&st)?;
    Ok(())
}

pub fn scope_from_disk_to_cache(
    scope_name: &str,
    process_rhex: bool,
) -> Result<Vec<Rhex>, anyhow::Error> {
    let scope_table = get_scope_table()?;
    let scope = scope_table.lookup(scope_name);
    if scope.is_none() {
        return Err(anyhow::anyhow!("scope not found"));
    }
    let scope_data =
        diskscope::load_scope("./data/ledger", scope_name, diskscope::ScopeSink::Both)?;
    let mut output = Vec::new();
    if process_rhex {
        for rhex in scope_data {
            output.extend(crate::rhex::process::process_rhex(&rhex, false));
        }
    }

    Ok(output)
}

pub fn bootstrap_rhex_cache() -> Result<(), anyhow::Error> {
    let scope_table = get_scope_table()?;
    for scope in scope_table.scopes {
        scope_from_disk_to_cache(scope.name.as_str(), true)?;
    }
    Ok(())
}

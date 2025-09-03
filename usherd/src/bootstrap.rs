use anyhow::Ok;
use hodeauxledger_core::scope::scope::Scope;
use hodeauxledger_io::disk::scope as diskscope;

fn get_scope_list(ledger_path: &str) -> Result<Vec<Scope>, anyhow::Error> {
    let scope_table = diskscope::load_scope_table(ledger_path)?;

    let arr = scope_table
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("scope_table root is not an array"))?;

    // Now arr is &Vec<Value>. You can map/deserialize each Value into Scope:
    let scopes: Vec<Scope> = arr
        .iter()
        .map(|v| serde_json::from_value(v.clone()))
        .collect::<Result<_, _>>()?;

    Ok(scopes)
}

pub fn bootstrap(verbose: bool) -> anyhow::Result<()> {
    // Load the root authorities into the cache
    if verbose {
        println!("Loading root authorities...");
    }

    // Load the scope table to see which scopes we need to take care
    // of.
    if verbose {
        println!("Loading 🌐 table...");
    }
    let scope_list = get_scope_list("./ledger/")?;

    // Verify that our status is current in the list of scopes
    let _ = scope_list;

    Ok(())
}

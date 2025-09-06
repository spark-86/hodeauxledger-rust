use anyhow::Ok;
use hodeauxledger_core::scope::scope::Scope;
use hodeauxledger_io::{cache::cache::Cache, disk::scope as diskscope, screen::pretty_print_rhex};
use hodeauxledger_services::scope::scope::scope_from_disk_to_cache;

fn get_scope_list(ledger_path: &str) -> Result<Vec<Scope>, anyhow::Error> {
    let scope_table = diskscope::load_scope_table(ledger_path)?;

    let scopes: Vec<Scope> = serde_json::from_str(&scope_table)?;

    Ok(scopes)
}

pub fn bootstrap(verbose: bool) -> anyhow::Result<()> {
    // Clear cache
    if verbose {
        println!("Clearing cache...");
    }
    let cache = Cache::connect("")?;

    cache.flush_everything()?;

    // Load the root authorities into the cache
    if verbose {
        println!("Loading root authorities...");
    }

    // Load the scope table to see which scopes we need to take care
    // of.
    if verbose {
        println!("Loading 🌐 table...");
    }
    let scope_list = get_scope_list("./data/ledger")?;

    // Verify that our status is current in the list of scopes

    for scope in scope_list {
        let scope_name = scope.name.as_str();
        let output = scope_from_disk_to_cache(scope_name, true)?;
        for rhex in output {
            let _ = pretty_print_rhex(&rhex);
        }
    }

    Ok(())
}

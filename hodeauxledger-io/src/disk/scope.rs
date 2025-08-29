use crate::disk::rhex::load_rhex;
use anyhow::{Result, anyhow, bail};
use hodeauxledger_core::{rhex::rhex::Rhex, scope::table::ScopeTable};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn load_scope(dir: &str) -> Result<Vec<Rhex>> {
    let base = Path::new(dir);
    let mut out = Vec::new();

    // 1) Load scope:genesis first
    let genesis_path = base.join("genesis.rhex");
    if !genesis_path.exists() {
        bail!("Missing genesis file: {}", genesis_path.display());
    }

    let curr = load_rhex(&genesis_path)?;
    let mut working_hash = curr
        .current_hash
        .ok_or_else(|| anyhow!("genesis has no current_hash"))?;

    out.push(curr);

    // 2) Walk forward: find the one file whose previous_hash == working_hash
    loop {
        // Try to find a child file in this directory.
        let mut found_child: Option<(PathBuf, Rhex)> = None;

        for entry in fs::read_dir(base)? {
            let entry = entry?;
            let p = entry.path();

            // skip non-files and genesis
            if !p.is_file() {
                continue;
            }
            if p.file_name().and_then(|s| s.to_str()) == Some("genesis.rhex") {
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("rhex") {
                continue;
            }

            // Load candidate and check previous_hash
            let candidate = match load_rhex(&p) {
                Ok(x) => x,
                Err(_) => continue, // skip unreadables
            };

            // Only advance if this file says its previous_hash == current working_hash
            if candidate.intent.previous_hash == working_hash {
                found_child = Some((p.clone(), candidate));
                break;
            }
        }

        match found_child {
            Some((_path, child)) => {
                working_hash = child
                    .current_hash
                    .ok_or_else(|| anyhow!("child record missing current_hash"))?;
                out.push(child);
            }
            None => {
                // No child found → we've hit the head for this scope.
                break;
            }
        }
    }

    Ok(out)
}

/// Loads the scope table from ledger_path/scope_table.json
/// * `ledger_path` - str path to the ledger directory
pub fn load_scope_table(ledger_path: &str) -> Result<serde_json::Value> {
    if ledger_path.is_empty() {
        anyhow::bail!("empty path");
    }
    let mut filename = PathBuf::from(ledger_path);
    filename.push("scope_table.json");
    let table = fs::read_to_string(&filename)?;
    let output = serde_json::from_str(&table)?;
    Ok(output)
}

pub fn save_scope_table(ledger_path: &str, table: &ScopeTable) -> Result<()> {
    if ledger_path.is_empty() {
        anyhow::bail!("empty path");
    }
    let mut filename = PathBuf::from(ledger_path);
    filename.push("scope_table.json");
    let table_str = table.to_string();
    fs::write(&filename, table_str)?;
    Ok(())
}

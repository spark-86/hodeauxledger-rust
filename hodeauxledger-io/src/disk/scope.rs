use crate::cache::{cache::Cache, rhex::cache_rhex};
use crate::disk::rhex::load_rhex;
use anyhow::{Result, bail};
use hodeauxledger_core::{rhex::rhex::Rhex, scope::table::ScopeTable};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub enum ScopeSink {
    Vec,
    Db,
}

pub fn load_scope(ledger_path: &str, scope: &str, sink: ScopeSink) -> Result<Vec<Rhex>> {
    let dir = format!("{}/{}", ledger_path, scope);

    let base = Path::new(&dir);
    let mut out = Vec::new();
    let cache = Cache::connect("cache.db")?;

    // 1) Load scope:genesis first
    let genesis_path =
        base.join("0000000000000000000000000000000000000000000000000000000000000000.rhex");
    if !genesis_path.exists() {
        bail!("Missing genesis file: {}", genesis_path.display());
    }
    let curr = load_rhex(&genesis_path)?;

    let mut working_hash = if curr.current_hash.is_some() {
        curr.current_hash.unwrap()
    } else {
        bail!("genesis has no ⬇️🧬");
    };
    match sink {
        ScopeSink::Vec => {
            out.push(curr);
        }
        ScopeSink::Db => {
            cache_rhex(&cache.conn, &curr)?;
        }
    }

    loop {
        // Try to find the next file in the chain
        let new_file = base.join(format!("{}.rhex", to_hex(&working_hash)));
        if !new_file.exists() {
            break;
        }
        let candidate = load_rhex(&new_file)?;
        working_hash = if candidate.current_hash.is_some() {
            candidate.current_hash.unwrap()
        } else {
            bail!("child record missing ⬇️🧬");
        };
        match sink {
            ScopeSink::Vec => {
                out.push(candidate);
            }
            ScopeSink::Db => {
                cache_rhex(&cache.conn, &candidate)?;
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

pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use std::fmt::Write;
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

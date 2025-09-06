use hodeauxledger_core::policy::{default::Default, policy::Policy};

use rusqlite::{Connection, params}; // your Default struct

pub fn cache_policy(
    conn: &Connection,
    scope: &str,
    policy: &Policy,
    current_hash: &[u8; 32],
) -> Result<(), anyhow::Error> {
    // clone Options from &Policy safely
    let note = policy.note.clone().unwrap_or_default();

    let defaults = policy.defaults.clone().unwrap_or(Default {
        rate_per_mark: 80,
        quorum_k: 1,
        roles: vec!["👑".to_string()],
        quorum_roles: vec!["👑".to_string()],
    });

    // DELETE must be parameterized with the scope
    conn.execute("DELETE FROM policies WHERE scope = ?1", params![scope])?;

    // Prepare values with proper types
    let default_roles = defaults.roles.join(",");
    let default_quorum_roles = defaults.quorum_roles.join(",");
    let default_rate = defaults.rate_per_mark as i64;
    let default_quorum_k = defaults.quorum_k as i64;

    let quorum_ttl = policy.quorum_ttl.unwrap_or(0) as i64;
    let effective_micromarks = policy.effective_micromark.unwrap_or(0) as i64;
    let expires_micromarks = policy.expiration_micromark.unwrap_or(0) as i64;

    conn.execute(
        "INSERT INTO policies (
            scope, note, default_rate, default_roles, default_quorum_k, default_quorum_roles,
            quorum_ttl, effective_micromarks, expires_micromarks, current_hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            scope,
            note,
            default_rate,
            default_roles,
            default_quorum_k,
            default_quorum_roles,
            quorum_ttl,
            effective_micromarks,
            expires_micromarks,
            current_hash,
        ],
    )?;

    Ok(())
}

pub fn build_table(conn: &Connection) -> Result<(), anyhow::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS policies (
            scope TEXT,
            note TEXT,
            default_rate INTEGER,
            default_roles TEXT,
            default_quorum_k INTEGER,
            default_quorum_roles TEXT,
            quorum_ttl INTEGER,
            effective_micromarks INTEGER,
            expires_micromarks INTEGER,
            current_hash BLOB,
            PRIMARY KEY (current_hash)
        )",
        [],
    )?;
    Ok(())
}

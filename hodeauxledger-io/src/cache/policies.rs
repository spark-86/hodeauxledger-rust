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

pub fn retrieve_policy(conn: &Connection, scope: &str) -> Result<Policy, anyhow::Error> {
    let mut stmt = conn.prepare(
        "SELECT note, default_rate, default_roles, default_quorum_k, default_quorum_roles,
                quorum_ttl, effective_micromarks, expires_micromarks, current_hash
         FROM policies
         WHERE scope = ?1",
    )?;
    let mut rows = stmt.query(params![scope])?;
    let mut policy = Policy {
        note: None,
        defaults: None,
        quorum_ttl: None,
        effective_micromark: None,
        expiration_micromark: None,
        scope: scope.to_string(),
        rules: Vec::new(),
    };
    if let Some(row) = rows.next()? {
        policy.note = row.get::<_, String>("note").ok();
        policy.defaults = Some(Default {
            rate_per_mark: row.get::<_, i64>("default_rate")? as u64,
            roles: row
                .get::<_, String>("default_roles")?
                .split(",")
                .map(|s| s.to_string())
                .collect(),
            quorum_k: row.get::<_, i64>("default_quorum_k")? as u8,
            quorum_roles: row
                .get::<_, String>("default_quorum_roles")?
                .split(",")
                .map(|s| s.to_string())
                .collect(),
        });
        policy.quorum_ttl = row.get::<_, i64>("quorum_ttl")?.try_into().ok();
        policy.effective_micromark = row.get::<_, i64>("effective_micromarks")?.try_into().ok();
        policy.expiration_micromark = row.get::<_, i64>("expires_micromarks")?.try_into().ok();
        // current_hash is not part of the Policy struct, but it's in the table
        // let current_hash: Vec<u8> = row.get("current_hash")?;
    }
    Ok(policy)
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

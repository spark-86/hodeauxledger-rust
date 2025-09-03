use hodeauxledger_core::Key;
use rusqlite::{Connection, params};

pub fn cache_key(conn: &Connection, scope: &str, key: &Key) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("INSERT INTO keys (scope, roles, pk, effective_micromark, expires_micromark) VALUES (?1, ?2, ?3, ?4, ?5)")?;
    stmt.execute(params![
        scope,
        key.roles.as_ref().unwrap().join(","),
        key.pk.unwrap().to_bytes(),
        key.effective_micromark,
        key.expires_micromark
    ])?;
    Ok(())
}

pub fn build_table(conn: &Connection) -> anyhow::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS keys (
            scope TEXT,
            roles TEXT,
            pk BLOB,
            effective_micromark INTEGER,
            expires_micromark INTEGER,
            PRIMARY KEY (scope, public_key)
        )",
        [],
    )?;
    Ok(())
}

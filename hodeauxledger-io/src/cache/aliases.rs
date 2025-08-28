use hodeauxledger_core::Alias;
use rusqlite::{Connection, params};

pub fn cache_alias(conn: &Connection, alias: &Alias) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("INSERT INTO aliases (name, scope, hash) VALUES (?1, ?2, ?3)")?;
    stmt.execute(params![alias.name, alias.scope, alias.hash])?;
    Ok(())
}

pub fn retrieve_alias(conn: &Connection, name: &str) -> anyhow::Result<Alias> {
    let mut stmt = conn.prepare("SELECT scope, hash FROM aliases WHERE name = ?1")?;
    let mut rows = stmt.query(params![name])?;
    let mut alias = Alias {
        name: String::new(),
        scope: String::new(),
        hash: [0u8; 32],
    };
    if let Some(row) = rows.next()? {
        alias.name = name.to_string();
        alias.scope = row.get("scope")?;
        alias.hash = row.get("hash")?;
    }
    Ok(alias)
}

pub fn evict_alias(conn: &Connection, name: &str, scope: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("DELETE FROM aliases WHERE name = ?1 AND scope = ?2")?;
    stmt.execute(params![name, scope])?;
    Ok(())
}

pub fn flush_aliases(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("DELETE FROM aliases")?;
    stmt.execute([])?;
    Ok(())
}

pub fn build_table(conn: &Connection) -> anyhow::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS aliases (
            name TEXT,
            scope TEXT,
            hash BLOB,
            PRIMARY KEY (name, scope)
        )",
        [],
    )?;
    Ok(())
}

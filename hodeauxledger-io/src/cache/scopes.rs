use rusqlite::{Connection, params};

pub fn cache_scope(
    conn: &Connection,
    scope: &str,
    role: &str,
    last_synced: &u64,
    head: &[u8; 32],
) -> Result<(), anyhow::Error> {
    let mut stmt = conn
        .prepare("INSERT INTO scopes (scope, role, last_synced, head) VALUES (?1, ?2, ?3, ?4)")?;
    stmt.execute(params![scope, role, last_synced, head])?;
    Ok(())
}

pub fn retrieve_scope(
    conn: &Connection,
    scope: &str,
) -> Result<(String, u64, [u8; 32]), anyhow::Error> {
    let mut stmt = conn.prepare(
        "SELECT role, last_synced, head
         FROM scopes
         WHERE scope = ?1 LIMIT 1",
    )?;
    let mut rows = stmt.query(params![scope])?;
    let mut role = String::new();
    let mut last_synced: u64 = 0;
    let mut head = [0u8; 32];
    if let Some(row) = rows.next()? {
        role = row.get("role")?;
        last_synced = row.get("last_synced")?;
        head = row.get("head")?;
    }
    Ok((role, last_synced, head))
}

pub fn evict_scope(conn: &Connection, scope: &str) -> Result<(), anyhow::Error> {
    let mut stmt = conn.prepare("DELETE FROM scopes WHERE scope = ?1")?;
    stmt.execute(params![scope])?;
    Ok(())
}

pub fn flush_scopes(conn: &Connection) -> Result<(), anyhow::Error> {
    let mut stmt = conn.prepare("DELETE FROM scopes")?;
    stmt.execute([])?;
    Ok(())
}

pub fn build_table(conn: &Connection) -> Result<(), anyhow::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scopes (
            scope TEXT PRIMARY KEY,
            role TEXT,
            last_synced INTEGER,
            head BLOB
        )",
        [],
    )?;
    Ok(())
}

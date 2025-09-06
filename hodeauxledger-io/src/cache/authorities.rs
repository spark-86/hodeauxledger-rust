use hodeauxledger_core::scope::authority::Authority;
use rusqlite::{Connection, params};

pub fn retrieve_authorities(
    conn: &Connection,
    scope: &str,
) -> Result<Vec<Authority>, anyhow::Error> {
    let mut stmt = conn
        .prepare("SELECT name,host,port,public_key,priority FROM authorities WHERE scope = ?1")?;
    let mut rows = stmt.query(params![scope])?;
    let mut authorities = Vec::new();
    while let Some(row) = rows.next()? {
        let name: String = row.get("name")?;
        let host: String = row.get("host")?;
        let port: u16 = row.get("port")?;
        let proto: String = row.get("proto")?;
        let public_key = row.get("public_key")?;
        let priority: u8 = row.get("priority")?;
        let authority = Authority {
            name,
            host,
            port,
            proto,
            public_key,
            priority,
        };
        authorities.push(authority);
    }

    Ok(Vec::new())
}

pub fn cache_authority(
    conn: &Connection,
    scope: &str,
    authorities: &[Authority],
) -> Result<(), anyhow::Error> {
    let mut stmt = conn.prepare(
        "INSERT INTO authorities (scope, name, host, port, proto, public_key, priority) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for authority in authorities {
        stmt.execute(params![
            scope,
            authority.name,
            authority.host,
            authority.port,
            authority.proto,
            authority.public_key,
            authority.priority
        ])?;
    }
    Ok(())
}

pub fn evict_authority(conn: &Connection, scope: &str, pk: &[u8; 32]) -> Result<(), anyhow::Error> {
    let mut stmt = conn.prepare("DELETE FROM authorities WHERE scope = ?1 AND public_key = ?2")?;
    stmt.execute(params![scope, pk])?;
    Ok(())
}

pub fn flush_authorities(conn: &Connection, scope: &str) -> Result<(), anyhow::Error> {
    let mut stmt = conn.prepare("DELETE FROM authorities WHERE scope = ?1")?;
    stmt.execute(params![scope])?;
    Ok(())
}

pub fn build_table(conn: &Connection) -> Result<(), anyhow::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS authorities (
            scope TEXT,
            name TEXT,
            host TEXT,
            port INTEGER,
            proto TEXT,
            public_key BLOB,
            priority INTEGER,
            PRIMARY KEY (scope, public_key)
        )",
        [],
    )?;
    Ok(())
}

use hodeauxledger_core::Rhex;
use rusqlite::{Connection, params};
use serde_json::Value;
use std::convert::TryInto;

pub fn cache_rhex(conn: &Connection, rhex: &Rhex) -> anyhow::Result<()> {
    let sig_string = serde_json::to_string(&rhex.signatures)?;
    let data_string = serde_json::to_string(&rhex.intent.data)?;

    let mut stmt = conn.prepare("INSERT INTO rhex (previous_hash, scope, nonce, at, author_public_key, usher_public_key, record_type, data, signatures, current_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)")?;
    stmt.execute(params![
        rhex.intent.previous_hash,
        rhex.intent.scope,
        rhex.intent.nonce,
        rhex.context.at,
        rhex.intent.author_public_key,
        rhex.intent.usher_public_key,
        rhex.intent.record_type,
        data_string,
        sig_string,
        rhex.current_hash
    ])?;
    Ok(())
}

pub fn retrieve_rhex(conn: &Connection, current_hash: &[u8; 32]) -> anyhow::Result<Rhex> {
    let mut stmt = conn.prepare(
        "SELECT previous_hash, scope, nonce, at,
                author_public_key, usher_public_key,
                record_type, data, signatures, current_hash
         FROM rhex
         WHERE current_hash = ?1",
    )?;

    let mut rows = stmt.query(params![current_hash])?;

    let mut out_rhex = Rhex {
        magic: *b"RHEX\x00\x00",
        intent: hodeauxledger_core::rhex::intent::Intent {
            previous_hash: [0u8; 32],
            scope: String::new(),
            nonce: String::new(),
            author_public_key: [0u8; 32],
            usher_public_key: [0u8; 32],
            record_type: String::new(),
            data: Value::Null,
        },
        context: hodeauxledger_core::rhex::context::Context { at: 0 },
        signatures: Vec::new(),
        current_hash: None,
    };

    if let Some(row) = rows.next()? {
        // Simple string/primitive gets
        out_rhex.intent.previous_hash = row.get::<_, [u8; 32]>("previous_hash")?;
        out_rhex.intent.scope = row.get::<_, String>("scope")?;
        out_rhex.intent.nonce = row.get::<_, String>("nonce")?;
        let at: i64 = row.get("at")?;
        out_rhex.context.at = at as u64;
        out_rhex.intent.record_type = row.get::<_, String>("record_type")?;

        // Fixed-length binary blobs
        let author_pk: Vec<u8> = row.get("author_public_key")?;
        out_rhex.intent.author_public_key = author_pk
            .try_into()
            .map_err(|_| anyhow::anyhow!("author_public_key not 32 bytes"))?;

        let usher_pk: Vec<u8> = row.get("usher_public_key")?;
        out_rhex.intent.usher_public_key = usher_pk
            .try_into()
            .map_err(|_| anyhow::anyhow!("usher_public_key not 32 bytes"))?;

        // JSON blobs
        let data_str: String = row.get("data")?;
        out_rhex.intent.data = serde_json::from_str(&data_str)?;

        let sig_str: String = row.get("signatures")?;
        out_rhex.signatures = serde_json::from_str(&sig_str)?;

        // current hash
        let curr: Vec<u8> = row.get("current_hash")?;
        out_rhex.current_hash = Some(
            curr.try_into()
                .map_err(|_| anyhow::anyhow!("current_hash not 32 bytes"))?,
        );
    }

    Ok(out_rhex)
}

pub fn evict_rhex(conn: &Connection, current_hash: &[u8; 32]) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("DELETE FROM rhex WHERE current_hash = ?1")?;
    stmt.execute(params![current_hash])?;
    Ok(())
}

pub fn flush_rhex(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("DELETE FROM rhex")?;
    stmt.execute([])?;
    Ok(())
}

pub fn build_table(conn: &Connection) -> anyhow::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rhex (
            previous_hash BLOB,
            scope TEXT,
            nonce TEXT,
            at INTEGER,
            author_public_key BLOB,
            usher_public_key BLOB,
            record_type TEXT,
            data TEXT,
            signatures TEXT,
            current_hash BLOB PRIMARY KEY
        )",
        [],
    )?;
    Ok(())
}

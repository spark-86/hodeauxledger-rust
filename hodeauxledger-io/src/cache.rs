use rusqlite::{Connection, params};

use serde_json::Value;
use std::convert::TryInto;

use hodeauxledger_core::Rhex;

#[derive(Debug)]
pub struct Cache {
    pub conn: Connection,
}

impl Cache {
    pub fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        Self { conn }
    }

    pub fn connect(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn cache_key(
        &self,
        key: &[u8; 32],
        roles: &[&str],
        scope: &str,
        expiry: &u64,
    ) -> anyhow::Result<()> {
        let role_list = roles.join(",");
        let mut stmt = self
            .conn
            .prepare("INSERT INTO keys (key, roles, scope, expiry) VALUES (?1, ?2, ?3, ?4)")?;
        stmt.execute(params![key, role_list, scope, expiry])?;
        Ok(())
    }

    pub fn evict_key(&self, key: &[u8; 32], scope: &str) -> anyhow::Result<()> {
        let mut stmt = self
            .conn
            .prepare("DELETE FROM keys WHERE key = ?1 AND scope = ?2")?;
        stmt.execute(params![key, scope])?;
        Ok(())
    }

    pub fn retrieve_key(&self, key: &[u8; 32], scope: &str) -> anyhow::Result<(String, u64)> {
        let mut stmt = self.conn.prepare(
            "SELECT roles, expiry
                 FROM keys
                 WHERE key = ?1 AND scope = ?2 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![key, scope])?;
        let mut roles = String::new();
        let mut expiry: u64 = 0;
        if let Some(row) = rows.next()? {
            roles = row.get("roles")?;
            expiry = row.get("expiry")?;
        };
        Ok((roles, expiry))
    }

    pub fn flush_all_keys(&self) -> anyhow::Result<()> {
        let mut stmt = self.conn.prepare("DELETE FROM keys")?;
        stmt.execute([])?;
        Ok(())
    }

    pub fn flush_scope_keys(&self, scope: &str) -> anyhow::Result<()> {
        let mut stmt = self.conn.prepare("DELETE FROM keys WHERE scope = ?1")?;
        stmt.execute(params![scope])?;
        Ok(())
    }
}

pub fn store_policy(
    conn: &Connection,
    policy: &serde_json::Value,
    scope: &str,
    expiry: &u64,
) -> anyhow::Result<()> {
    let policy_string = serde_json::to_string(policy)?;

    let mut stmt = conn.prepare("DELETE FROM policies WHERE scope = ?1")?;
    stmt.execute(params![scope])?;

    let mut stmt =
        conn.prepare("INSERT INTO policies (policy, scope, expiry) VALUES (?1, ?2, ?3)")?;
    stmt.execute(params![policy_string, scope, expiry])?;
    Ok(())
}

pub fn retrieve_policy(conn: &Connection, scope: &str) -> anyhow::Result<serde_json::Value> {
    let mut stmt = conn.prepare(
        "SELECT policy
         FROM policies
         WHERE scope = ?1 ORDER BY effective DESC LIMIT 1",
    )?;
    let mut rows = stmt.query(params![scope])?;
    let mut policy_string = String::new();
    if let Some(row) = rows.next()? {
        policy_string = row.get("policy")?;
    }
    Ok(serde_json::from_str(&policy_string)?)
}

pub fn revoke_policy(conn: &Connection, scope: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("DELETE FROM policies WHERE scope = ?1")?;
    stmt.execute(params![scope])?;
    Ok(())
}

pub fn flush_policies(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("DELETE FROM policies")?;
    stmt.execute([])?;
    Ok(())
}

pub fn store_rhex(conn: &Connection, rhex: &Rhex) -> anyhow::Result<()> {
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
        magic: *b"RHEX\x01\x00",
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

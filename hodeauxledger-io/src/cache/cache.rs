use rusqlite::{Connection, params};

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

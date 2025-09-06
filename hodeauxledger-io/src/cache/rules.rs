use hodeauxledger_core::policy::rule::Rule;
use rusqlite::{Connection, params};

pub fn cache_rule(conn: &Connection, rule: &Rule, scope: &str) -> Result<(), anyhow::Error> {
    let record_type = rule.record_type.clone();
    let rate = rule.rate_per_mark as i64;
    let roles = rule.append_roles.join(",");
    let quorum = rule.quorum_k as i64;
    let quorum_roles = rule.quorum_roles.join(",");

    conn.execute(
        "DELETE FROM rules WHERE scope = ?1 AND record_type = ?2",
        params![scope],
    )?;

    conn.execute(
        "INSERT INTO rules (
            scope, record_type, rate, roles, quorum, quorum_roles
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![scope, record_type, rate, roles, quorum, quorum_roles,],
    )?;

    Ok(())
}

pub fn retrieve_rules(conn: &Connection, scope: &str) -> Result<Vec<Rule>, anyhow::Error> {
    let mut stmt = conn.prepare(
        "SELECT record_type, rate, roles, quorum, quorum_roles
         FROM rules
         WHERE scope = ?1",
    )?;
    let mut rows = stmt.query(params![scope])?;
    let mut rules = Vec::new();
    while let Some(row) = rows.next()? {
        let record_type: String = row.get("record_type")?;
        let rate: i64 = row.get("rate")?;
        let roles: String = row.get("roles")?;
        let quorum: i64 = row.get("quorum")?;
        let quorum_roles: String = row.get("quorum_roles")?;
        rules.push(Rule {
            record_type,
            rate_per_mark: rate as u64,
            append_roles: roles.split(",").map(|s| s.to_string()).collect(),
            quorum_k: quorum as u8,
            quorum_roles: quorum_roles.split(",").map(|s| s.to_string()).collect(),
        });
    }
    Ok(rules)
}

pub fn evict_rule(conn: &Connection, scope: &str, record_type: &str) -> Result<(), anyhow::Error> {
    let mut stmt = conn.prepare("DELETE FROM rules WHERE scope = ?1 AND record_type = ?2")?;
    stmt.execute(params![scope, record_type])?;
    Ok(())
}

pub fn flush_rules(conn: &Connection) -> Result<(), anyhow::Error> {
    let mut stmt = conn.prepare("DELETE FROM rules")?;
    stmt.execute([])?;
    Ok(())
}

pub fn build_table(conn: &Connection) -> Result<(), anyhow::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rules (
            scope TEXT,
            note TEXT,
            record_type TEXT,
            rate INTEGER,
            roles TEXT,
            quorum INTEGER,
            quorum_roles TEXT,
            PRIMARY KEY (scope, record_type)
        )",
        [],
    )?;
    Ok(())
}

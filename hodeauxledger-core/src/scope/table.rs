use crate::scope::scope::Scope;
use anyhow::Result;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScopeTable {
    pub scopes: Vec<Scope>,
}

impl ScopeTable {
    pub fn new(scopes: Vec<Scope>) -> Self {
        Self { scopes }
    }

    /// Accepts either:
    /// - {"scopes":[ ... ]}  (preferred)
    /// - [ ... ]             (back-compat; wraps into {scopes: [...]})
    pub fn from_json(json: serde_json::Value) -> Result<Self> {
        // Try the preferred object shape first
        if json.get("scopes").is_some() {
            let table: ScopeTable = serde_json::from_value(json)?;
            return Ok(table);
        }
        // Fallback: bare array of scopes
        if json.is_array() {
            let scopes: Vec<Scope> = serde_json::from_value(json)?;
            return Ok(Self { scopes });
        }
        Err(anyhow!("expected {{\"scopes\": [...]}} or [...]"))
    }

    /// Serialize the full wrapper (not just the inner vec)
    pub fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }

    pub fn lookup(&self, scope_name: &str) -> Option<&Scope> {
        self.scopes.iter().find(|s| s.name == scope_name)
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn remove_scope(&mut self, scope_name: &str) {
        self.scopes.retain(|s| s.name != scope_name);
    }
}

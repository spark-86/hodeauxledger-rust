use serde::{Deserialize, Serialize};

use crate::policy::rule::Rule;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Policy {
    #[serde(rename = "🌐", alias = "scope")]
    pub scope: String, // String name of the scope this affects
    #[serde(rename = "⛓️", alias = "rules")]
    pub rules: Vec<Rule>, // Set of append rules
    #[serde(rename = "🤝⏳", alias = "quorum_ttl")]
    pub quorum_ttl: u64, // Amount of time to give quorum
    #[serde(rename = "🟢🕑", alias = "effective_micromark")]
    pub effective_micromark: u64, // Effective micromark time
    #[serde(rename = "🔴🕑", alias = "expiration_micromark")]
    pub expiration_micromark: u64, // Expiration micromark time
}

impl Policy {
    pub fn new(scope: &str, rules: Vec<Rule>) -> Self {
        Self {
            scope: scope.to_string(),
            rules,
            quorum_ttl: 0,
            effective_micromark: 0,
            expiration_micromark: 0,
        }
    }

    pub fn default() -> Self {
        let rules = vec![Rule::new("policy:set", &["👑"], 1, 80)];
        Self {
            scope: "".to_string(),
            rules,
            quorum_ttl: 1_000_000, // 1 Mark
            effective_micromark: 0,
            expiration_micromark: 0,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "rules": self.rules,
            "quorum_ttl": self.quorum_ttl,
            "effective_micromark": self.effective_micromark,
            "expiration_micromark": self.expiration_micromark,
        })
    }

    pub fn from_json(json: serde_json::Value) -> Self {
        serde_json::from_value(json).expect("failed to parse policy")
    }
}

use serde::{Deserialize, Serialize};

use crate::policy::{default::Default, rule::Rule};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Policy {
    #[serde(rename = "🌐", alias = "scope")]
    pub scope: String, // String name of the scope this affects
    #[serde(rename = "🧱", alias = "defaults")]
    pub defaults: Option<Default>, // Optional default policy
    #[serde(rename = "⛓️", alias = "rules")]
    pub rules: Vec<Rule>, // Set of append rules
    #[serde(rename = "🤝⏳", alias = "quorum_ttl")]
    pub quorum_ttl: Option<u64>, // Amount of time to give quorum
    #[serde(rename = "🟢🕑", alias = "effective_micromark")]
    pub effective_micromark: Option<u64>, // Effective micromark time
    #[serde(rename = "🔴🕑", alias = "expiration_micromark")]
    pub expiration_micromark: Option<u64>, // Expiration micromark time
    #[serde(rename = "🗒️", alias = "note")]
    pub note: Option<String>, // Optional note
}

impl Policy {
    pub fn new(scope: &str, rules: Vec<Rule>) -> Self {
        Self {
            scope: scope.to_string(),
            defaults: None,
            rules,
            quorum_ttl: None,
            effective_micromark: None,
            expiration_micromark: None,
            note: None,
        }
    }

    pub fn default() -> Self {
        let rules = vec![Rule::new("policy:set", &["👑"], 1, 80)];
        Self {
            scope: "".to_string(),
            defaults: Some(Default {
                roles: vec!["👑".to_string()],
                quorum_k: 1,
                rate_per_mark: 80,
            }),
            rules,
            quorum_ttl: Some(1_000_000), // 1 Mark
            effective_micromark: Some(0),
            expiration_micromark: Some(0),
            note: None,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "⛓️": self.rules,
            "🧱": self.defaults,
            "🗒️": self.note,
            "🌐": self.scope,
            "🤝⏳": self.quorum_ttl,
            "🟢🕑": self.effective_micromark,
            "🔴🕑": self.expiration_micromark,
        })
    }

    pub fn from_json(json: serde_json::Value) -> Self {
        serde_json::from_value(json).expect("failed to parse policy")
    }
}

use serde::{Deserialize, Serialize};

use crate::{policy::rule::Rule, scope::authority::Authority};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Policy {
    pub scope_name: String,          // String name of the scope this affects
    pub rules: Vec<Rule>,            // Set of append rules
    pub quorum_ttl: u64,             // Amount of time to give quorum
    pub authorities: Vec<Authority>, // Authorities for the
}

impl Policy {
    pub fn new(scope_name: &str, rules: Vec<Rule>) -> Self {
        Self {
            scope_name: scope_name.to_string(),
            rules,
            quorum_ttl: 0,
            authorities: Vec::new(),
        }
    }
}

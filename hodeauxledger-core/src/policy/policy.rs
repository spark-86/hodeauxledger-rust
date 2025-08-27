use serde::Deserialize;

use crate::policy::rule::Rule;

#[derive(Debug, Deserialize)]
pub struct Policy {
    pub scope_name: String, // String name of the scope this affects
    pub rules: Vec<Rule>,   // Set of append rules
}

impl Policy {
    pub fn new(scope_name: &str, rules: Vec<Rule>) -> Self {
        Self {
            scope_name: scope_name.to_string(),
            rules,
        }
    }
}

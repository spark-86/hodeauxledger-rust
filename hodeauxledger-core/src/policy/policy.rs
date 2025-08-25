use crate::policy::rule::Rule;

pub struct Policy {
    pub scope_name: String,
    pub rules: Vec<Rule>,
}

impl Policy {
    pub fn new(scope_name: &str, rules: Vec<Rule>) -> Self {
        Self {
            scope_name: scope_name.to_string(),
            rules,
        }
    }
}

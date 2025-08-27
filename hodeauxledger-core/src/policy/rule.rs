use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct Rule {
    pub record_type: String,
    pub append_roles: Vec<String>,
    pub quorum_k: u8,
    pub rate_per_mark: u64,
}

impl Rule {
    /// Create a new Rule.
    pub fn new(record_type: &str, append_roles: &[&str], quorum_k: u8, rate_per_mark: u64) -> Self {
        Self {
            record_type: record_type.to_string(),
            append_roles: append_roles.iter().map(|s| s.to_string()).collect(),
            quorum_k,
            rate_per_mark,
        }
    }

    /// Check if any of the provided roles are in this rule's append_roles.
    /// E.g. if rule allows ["authority", "usher"] and we have ["mirror", "usher"],
    pub fn is_in_roles(&self, roles: &[&String]) -> bool {
        roles.iter().any(|role| self.append_roles.contains(role))
    }
}

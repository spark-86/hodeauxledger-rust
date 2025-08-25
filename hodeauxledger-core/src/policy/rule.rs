#[derive(Clone)]
pub struct Rule {
    pub record_type: String,
    pub append_roles: Vec<String>,
    pub quorum_k: u8,
}

impl Rule {
    pub fn new(record_type: &str, append_roles: &[&str], quorum_k: u8) -> Self {
        Self {
            record_type: record_type.to_string(),
            append_roles: append_roles.iter().map(|s| s.to_string()).collect(),
            quorum_k,
        }
    }

    pub fn is_in_roles(&self, roles: &[&String]) -> bool {
        roles.iter().any(|role| self.append_roles.contains(role))
    }
}

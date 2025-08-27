use serde::Deserialize;

use crate::{policy::policy::Policy, scope::authority::Authority};

#[derive(Debug, Deserialize)]
pub struct Scope {
    pub name: String,
    pub role: String,
    pub last_synced: u64,
    pub policy: Policy,
    pub authorities: Vec<Authority>,
}

impl Scope {
    pub fn new(name: &String, role: &String) -> Self {
        Self {
            name: name.to_string(),
            role: role.to_string(),
            last_synced: 0,
            policy: Policy::new("", [].to_vec()),
            authorities: [].to_vec(),
        }
    }

    pub fn can_append_rt(&self, record_type: &str) -> bool {
        if !self.writable() {
            return false;
        }
        let record_type = record_type.to_string();
        self.policy
            .rules
            .iter()
            .any(|rule| rule.record_type == record_type)
    }

    /// Is the scope writable or are we just a mirror?
    pub fn writable(&self) -> bool {
        match self.role.as_str() {
            "authority" => true,
            _ => false,
        }
    }

    pub fn remove_authority_by_key(&mut self, authority_pk: [u8; 32]) {
        self.authorities.retain(|a| a.public_key != authority_pk);
    }
}

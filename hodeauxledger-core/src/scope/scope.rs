use serde::{Deserialize, Serialize};

use crate::{
    policy::policy::Policy,
    scope::authority::{self, Authority},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scope {
    pub name: String,
    pub role: String, // "authority" | "mirror" | ...
    pub last_synced: u64,
    pub policy: Policy,
    pub authorities: Vec<Authority>,
    pub head: [u8; 32],
}

impl Scope {
    pub fn new(name: &str, role: &str) -> Self {
        Self {
            name: name.to_string(),
            role: role.to_string(),
            last_synced: 0,
            policy: Policy::new("", Vec::new()),
            authorities: Vec::new(),
            head: [0u8; 32],
        }
    }

    pub fn can_append_rt(&self, record_type: &str) -> bool {
        if !self.writable() {
            return false;
        }
        self.policy
            .rules
            .iter()
            .any(|rule| rule.record_type == record_type)
    }

    /// Is the scope writable or are we just a mirror?
    pub fn writable(&self) -> bool {
        matches!(self.role.as_str(), "authority")
    }

    pub fn remove_authority_by_key(&mut self, authority_pk: [u8; 32]) {
        self.authorities.retain(|a| a.public_key != authority_pk);
    }

    pub fn to_name_parts(&self) -> Vec<String> {
        self.name.split('.').map(|s| s.to_string()).collect()
    }

    /// Weighted K pick; returns **owned** so you can move/send it.
    pub fn get_authorities_weighted(&self, k: usize) -> Vec<Authority> {
        let n = self.authorities.len();
        if n == 0 || k == 0 {
            return Vec::new();
        }
        if n <= k {
            return self.authorities.clone();
        }
        authority::pick_k_weighted_unique(&self.authorities, k)
            .into_iter()
            .cloned()
            .collect()
    }
}

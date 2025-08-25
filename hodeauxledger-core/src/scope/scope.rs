use crate::policy::policy::Policy;

pub struct Scope {
    pub name: String,
    pub role: String,
    pub last_synced: u64,
    pub policy: Policy,
}

impl Scope {
    pub fn new(name: &String, role: &String) -> Self {
        Self {
            name: name.to_string(),
            role: role.to_string(),
            last_synced: 0,
            policy: Policy::new("", [].to_vec()),
        }
    }

    pub fn can_append(&self, record_type: &str) -> bool {
        if !self.writable() {
            return false;
        }
        let record_type = record_type.to_string();
        self.policy
            .rules
            .iter()
            .any(|rule| rule.record_type == record_type)
    }

    pub fn writable(&self) -> bool {
        let writable = match self.role.as_str() {
            "authority" => true,
            _ => false,
        };
        writable
    }
}

use crate::to_base64;

pub struct Alias {
    pub name: String,
    pub scope: String,
    pub hash: [u8; 32],
}

impl Alias {
    pub fn new(name: &str, scope: &str, hash: &[u8; 32]) -> Self {
        Self {
            name: name.to_string(),
            scope: scope.to_string(),
            hash: *hash,
        }
    }
    pub fn to_string(&self) -> String {
        format!("rhex://{}/{}", self.scope, self.name)
    }
    pub fn to_resolved(&self) -> String {
        format!("rhex://{}/{}", self.scope, to_base64(&self.hash))
    }
}

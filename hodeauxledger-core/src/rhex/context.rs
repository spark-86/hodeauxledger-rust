use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    pub at: u64,
}

impl Context {
    pub fn new() -> Self {
        Self { at: 0 }
    }
}

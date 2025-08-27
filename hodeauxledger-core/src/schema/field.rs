use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaField {
    pub name: String,
    pub label: String,
    pub description: String,
    pub data_type: String,
    pub value: Value,
    pub required: bool,
}

impl SchemaField {
    pub const DT_STRING: &str = "string";
    pub const DT_NUMBER: &str = "number";
    pub const DT_BOOLEAN: &str = "boolean";
    pub const DT_OBJECT: &str = "object";
    pub const DT_ARRAY: &str = "array";

    pub fn new(
        name: &str,
        label: &str,
        description: &str,
        data_type: &str,
        value: Value,
        required: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            description: description.to_string(),
            data_type: data_type.to_string(),
            value,
            required,
        }
    }
}

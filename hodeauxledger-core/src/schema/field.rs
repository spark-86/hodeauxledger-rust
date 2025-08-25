use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SchemaField {
    name: String,
    label: String,
    description: String,
    data_type: String,
    value: String,
    required: bool,
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
        value: &str,
        required: &bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            description: description.to_string(),
            data_type: data_type.to_string(),
            value: value.to_string(),
            required: *required,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": &self.name,
            "label": &self.label,
            "description": &self.description,
            "data_type": &self.data_type,
            "value": &self.value,
            "required": &self.required,
        })
    }

    pub fn from_json(json: serde_json::Value) -> Self {
        Self {
            name: json["name"].as_str().unwrap().to_string(),
            label: json["label"].as_str().unwrap().to_string(),
            description: json["description"].as_str().unwrap().to_string(),
            data_type: json["data_type"].as_str().unwrap().to_string(),
            value: json["value"].as_str().unwrap().to_string(),
            required: json["required"].as_bool().unwrap(),
        }
    }
}

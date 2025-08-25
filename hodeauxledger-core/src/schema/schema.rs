use crate::schema::field::SchemaField;

pub struct Schema {
    id_str: String,
    fields: Vec<SchemaField>,
}

impl Schema {
    pub fn new(id_str: &str, fields: Vec<SchemaField>) -> Self {
        Self {
            id_str: id_str.to_string(),
            fields,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": &self.id_str,
            "fields": &self.fields,
        })
    }

    pub fn from_json(json: serde_json::Value) -> Self {
        Self {
            id_str: json["id"].as_str().unwrap().to_string(),
            fields: json["fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| SchemaField::from_json(f.clone()))
                .collect(),
        }
    }
}

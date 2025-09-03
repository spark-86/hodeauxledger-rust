use crate::schema::field::SchemaField;

pub struct Schema {
    pub id_str: String,
    pub version: String,
    pub fields: Vec<SchemaField>,
}

impl Schema {
    pub fn new(id_str: &str, version: &str, fields: Vec<SchemaField>) -> Self {
        Self {
            id_str: id_str.to_string(),
            version: version.to_string(),
            fields,
        }
    }
}

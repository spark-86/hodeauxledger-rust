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

pub fn breakdown_schema_url(url: &str) -> Result<(String, String), anyhow::Error> {
    let uri = url[..14].to_string();
    if uri == "rhex://schema/" {
        // We have the proper start
        let working_part = &url[14..];
        let parts: Vec<&str> = working_part.split('@').collect();
        let id_str = parts[0];
        let version = parts[1];
        return Ok((id_str.to_string(), version.to_string()));
    }
    anyhow::bail!("Invalid schema URL");
}

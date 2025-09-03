use hodeauxledger_core::{RhexUrl, schema::schema::Schema};

pub fn check_schema(data: &serde_json::Value) -> Result<(), anyhow::Error> {
    let schema_url = data.get("schema").and_then(|v| v.as_str());
    if schema_url.is_none() {
        return Err(anyhow::anyhow!("missing schema"));
    }
    let schema_url = schema_url.unwrap();

    let schema = get_schema(&RhexUrl::from_string(&schema_url)?)?;
    let _ = schema;
    Ok(())
}

fn get_schema(url: &RhexUrl) -> Result<Schema, anyhow::Error> {
    let _ = url;
    Ok(Schema::new("rhex://schema/none", "", vec![]))
}

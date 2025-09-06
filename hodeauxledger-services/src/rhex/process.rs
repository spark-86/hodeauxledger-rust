use hodeauxledger_core::{Rhex, RhexUrl, rhex, schema::schema::Schema};

use crate::process::{request, scope};
pub fn process_rhex(rhex: &Rhex, first_time: bool) -> Vec<Rhex> {
    let exploded_record_type = rhex.intent.record_type.split(":").collect::<Vec<&str>>();
    let record_major = exploded_record_type[0];

    let result = match record_major {
        "🌐" | "scope" => scope::process_scope_rhex(rhex, first_time),
        //"🔑" | "key" => {}
        //"👑" | "authority" => {}
        "📩" | "request" => request::process_request_rhex(rhex, first_time),
        //"📦" | "record" => {}
        _ => Ok(Vec::new()),
    };
    if result.is_err() {}
    let returned_rhex = result.unwrap();
    returned_rhex
}

pub fn get_schema(rhex: &Rhex) -> Result<Schema, anyhow::Error> {
    let schema_rec = rhex.intent.data.get("schema").and_then(|v| v.as_str());
    if schema_rec.is_none() {
        return Err(anyhow::anyhow!("missing schema"));
    }
    let schema_url = RhexUrl::from_string(schema_rec.unwrap());
    let schema = Schema::new(&schema_url.unwrap().hash_alias.clone(), "0", [].to_vec());
    Ok(schema)
}

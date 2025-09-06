use hodeauxledger_io::cache::{cache::Cache, rhex::retrieve_scope_rhex, scopes::retrieve_scope};

pub fn get_record_types_from_cache(
    scope: &str,
    record_types: &Vec<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let cache = Cache::connect("")?;
    let scope_data = retrieve_scope_rhex(&cache.conn, scope);
    if scope_data.is_err() {
        return Err(anyhow::anyhow!("scope not found"));
    }
    let scope_data = scope_data.unwrap();

    let mut out = Vec::new();
    for rhex in scope_data {
        for record_type in record_types {
            if rhex.intent.record_type == *record_type {
                out.push(rhex.intent.record_type.clone());
            }
        }
    }
    Ok(out)
}

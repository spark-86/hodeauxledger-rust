use hodeauxledger_core::{Intent, Rhex};
use hodeauxledger_io::cache;
use hodeauxledger_io::cache::cache::Cache;
use hodeauxledger_io::cache::rhex::retrieve_scope_rhex;
use serde_json::json;

pub fn request_rhex(rhex: &Rhex, first_time: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    let cache = Cache::connect("")?;
    let scope = &rhex.intent.scope;
    let results = retrieve_scope_rhex(&cache.conn, scope);
    if results.is_err() {
        return Err(anyhow::anyhow!("scope not found"));
    }
    let results = results.unwrap();
    if first_time {
        return Ok(results);
    }
    Ok(Vec::new())
}

pub fn head_rhex(rhex: &Rhex, first_time: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    let scope_name = &rhex.intent.scope;
    let cache = Cache::connect("")?;
    let (_, _, head) = cache::scopes::retrieve_scope(&cache.conn, scope_name)?;
    let draft = Rhex::draft(Intent {
        previous_hash: [255u8; 32],
        scope: scope_name.to_string(),
        nonce: Rhex::gen_nonce(),
        author_public_key: rhex.intent.usher_public_key,
        usher_public_key: rhex.intent.usher_public_key,
        record_type: "response:head".to_string(),
        data: json!({
            "head" : head
        }),
    });
    let _ = first_time;
    Ok(vec![draft])
}

pub fn process_request_rhex(rhex: &Rhex, first_time: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    let response = match rhex.intent.record_type.as_str() {
        "📩:R⬢" | "request:rhex" => request_rhex(rhex, first_time),
        "📩:➡️🧬" | "request:head" => head_rhex(rhex, first_time),
        _ => Ok(Vec::new()),
    };
    if response.is_err() {}
    let returned_rhex = response.unwrap();
    Ok(returned_rhex)
}

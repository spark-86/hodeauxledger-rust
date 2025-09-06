use hodeauxledger_core::Rhex;
use hodeauxledger_io::cache::cache::Cache;
use hodeauxledger_io::cache::rhex::retrieve_scope_rhex;

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

pub fn process_request_rhex(rhex: &Rhex, first_time: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    let response = match rhex.intent.record_type.as_str() {
        "📩:R⬢" | "request:rhex" => request_rhex(rhex, first_time),
        _ => Ok(Vec::new()),
    };
    if response.is_err() {}
    let returned_rhex = response.unwrap();
    Ok(returned_rhex)
}

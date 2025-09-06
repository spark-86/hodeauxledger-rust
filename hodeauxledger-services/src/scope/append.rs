use hodeauxledger_core::Key;
use hodeauxledger_io::{Cache, cache};

pub fn can_append(scope: &str, record_type: &str, key: &Key) -> Result<bool, anyhow::Error> {
    // Get the policy for the scope
    let cache = Cache::connect("")?;
    cache::policies::retrieve_policy(&cache.conn, &scope);

    // Figure out if this key is able to append this record_type at all

    // Do we meet the requirements?

    Ok(true)
}

use crate::cache::{self, cache::Cache};

pub fn build_cache_db(path: &str) -> Result<(), anyhow::Error> {
    let cache = Cache::connect(path)?;
    cache::authorities::build_table(&cache.conn)?;
    println!("built authorities table");
    cache::scopes::build_table(&cache.conn)?;
    cache::rhex::build_table(&cache.conn)?;
    cache::key::build_table(&cache.conn)?;
    cache::aliases::build_table(&cache.conn)?;

    Ok(())
}

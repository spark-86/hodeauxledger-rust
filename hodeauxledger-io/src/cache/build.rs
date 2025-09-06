use crate::cache::{self, cache::Cache};

pub fn build_cache_db(path: &str) -> Result<(), anyhow::Error> {
    let cache = Cache::connect(path)?;
    cache::authorities::build_table(&cache.conn)?;
    println!("built authorities table");
    cache::scopes::build_table(&cache.conn)?;
    println!("built scopes table");
    cache::rhex::build_table(&cache.conn)?;
    println!("built rhex table");
    let wtf = cache::key::build_table(&cache.conn);
    if wtf.is_err() {
        println!("Key table error {}", wtf.unwrap_err());
    }
    println!("built keys table");
    //cache::policies::build_table(&cache.conn)?;
    let wtf = cache::policies::build_table(&cache.conn);
    if wtf.is_err() {
        println!("Policy table error {}", wtf.unwrap_err());
    }
    println!("built policies table");
    cache::aliases::build_table(&cache.conn)?;
    println!("built aliases table");
    cache::rules::build_table(&cache.conn)?;
    println!("built rules table");
    Ok(())
}

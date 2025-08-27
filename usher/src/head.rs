pub fn get_head(scope: &str) -> anyhow::Result<(), anyhow::Error> {
    println!("Hello, {}!", scope);
    Ok(())
}

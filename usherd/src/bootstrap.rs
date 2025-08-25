use hodeauxledger_io::disk;

pub fn get_scope_list(ledger_path: &str) -> Result<(), anyhow::Error> {
    let scope_table = disk::get_scope_table(ledger_path)?;
    Ok(())
}

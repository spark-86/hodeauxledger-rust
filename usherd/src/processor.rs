use hodeauxledger_core::{Key, Rhex};
use hodeauxledger_services::{build::error, rhex};

pub fn process_rhex(rhex: &Rhex, hot_key: &Key, verbose: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    // First we verify the R⬢
    if verbose {
        println!("Verifying R⬢...")
    }
    if let Err(e) = rhex.validate() {
        eprintln!("❌ R⬢ validation failed: {e}");
        let err_rhex = error::verifiy_failed(hot_key, e, rhex)?;
        return Ok(vec![err_rhex]);
    }
    if verbose {
        println!("R⬢ verified!")
    }

    // Next we make sure we can even do anything with this.
    // Pull scope policy

    // Can we submit this type of R⬢?

    // Does this match schema?

    // All the checks are clear, chocks are loose and boosters are
    // a go.
    let returned_rhex = rhex::process::process_rhex(rhex, true);
    Ok(returned_rhex)
}

use hodeauxledger_core::{Key, Rhex};
use hodeauxledger_services::{build::error, rhex};

pub fn process_rhex(rhex: &Rhex, hot_key: &Key, verbose: bool) -> Result<Vec<Rhex>, anyhow::Error> {
    // First we verify the R⬢
    if verbose {
        println!("Verifying R⬢...")
    }

    // If this R⬢ only has an author sig, we're assuming they are looking
    // for an usher to sign off. Check to make sure we are an authority
    // for the scope in question
    if rhex.signatures.len() == 1 {
        if rhex.signatures[0].sig_type != 0 {
            return Err(anyhow::anyhow!("invalid signature type"));
        }
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

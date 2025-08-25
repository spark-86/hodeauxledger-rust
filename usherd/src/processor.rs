use hodeauxledger_core::Rhex;

pub fn process_rhex(rhex: Rhex, verbose: bool) {
    // First we verify the R⬢
    if verbose {
        println!("Verifying R⬢...")
    }
    if let Err(e) = rhex.validate() {
        eprintln!("❌ R⬢ validation failed: {e}");
        return;
    }
    if verbose {
        println!("R⬢ verified!")
    }

    // Next we make sure we can even do anything with this.
    // Pull scope policy

    // Can we submit this type of R⬢?

    // Does this need to be forwarded?

    // Does this match schema?
}

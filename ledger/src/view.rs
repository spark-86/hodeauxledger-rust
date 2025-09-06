use hodeauxledger_core::Rhex;
use hodeauxledger_io::disk::rhex as diskrhex;
use hodeauxledger_io::screen::pretty_print_rhex;
use std::path::Path;

use crate::argv;

pub fn view_rhex(rhex: &Rhex) -> anyhow::Result<(), anyhow::Error> {
    pretty_print_rhex(rhex);
    Ok(())
}

pub fn view(args: &argv::ViewArgs) -> anyhow::Result<(), anyhow::Error> {
    let rhex_path = &args.input;
    let rhex = diskrhex::load_rhex(&Path::new(rhex_path).to_path_buf())?;
    view_rhex(&rhex)?;
    Ok(())
}

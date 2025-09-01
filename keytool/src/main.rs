use clap::Parser;
use owo_colors::OwoColorize;

use crate::argv::Command;

mod argv;
mod crypto;
mod sign;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn show_banner() {
    println!(
        "{}{}",
        "HodeauxLedger Key Tool v".magenta().bold(),
        VERSION.magenta().bold()
    );
}

fn main() -> anyhow::Result<()> {
    let args = argv::Cli::parse();
    if !args.quiet {
        show_banner();
    }
    match args.cmd {
        Command::Generate(generate_args) => {
            crypto::generate_key(generate_args, args.verbose, args.quiet)?
        }
        Command::View(view_args) => crypto::view_key(view_args, args.verbose, args.quiet)?,
        Command::Sign(sign_args) => sign::sign(sign_args, args.verbose, args.quiet)?,
        Command::Verify(verify_args) => sign::verify(verify_args, args.verbose, args.quiet)?,
    };
    Ok(())
}

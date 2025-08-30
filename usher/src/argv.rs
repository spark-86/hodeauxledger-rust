use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ledger", about = "HodeauxLedger 📣 Client Tool")]
pub struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Choose an operation
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    // Submit a R⬢ to a usher
    Submit(SubmitArgs),

    // Get authorities
    Auth(AuthArgs),
}

#[derive(Args, Debug)]
pub struct SubmitArgs {
    #[arg(short, long)]
    pub rhex: String,

    #[arg(short, long)]
    pub host: String,

    #[arg(short, long)]
    pub port: String,
}

#[derive(Args, Debug)]
pub struct AuthArgs {
    #[arg(short, long)]
    pub scope: String,
}

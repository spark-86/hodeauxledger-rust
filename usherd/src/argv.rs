use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "usherd", about = "HodeauxLedger 📣😈 Tool")]
pub struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Path to data
    pub ledger_path: Option<String>,

    /// Choose an operation
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    // Listen for connections
    Listen(ListenArgs),

    // Rebuild cache database from scratch
    Rebuild(RebuildArgs),
}

#[derive(Args, Debug)]
pub struct ListenArgs {
    /// Port to listen on
    #[arg(short, long)]
    pub port: Option<String>,

    /// Host to listen on
    #[arg(long)]
    pub host: Option<String>,

    /// Path to hot key
    #[arg(long)]
    pub hot_key: Option<String>,
}

#[derive(Args, Debug)]
pub struct RebuildArgs {
    /// Path to DB file   
    #[arg(short, long)]
    pub db_path: Option<String>,
}

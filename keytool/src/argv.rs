use clap::{Args, Parser, Subcommand};

/// HodeauxLedger Standard Tool
#[derive(Parser, Debug)]
#[command(name = "ledger", about = "HodeauxLedger Key Tool")]
pub struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Choose an operation
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    // Sign
    Sign(SignArgs),

    // Verify
    Verify(VerifyArgs),

    // Generate
    Generate(GenerateArgs),

    // View
    View(ViewArgs),
}

#[derive(Args, Debug)]
pub struct SignArgs {
    /// 🔑 file to load
    #[arg(short, long)]
    pub load: String,

    /// 🔑 password to decrypt keyfile
    #[arg(short, long)]
    pub password: Option<String>,

    /// Is the key hot?
    #[arg(long)]
    pub hot: bool,

    /// R⬢ to sign
    #[arg(long)]
    pub rhex_input: String,

    /// R⬢ to ouput
    #[arg(long)]
    pub rhex_output: String,

    #[arg(long)]
    pub signature_type: Option<String>,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// R⬢ to verify
    #[arg(long)]
    pub input: String,
}

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// 🔑 file to save
    #[arg(short, long)]
    pub save: String,

    #[arg(short, long)]
    pub password: Option<String>,

    #[arg(long)]
    pub show_private_key: bool,

    #[arg(long)]
    pub hot: bool,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// 🔑 file to load
    #[arg(short, long)]
    pub load: String,

    /// 🔑 password to decrypt 🔑 file
    #[arg(short, long)]
    pub password: Option<String>,

    #[arg(long)]
    pub hot: bool,

    #[arg(long)]
    pub show_private_key: bool,
}

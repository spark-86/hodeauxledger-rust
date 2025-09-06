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

    // Make an enc key hot
    Hot(HotArgs),

    // Encrypt hot key
    Encrypt(EncryptArgs),
}

#[derive(Args, Debug)]
pub struct KeyOpts {
    /// 🔑 key file to load
    #[arg(short, long)]
    pub keyfile: String,

    #[arg(short, long)]
    pub password: Option<String>,

    /// Is the key hot?
    #[arg(long)]
    pub hot: bool,
}

#[derive(Args, Debug)]
pub struct SignArgs {
    #[command(flatten)]
    pub keys: KeyOpts,

    /// R⬢ to sign
    #[arg(short, long)]
    pub input: String,

    /// R⬢ to ouput
    #[arg(short, long)]
    pub output: String,

    #[arg(long)]
    pub signature_type: String,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// R⬢ to verify
    #[arg(long)]
    pub input: String,
}

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// 🔑 shit
    #[command(flatten)]
    pub keys: KeyOpts,

    #[arg(long)]
    pub show_private_key: bool,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    #[command(flatten)]
    pub keys: KeyOpts,
    /// 🔑 file to load

    #[arg(long)]
    pub show_private_key: bool,
}

#[derive(Args, Debug)]
pub struct HotArgs {
    /// 🔑 file to load
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub password: String,

    /// 🔑 file to save
    #[arg(short, long)]
    pub output: String,
}

#[derive(Args, Debug)]
pub struct EncryptArgs {
    /// 🔑 file to load
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub password: String,

    /// 🔑 file to save
    #[arg(short, long)]
    pub output: String,
}

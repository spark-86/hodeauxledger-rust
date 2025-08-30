use clap::{Args, Parser, Subcommand};

/// HodeauxLedger Standard Tool
#[derive(Parser, Debug)]
#[command(name = "ledger", about = "HodeauxLedger Standard Tool")]
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
    /// Build an unsigned R⬢ from inputs (no signatures yet)
    Build(BuildArgs),

    /// Craft + sign an R⬢ in one go (author key required)
    Craft(CraftArgs),

    /// Finalize an existing partial R⬢ (add signatures / complete fields)
    Finalize(FinalizeArgs),

    /// Verify an R⬢ (hash chain + signatures)
    Verify(VerifyArgs),

    /// Create scope genesis records
    Genesis(GenesisArgs),
}

/* ---------- shared option bundles ---------- */

#[derive(Args, Debug)]
pub struct KeyOpts {
    /// 🔑 key file to load
    #[arg(short, long)]
    pub keyfile: Option<String>,
    /// 🔑 password to decrypt keyfile
    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Args, Debug)]
pub struct SaveOpt {
    /// R⬢ filename to save
    #[arg(short, long)]
    pub save: Option<String>,
}

/* ---------- subcommands ---------- */

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// 📄 record type (e.g., policy:set)
    #[arg(long)]
    pub record_type: String,

    /// 🔭 scope (e.g., core.trust)
    #[arg(long)]
    pub scope: String,

    /// 📊 JSON file for data payload
    #[arg(long, value_name = "FILE")]
    pub data_file: String,

    /// ⬅️🧬 previous hash (hex/base64)
    #[arg(long)]
    pub previous_hash: Option<String>,

    /// 🎲 nonce
    #[arg(long)]
    pub nonce: Option<String>,

    /// ✍️🔓 author public key (base64)
    #[arg(long)]
    pub author_public_key: Option<String>,

    /// 📣🔓 usher public key (base64)
    #[arg(long)]
    pub usher_public_key: Option<String>,

    #[command(flatten)]
    pub save: SaveOpt,
}

#[derive(Args, Debug)]
pub struct CraftArgs {
    /// 📄 record type
    #[arg(long)]
    pub record_type: String,

    /// 🔭 scope
    #[arg(long)]
    pub scope: String,

    /// 📊 JSON file for data payload
    #[arg(long, value_name = "FILE")]
    pub data_file: String,

    /// ⬅️🧬 previous hash
    #[arg(long)]
    pub previous_hash: Option<String>,

    /// 🎲 nonce
    #[arg(long)]
    pub nonce: Option<String>,

    #[arg(long)]
    pub author_public_key: Option<String>,

    #[arg(long)]
    pub usher_public_key: Option<String>,

    #[command(flatten)]
    pub keys: KeyOpts,

    #[command(flatten)]
    pub save: SaveOpt,
}

#[derive(Args, Debug)]
pub struct FinalizeArgs {
    /// R⬢ file to open and finalize
    #[arg(short, long, value_name = "FILE")]
    pub rhex: String,

    #[command(flatten)]
    pub keys: KeyOpts,

    #[command(flatten)]
    pub save: SaveOpt,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// R⬢ file to verify
    #[arg(short, long, value_name = "FILE")]
    pub rhex: String,
}

#[derive(Args, Debug)]
pub struct GenesisArgs {
    /// 📊 JSON file for genesis metadata (description, etc.)
    #[arg(long, value_name = "FILE")]
    pub data_file: String,

    #[command(flatten)]
    pub keys: KeyOpts,

    #[command(flatten)]
    pub save: SaveOpt,
}

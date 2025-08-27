use clap::Parser;
use owo_colors::OwoColorize;

mod crypto;
mod sign;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "keytool", about = "HodeauxLedger Key Tool")]
struct Cli {
    action: String,

    #[arg(short, long)]
    load: Option<String>,

    #[arg(short, long)]
    save: Option<String>,

    #[arg(long)]
    hot: bool,

    #[arg(short, long)]
    password: Option<String>,

    #[arg(long)]
    show_private_key: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long)]
    rhex: Option<String>,

    #[arg(long)]
    signature_type: Option<String>,

    #[arg(long)]
    rhex_output: Option<String>,
}

fn show_banner() {
    println!(
        "{}{}",
        "HodeauxLedger Key Tool v".magenta().bold(),
        VERSION.magenta().bold()
    );
}

fn main() -> anyhow::Result<()> {
    let args: Cli = Cli::parse();
    let action = args.action.as_str();
    if !args.quiet {
        show_banner();
    }
    match action {
        "generate" => crypto::generate_key(args)?,
        "view" => crypto::view_key(args)?,
        "sign" => sign::sign(args)?,
        "verify" => sign::verify(args)?,
        _ => {
            anyhow::bail!("unknown operation");
        }
    };
    Ok(())
}

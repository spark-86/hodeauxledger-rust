use clap::Parser;
use cursive::{
    Cursive,
    view::{Nameable, Resizable},
    views::{Button, Dialog, EditView, LinearLayout, SelectView, TextView},
};
use hodeauxledger_io::disk::rhex as diskrhex;
use hodeauxledger_io::screen::pretty_print_rhex;
use std::io::Write;
use std::path::Path;

mod craft;
mod genesis;

//const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "ledger", about = "HodeauxLedger Standard Tool")]
struct Cli {
    action: String,

    #[arg(short, long)]
    save: Option<String>,

    #[arg(long)]
    previous_hash: Option<String>,

    #[arg(long)]
    nonce: Option<String>,

    #[arg(long)]
    author_public_key: Option<String>,

    #[arg(long)]
    usher_public_key: Option<String>,

    #[arg(long)]
    data_file: Option<String>,

    #[arg(long)]
    record_type: Option<String>,

    #[arg(long)]
    scope: Option<String>,

    #[arg(long)]
    schema: Option<String>,

    #[arg(short, long)]
    rhex: Option<String>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    keyfile: Option<String>,

    #[arg(short, long)]
    password: Option<String>,
}

static RECORD_TYPES: &[&str] = &["policy:set", "scope:create", "scope:genesis"];
const ID_RT_LABEL: &str = "record_type_label";

fn open_record_type_picker(siv: &mut Cursive) {
    let mut sv = SelectView::<String>::new().autojump();
    for rt in RECORD_TYPES {
        sv.add_item(*rt, rt.to_string());
    }

    sv.set_on_submit(|s, selected: &String| {
        // update label and close
        s.call_on_name(ID_RT_LABEL, |tv: &mut TextView| {
            tv.set_content(selected.clone())
        });
        s.pop_layer();
    });

    let list = sv.fixed_size((32, 8));
    siv.add_layer(
        Dialog::around(list)
            .title("Select record type")
            .button("Cancel", |s| {
                s.pop_layer();
            }),
    );
}

/// Presents a UI for building intents
fn build_intent(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let save_path = args
        .save
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("save must be specified"))?;
    println!("save path: {}", save_path);
    let mut siv = cursive::default();

    siv.add_layer(
        Dialog::new()
            .title("R⬢ Draft Intent")
            .content(
                LinearLayout::vertical()
                    .child(
                        LinearLayout::horizontal()
                            .child(TextView::new("Scope: "))
                            .child(EditView::new().fixed_width(20).with_name("scope_input")),
                    )
                    .child(
                        LinearLayout::horizontal()
                            .child(TextView::new("Author Public Key: "))
                            .child(EditView::new().fixed_width(20).with_name("author_pk_input")),
                    )
                    .child(
                        LinearLayout::horizontal()
                            .child(TextView::new("Usher Public Key: "))
                            .child(EditView::new().fixed_width(20).with_name("usher_pk_input")),
                    )
                    .child(
                        LinearLayout::horizontal()
                            .child(TextView::new("Record Type  "))
                            .child(TextView::new("").with_name(ID_RT_LABEL))
                            .child(Button::new("Change…", open_record_type_picker)),
                    ),
            )
            .button("Submit", |s| {
                s.pop_layer(); // Close the dialog
                s.add_layer(Dialog::info(format!("Sigh")).button("Quit", |s| s.quit()));
            })
            .button("Cancel", |s| s.quit()),
    );
    siv.run();
    Ok(())
}

// Finalize a R⬢ by calculating it's current_hash and saving it
fn finalize_rhex(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let rhex_path = args
        .rhex
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("rhex must be specified"))?;

    let rhex = diskrhex::load_rhex(&Path::new(rhex_path).to_path_buf())?;
    let rhex = rhex.finalize()?;
    // output completed R⬢
    if let Some(save_path) = &args.save {
        diskrhex::save_rhex(&Path::new(save_path).to_path_buf(), &rhex)?;
        pretty_print_rhex(&rhex);
    } else {
        let v = serde_cbor::value::to_value(&rhex)?;
        let bytes = serde_cbor::to_vec(&v)?;
        std::io::stdout().write_all(&bytes)?;
    }
    Ok(())
}

/// Verifies the current_hash of the R⬢
fn verify_current_hash(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let rhex_path = args
        .rhex
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("rhex must be specified"))?;
    let rhex = diskrhex::load_rhex(&Path::new(rhex_path).to_path_buf())?;
    rhex.verify_hash()?;
    println!("✅ R⬢ hash verified.");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let action = args.action.as_str();
    //println!(
    //    "{}{}",
    //    "HodeauxLedger Tool v".magenta().bold(),
    //    VERSION.magenta().bold()
    //);
    match action {
        "build" => build_intent(&args)?,
        "craft" => craft::craft_intent(&args)?,
        "finalize" => finalize_rhex(&args)?,
        "verify" => verify_current_hash(&args)?,
        "genesis" => genesis::create_genesis(&args)?,
        _ => {
            anyhow::bail!("unknown operation");
        }
    };
    Ok(())
}

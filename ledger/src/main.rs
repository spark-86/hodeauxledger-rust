use clap::Parser;
use cursive::{
    Cursive,
    view::{Nameable, Resizable},
    views::{Button, Dialog, EditView, LinearLayout, SelectView, TextView},
};
use hodeauxledger_core::crypto::b64::from_base64_to_32;
use hodeauxledger_core::crypto::key;
use hodeauxledger_core::rhex::signature::Signature;
use hodeauxledger_core::rhex::{intent::Intent, rhex::Rhex};
use hodeauxledger_io::disk;
use hodeauxledger_io::screen::pretty_print_rhex;
use std::io::Write;
use std::{path::Path, time::SystemTime};

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

/// Craft an intent entirely from the command line to be signed by
/// the keytool.
fn craft_intent(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let save_path = args
        .save
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("save must be specified"))?;
    let author_pk_b64 = args
        .author_public_key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("author_public_key must be specified"))?;
    let usher_pk_b64 = args
        .usher_public_key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("usher_public_key must be specified"))?;
    let scope = args
        .scope
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("scope must be specified"))?;
    let data_file = args
        .data_file
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("data_file must be specified"))?;
    let record_type = args
        .record_type
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("record_type must be specified"))?;
    let previous_hash_b64 = args
        .previous_hash
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("previous_hash must be specified"))?;
    let nonce = args.nonce.clone().unwrap_or_else(|| Rhex::gen_nonce());
    let data = disk::load_json_data(data_file)?;
    let ph_bytes = match previous_hash_b64.len() {
        32 => from_base64_to_32(previous_hash_b64)?,
        0 => [0u8; 32],
        _ => anyhow::bail!("previous_hash must be 32 bytes or empty"),
    };
    let nonce = nonce.to_string();
    let author_pk = from_base64_to_32(author_pk_b64)?;
    let usher_pk = from_base64_to_32(usher_pk_b64)?;
    let intent = Intent::new(
        ph_bytes,
        &scope,
        &nonce,
        author_pk,
        usher_pk,
        record_type,
        data,
    );

    let rhex = Rhex::draft(intent, Vec::new());
    // output rhex intent
    //disk::save_intent(save_path, &rhex.intent)?;
    disk::save_rhex(&Path::new(save_path).to_path_buf(), &rhex)?;
    Ok(())
}

// Finalize a R⬢ by calculating it's current_hash and saving it
fn finalize_rhex(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let rhex_path = args
        .rhex
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("rhex must be specified"))?;

    let rhex = disk::load_rhex(&Path::new(rhex_path).to_path_buf())?;
    let rhex = rhex.finalize()?;
    // output completed R⬢
    if let Some(save_path) = &args.save {
        disk::save_rhex(&Path::new(save_path).to_path_buf(), &rhex)?;
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
    let rhex = disk::load_rhex(&Path::new(rhex_path).to_path_buf())?;
    rhex.verify_hash()?;
    println!("✅ R⬢ hash verified.");
    Ok(())
}

/// Creates a genesis record. In theory only used once, ever. Why it's
/// part of the tool and not it's own standalone thing I don't know.
fn create_genesis(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let save_path = args
        .save
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("save must be specified"))?;
    let keyfile = args
        .keyfile
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("keyfile must be specified"))?;
    let password = args
        .password
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("password must be specified"))?;
    let sk = disk::load_key(keyfile, password)?;
    let pk = sk.verifying_key();
    let pk_bytes = pk.to_bytes();
    let data = serde_json::json!({
        "schema": "schema/scope.genesis/1",
        "description": "Trust Architecture Core Scope Genesis",
        "unix_at": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as u64,
    });
    let mut rhex = Rhex::draft(
        Intent::new(
            [0u8; 32],
            "",
            Rhex::gen_nonce().as_str(),
            pk_bytes,
            pk_bytes,
            "scope:genesis",
            data,
        ),
        Vec::new(),
    );

    // Sign as author
    let intent_hash = rhex.to_author_hash()?;
    rhex.signatures.push(Signature {
        sig_type: 0,
        public_key: pk_bytes,
        sig: key::sign(&intent_hash, &sk).into(),
    });

    // Sign as usher
    let usher_hash = rhex.to_usher_hash()?;
    rhex.signatures.push(Signature {
        sig_type: 1,
        public_key: pk_bytes,
        sig: key::sign(&usher_hash, &sk).into(),
    });

    // Sign quorum
    let quorum_hash = rhex.to_quorum_hash()?;
    rhex.signatures.push(Signature {
        sig_type: 2,
        public_key: pk_bytes,
        sig: key::sign(&quorum_hash, &sk).into(),
    });
    let final_rhex = rhex.finalize()?;
    disk::save_rhex(&Path::new(save_path).to_path_buf(), &final_rhex)?;
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
        "craft" => craft_intent(&args)?,
        "finalize" => finalize_rhex(&args)?,
        "verify" => verify_current_hash(&args)?,
        "genesis" => create_genesis(&args)?,
        _ => {
            anyhow::bail!("unknown operation");
        }
    };
    Ok(())
}

use cursive::{
    Cursive,
    view::{Nameable, Resizable},
    views::{Button, Dialog, EditView, LinearLayout, SelectView, TextView},
};
use hodeauxledger_io::disk::rhex as diskrhex;
use hodeauxledger_io::screen::pretty_print_rhex;
use std::io::Write;
use std::path::Path;

use crate::argv::{BuildArgs, Command, FinalizeArgs, VerifyArgs};
use clap::Parser;

mod argv;
mod craft;
mod genesis;

//const VERSION: &str = env!("CARGO_PKG_VERSION");

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
fn build_intent(args: &BuildArgs) -> anyhow::Result<(), anyhow::Error> {
    let save_path = args.save.save.as_deref().unwrap_or("");
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
fn finalize_rhex(args: &FinalizeArgs) -> anyhow::Result<(), anyhow::Error> {
    let rhex_path = &args.rhex;
    let rhex = diskrhex::load_rhex(&Path::new(rhex_path).to_path_buf())?;
    let rhex = rhex.finalize()?;
    // output completed R⬢
    if let Some(save_path) = &args.save.save {
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
fn verify_current_hash(args: &VerifyArgs) -> anyhow::Result<(), anyhow::Error> {
    let rhex_path = &args.input;
    let rhex = diskrhex::load_rhex(&Path::new(rhex_path).to_path_buf())?;
    rhex.verify_hash()?;
    println!("✅ R⬢ hash verified.");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = argv::Cli::parse();
    let action = args.cmd;
    //println!(
    //    "{}{}",
    //    "HodeauxLedger Tool v".magenta().bold(),
    //    VERSION.magenta().bold()
    //);
    match action {
        Command::Build(args) => build_intent(&args)?,
        Command::Craft(args) => craft::craft_intent(&args)?,
        Command::Finalize(args) => finalize_rhex(&args)?,
        Command::Verify(args) => verify_current_hash(&args)?,
        Command::Genesis(args) => genesis::create_genesis(&args)?,
    };
    Ok(())
}

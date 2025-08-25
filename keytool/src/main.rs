use anyhow::Context;
use clap::Parser;
use ed25519_dalek::ed25519::signature::Verifier;
use ed25519_dalek::{Signature as DalekSig, VerifyingKey};
use hodeauxledger_core::crypto::key;
use hodeauxledger_core::{Signature as RhexSignature, to_base64};
use hodeauxledger_io::disk;
use owo_colors::OwoColorize;
use std::path::Path; // brings .verify() into scope

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "keytool", about = "HodeauxLedger Key Tool")]
struct Cli {
    action: String,

    #[arg(short, long)]
    load: Option<String>,

    #[arg(short, long)]
    save: Option<String>,

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

/**
 * Generate a keypair. If save isn't specified output to the screen.
 */
fn generate_key(args: Cli) -> Result<(), anyhow::Error> {
    let password = args
        .password
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("password must be specified"))?;
    let save_path = args
        .save
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("save must be specified"))?;
    let show_private_key = args.show_private_key;
    let quiet = args.quiet;
    let verbose = args.verbose;
    println!("Generating keypair...");
    let sk64 = key::generate_key();
    let sk = key::sk64_to_signing_key(&sk64);
    let pk = sk.verifying_key();
    if !quiet {
        println!("Public key: {}", to_base64(&pk.to_bytes()));
        if show_private_key {
            println!("Private key: {}", to_base64(&sk64));
        }
    }
    if verbose {
        println!("Saving key to {}", save_path);
    }
    let _ = disk::save_key(Path::new(save_path), &password, &sk)?;

    Ok(())
}

fn view_key(args: Cli) -> Result<(), anyhow::Error> {
    if (args.load).is_none() {
        anyhow::bail!("Must specify a keyfile to load")
    }
    if (args.password).is_none() {
        anyhow::bail!("Must specify a password to decrypt the keyfile")
    }
    let keyfile = args.load.unwrap_or("".to_string());
    let password = args.password.unwrap_or("".to_string());
    let sk = disk::load_key(&keyfile, &password)?;
    println!("Public key: {}", to_base64(&sk.verifying_key().to_bytes()));
    if args.show_private_key {
        println!("Private key: {}", to_base64(&key::signing_key_to_sk64(&sk)));
    }
    Ok(())
}

fn sign(args: Cli) -> Result<(), anyhow::Error> {
    // Gotta figure out why this is not signing the same thing
    // verify is verifying... chatGPT seems to think its to do with
    // the signatures which it shouldn't even be fucking with on the
    // author stage.
    let load = args
        .load
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("load must be specified"))?;
    let password = args
        .password
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("password must be specified"))?;
    let rhex = args
        .rhex
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("rhex must be specified"))?;
    let rhex_output = args.rhex_output.as_deref();
    let signature_type = args
        .signature_type
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("signature_type must be specified"))?;
    let mut sig_id = 0;
    match signature_type {
        "author" => sig_id = 0,
        "usher" => sig_id = 1,
        "quorum" => sig_id = 2,
        _ => {}
    }

    if args.verbose {
        println!("Loading key from {}", load);
    }
    let sk = disk::load_key(&load, &password)?;
    println!("Public key: {}", to_base64(&sk.verifying_key().to_bytes()));
    if args.verbose {
        println!("Loading R⬢ from {}", rhex);
    }
    let p = Path::new(rhex);
    let rhex = disk::load_rhex(&p.to_path_buf())?;

    let sig: ed25519_dalek::Signature = match signature_type {
        "author" => {
            // Old line to get the hash... we now use the fn off Rhex
            //let message_bytes = rhex.intent.canonical_bytes()?;
            let message_hash = rhex.to_author_hash()?;
            key::sign(&message_hash, &sk)
        }
        "usher" => {
            // Lawd have mercy this was some frail ass shit.
            //let author_sig = rhex.signatures[0].sig;
            //let at = rhex.context.at;
            //let at_bytes = at.to_be_bytes();
            //let mut buf = Vec::with_capacity(author_sig.len() + at_bytes.len());
            //buf.extend_from_slice(&author_sig);
            //buf.extend_from_slice(&at_bytes);
            let usher_context_hash = rhex.to_usher_hash()?;
            key::sign(&usher_context_hash, &sk)
        }
        "quorum" => {
            // Nope... not here either!
            //let mut buf = Vec::new();
            //buf.extend_from_slice(&rhex.signatures[0].sig);
            //buf.extend_from_slice(&rhex.signatures[1].sig);
            let sig_hash = rhex.to_quorum_hash()?;
            key::sign(&sig_hash, &sk)
        }
        _ => anyhow::bail!("unknown signature type {signature_type}"),
    };
    if let Some(rhex_output) = rhex_output {
        let mut out_rhex = rhex.clone();
        let new_sig = RhexSignature {
            sig_type: sig_id,
            public_key: sk.verifying_key().to_bytes(),
            sig: sig.to_bytes(),
        };
        out_rhex.signatures.push(new_sig);
        disk::save_rhex(&Path::new(rhex_output).to_path_buf(), &out_rhex)?;
    }
    Ok(())
}

fn verify(args: Cli) -> Result<(), anyhow::Error> {
    let load_rhex = args
        .rhex
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("rhex must be specified"))?;
    let quiet = args.quiet;
    let verbose = args.verbose;

    if !quiet {
        println!("Loading rhex from disk {load_rhex}...");
    }
    let rhex = disk::load_rhex(&Path::new(load_rhex).to_path_buf())?;

    // Precompute the author message hash: blake3(intent.canonical_bytes())
    let author_msg_hash = rhex.to_author_hash()?;

    for (i, sigrec) in rhex.signatures.iter().enumerate() {
        // Recreate verifier + signature types
        let vk = VerifyingKey::from_bytes(&sigrec.public_key)
            .with_context(|| format!("bad verifying key in signature #{i}"))?;
        let sig = DalekSig::from_bytes(&sigrec.sig);

        // Build the exact message that was signed for this sig_type
        let msg: [u8; 32] = match sigrec.sig_type {
            // author: H(canonical intent)
            0 => author_msg_hash,

            // usher: H( author_sig || rhex.context.at.to_be_bytes() )
            1 => rhex.to_usher_hash()?,

            // quorum: H( author_sig || usher_sig )
            2 => rhex.to_quorum_hash()?,

            _ => anyhow::bail!("unknown signature type {}", sigrec.sig_type),
        };

        // Verify
        match sigrec.sig_type {
            0 => {
                if verbose {
                    println!("Verifying author signature...");
                }
                if !key::verify(&msg, &sig, &vk) {
                    anyhow::bail!("❌ Author verification failed");
                }
            }
            1 => {
                if !key::verify(&msg, &sig, &vk) {
                    anyhow::bail!("❌ Usher verification failed");
                }
            }
            2 => {
                if !key::verify(&msg, &sig, &vk) {
                    anyhow::bail!("❌ Quorum verification failed");
                }
            }
            _ => {
                if !vk.verify(&msg, &sig).is_ok() {
                    anyhow::bail!(
                        "verification failed for signature #{i} (type {})",
                        sigrec.sig_type
                    );
                }
            }
        }

        //vk.verify(&msg, &sig).with_context(|| {
        //    format!(
        //       "verification failed for signature #{i} (type {})",
        //        sigrec.sig_type
        //    )
        //})?;

        if verbose {
            println!(
                "✅ verified sig #{i} (type {}), pk={}, sig={}, sig_hash={}",
                sigrec.sig_type,
                to_base64(&sigrec.public_key),
                to_base64(&sigrec.sig),
                to_base64(&msg)
            );
        }
    }

    if !quiet {
        println!("All signatures verified.");
    }
    Ok(())
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
        "generate" => generate_key(args)?,
        "view" => view_key(args)?,
        "sign" => sign(args)?,
        "verify" => verify(args)?,
        _ => {
            anyhow::bail!("unknown operation");
        }
    };
    Ok(())
}

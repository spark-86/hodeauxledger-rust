use std::path::Path;

use anyhow::{Context, Ok, Result, anyhow, bail, ensure};
use ed25519_dalek::{Signature as DalekSig, SigningKey, VerifyingKey};
use hodeauxledger_core::rhex::signature::SigType;
use hodeauxledger_core::to_base64;
use hodeauxledger_core::{Rhex, Signature, crypto::key};
use hodeauxledger_io::disk;

use crate::{Cli, crypto};

pub fn sign_rhex(
    mut rhex: Rhex,
    sig_type: SigType,
    sk: &SigningKey,
    verbose: bool,
) -> Result<Rhex> {
    // enforce signing order / prerequisites
    check_for_sigs(&rhex, sig_type)?;

    // pick the correct hash to sign
    let hash = match sig_type {
        SigType::Author => rhex.to_author_hash()?,
        SigType::Usher => rhex.to_usher_hash()?,
        SigType::Quorum => rhex.to_quorum_hash()?,
    };

    // sign without extra allocs
    let sig = key::sign(hash.as_ref(), sk);

    // append signature
    rhex.signatures.push(Signature {
        sig_type: sig_type.into(),                 // u8
        public_key: sk.verifying_key().to_bytes(), // [u8; 32]
        sig: sig.to_bytes(),                       // [u8; 64]
    });

    if verbose {
        println!("🔑 Signature added for type {:?}", sig_type);
    }

    Ok(rhex) // caller gets the new value; original binding was moved
}

fn check_for_sigs(rhex: &Rhex, sig_type: SigType) -> Result<()> {
    let has_author = rhex.signatures.iter().any(|s| s.sig_type == 0);
    let has_usher = rhex.signatures.iter().any(|s| s.sig_type == 1);

    match sig_type {
        SigType::Author => {
            ensure!(!has_author, "author signature already present");
        }
        SigType::Usher => {
            ensure!(has_author, "author signature is required before usher");
            ensure!(!has_usher, "usher signature already present");
        }
        SigType::Quorum => {
            ensure!(has_author, "author signature missing; cannot add quorum");
            ensure!(has_usher, "usher signature missing; cannot add quorum");
        }
    }
    Ok(())
}

pub fn verify_rhex(rhex: &Rhex, verbose: bool) -> Result<bool> {
    for sigrec in &rhex.signatures {
        // Get VerifyingKey from our signatures public key
        let vk =
            VerifyingKey::from_bytes(&sigrec.public_key).context("invalid public key bytes")?;

        // Get the signature itself
        let sig = DalekSig::from_bytes(&sigrec.sig);

        // Pick the right message to verify the hash on
        let st = SigType::try_from(sigrec.sig_type);
        if st.is_err() {
            return Ok(false);
        }
        let sigrec = st.unwrap();
        let msg = match sigrec {
            SigType::Author => rhex.to_author_hash()?,
            SigType::Usher => rhex.to_usher_hash()?,
            SigType::Quorum => rhex.to_quorum_hash()?,
        };
        if !key::verify(&msg, &sig, &vk) {
            if verbose {
                println!("❌ Signature verification failed for type {:?}", sigrec);
            }
            return Ok(false);
        }
        if verbose {
            println!("✅ Signature verified for type {:?}", sigrec);
        }
    }
    Ok(true)
}

pub fn sign(args: Cli) -> Result<(), anyhow::Error> {
    // Parse command line args
    let load = args
        .load
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("load must be specified"))?;
    let password_opt = args.password.as_deref();
    let hot = args.hot;
    let verbose = args.verbose;
    if !hot && password_opt.is_none() {
        anyhow::bail!("password must be specified when not using --hot");
    }
    let password = password_opt.unwrap_or("");
    let rhex_in = args
        .rhex
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("rhex must be specified"))?;
    let rhex_output = args
        .rhex_output
        .as_deref()
        .ok_or_else(|| anyhow!("rhex_output must be specified"))?;
    let signature_type = match args.signature_type.as_deref() {
        Some("author") | Some("0") => SigType::Author,
        Some("usher") | Some("1") => SigType::Usher,
        Some("quorum") | Some("2") => SigType::Quorum,
        Some(&_) => {
            bail!("signature_type must be one of: author, usher, quorum");
        }
        None => {
            bail!("signature_type must be specified");
        }
    };

    // Load the key to sign with
    if verbose {
        println!("Loading key from {}", load);
    }
    let sk = if hot {
        crypto::load_hot_key(Path::new(load))?
    } else {
        crypto::load_encrypted_key(Path::new(load), password)?
    };
    println!("Public key: {}", to_base64(&sk.verifying_key().to_bytes()));
    if verbose {
        println!("Loading R⬢ from {}", rhex_in);
    }
    let p = Path::new(rhex_in);
    let rhex = disk::load_rhex(&p.to_path_buf())?;

    let done_rhex = match signature_type {
        SigType::Author => sign_rhex(rhex, signature_type, &sk, verbose)?,
        SigType::Usher => sign_rhex(rhex, signature_type, &sk, verbose)?,
        SigType::Quorum => sign_rhex(rhex, signature_type, &sk, verbose)?,
    };

    disk::save_rhex(&Path::new(rhex_output).to_path_buf(), &done_rhex)?;
    Ok(())
}

pub fn verify(args: Cli) -> Result<(), anyhow::Error> {
    let rhex_in = args
        .rhex
        .as_deref()
        .ok_or_else(|| anyhow!("rhex must be specified"))?;
    let verbose = args.verbose;
    let rhex = disk::load_rhex(&Path::new(rhex_in).to_path_buf())?;
    let err = verify_rhex(&rhex, verbose);
    if err.is_err() {
        anyhow::bail!("Signature verification failed");
    }

    Ok(())
}
